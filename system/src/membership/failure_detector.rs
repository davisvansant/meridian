use tokio::time::{sleep, timeout, Duration};

use tokio::sync::mpsc;

use crate::channel::MembershipListSender;
use crate::channel::ShutdownReceiver;
use crate::channel::{
    get_alive, get_confirmed, get_node, get_suspected, insert_alive, insert_confirmed,
    insert_suspected, remove_alive, remove_confirmed, remove_suspected, send_message,
};
use crate::channel::{MembershipCommunicationsMessage, MembershipCommunicationsSender};
use crate::channel::{MembershipFailureDetectorReceiver, MembershipFailureDetectorRequest};
use crate::membership::Message;

use crate::node::Node;

use std::net::{IpAddr, SocketAddr};

use std::str::FromStr;

pub struct FailureDectector {
    protocol_period: Duration,
    list_sender: MembershipListSender,
    send_udp_message: MembershipCommunicationsSender,
    receiver: MembershipFailureDetectorReceiver,
}

impl FailureDectector {
    pub async fn init(
        list_sender: MembershipListSender,
        receiver: MembershipFailureDetectorReceiver,
        send_udp_message: MembershipCommunicationsSender,
    ) -> FailureDectector {
        let protocol_period = Duration::from_secs(10);

        FailureDectector {
            protocol_period,
            list_sender,
            receiver,
            send_udp_message,
        }
    }

    pub async fn run(
        &mut self,
        shutdown: &mut ShutdownReceiver,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(request) = self.receiver.recv().await {
            match request {
                MembershipFailureDetectorRequest::Launch => {
                    println!("launching membership failure detector!");
                }
            }
        }

        loop {
            tokio::select! {
                biased;
                _ = shutdown.recv() => {
                    println!("shutting down failure dectector...");

                    self.receiver.close();

                    break
                }
                result = self.probe() => {
                    match result {
                        Ok(()) => println!("probe complete!"),
                        Err(error) => println!("probe failed with error -> {:?}", error),
                    }
                }
            }
        }

        Ok(())
    }

    async fn probe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (_placeholder_sender, mut placeholder_receiver) = mpsc::channel::<Node>(1);

        let alive_list = get_alive(&self.list_sender).await?;

        for member in alive_list {
            let address = member.membership_address().await;
            let node = get_node(&self.list_sender).await?;
            let local_alive_list = get_alive(&self.list_sender).await?;
            let local_suspected_list = get_suspected(&self.list_sender).await?;
            let local_confirmed_list = get_confirmed(&self.list_sender).await?;

            let ping = Message::Ping
                .build_list(
                    &node,
                    &local_alive_list,
                    &local_suspected_list,
                    &local_confirmed_list,
                )
                .await;

            send_message(&self.send_udp_message, &ping, address).await?;

            match timeout(self.protocol_period, placeholder_receiver.recv()).await {
                Ok(Some(ping_target)) => {
                    if member == ping_target {
                        remove_suspected(&self.list_sender, &member).await?;
                        remove_confirmed(&self.list_sender, &member).await?;
                        insert_alive(&self.list_sender, &member).await?;
                    } else {
                        println!("nodes dont match!");
                    }
                }
                Ok(None) => {
                    println!("failed to receive response from suspected node...");

                    remove_alive(&self.list_sender, &member).await?;
                    remove_confirmed(&self.list_sender, &member).await?;
                    insert_suspected(&self.list_sender, &member).await?;
                }
                Err(error) => {
                    println!(
                        "membership failure detector protocol period expired... {:?}",
                        error,
                    );

                    remove_alive(&self.list_sender, &member).await?;
                    remove_confirmed(&self.list_sender, &member).await?;
                    insert_suspected(&self.list_sender, &member).await?;
                }
            }
        }

        Ok(())
    }
}
