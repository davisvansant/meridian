use tokio::time::{sleep, timeout, Duration};

use tokio::sync::mpsc;

use crate::channel::MembershipListSender;
use crate::channel::ShutdownReceiver;
use crate::channel::{
    get_alive, insert_alive, insert_confirmed, insert_suspected, remove_alive, remove_confirmed,
    remove_suspected,
};
use crate::channel::{MembershipCommunicationsMessage, MembershipCommunicationsSender};
use crate::channel::{MembershipFailureDetectorReceiver, MembershipFailureDetectorRequest};

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
        let address = IpAddr::from_str("0.0.0.0")?;
        let placeholder_suspected_node = Node::init(address, 10000, 15000, 25000).await?;

        let (_placeholder_sender, mut placeholder_receiver) = mpsc::channel::<Node>(1);

        match timeout(self.protocol_period, placeholder_receiver.recv()).await {
            Ok(Some(placeholder_node)) => {
                if placeholder_node == placeholder_suspected_node {
                    remove_suspected(&self.list_sender, &placeholder_node).await?;
                    remove_confirmed(&self.list_sender, &placeholder_node).await?;
                    insert_alive(&self.list_sender, &placeholder_node).await?;
                } else {
                    println!("nodes dont match!");
                }
            }
            Ok(None) => {
                println!("failed to receive response from suspected node...");

                remove_alive(&self.list_sender, &placeholder_suspected_node).await?;
                remove_confirmed(&self.list_sender, &placeholder_suspected_node).await?;
                insert_suspected(&self.list_sender, &placeholder_suspected_node).await?;
            }
            Err(error) => {
                println!(
                    "membership failure detector protocol period expired... {:?}",
                    error,
                );

                remove_alive(&self.list_sender, &placeholder_suspected_node).await?;
                remove_confirmed(&self.list_sender, &placeholder_suspected_node).await?;
                insert_suspected(&self.list_sender, &placeholder_suspected_node).await?;
            }
        }

        Ok(())
    }
}
