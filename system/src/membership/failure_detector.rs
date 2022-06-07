use tokio::time::{timeout, Duration};

use crate::channel::MembershipListSender;
use crate::channel::ShutdownReceiver;
use crate::{error, info, warn};
// use crate::channel::{
//     get_alive, get_confirmed, get_node, get_suspected, insert_alive, insert_confirmed,
//     insert_suspected, remove_alive, remove_confirmed, remove_suspected, send_message,
// };
use crate::channel::{
    get_alive, get_confirmed, get_node, get_suspected, insert_alive, insert_suspected,
    remove_alive, remove_confirmed, remove_suspected, send_message,
};
// use crate::channel::{MembershipCommunicationsMessage, MembershipCommunicationsSender};
use crate::channel::MembershipCommunicationsSender;
use crate::channel::{
    MembershipFailureDetectorPingTarget, MembershipFailureDetectorPingTargetSender,
};
// use crate::channel::{MembershipFailureDetectorReceiver, MembershipFailureDetectorRequest};
use crate::channel::MembershipFailureDetectorReceiver;
use crate::membership::Message;

pub struct FailureDectector {
    protocol_period: Duration,
    list_sender: MembershipListSender,
    send_udp_message: MembershipCommunicationsSender,
    receiver: MembershipFailureDetectorReceiver,
    ping_target_channel: MembershipFailureDetectorPingTargetSender,
}

impl FailureDectector {
    pub async fn init(
        list_sender: MembershipListSender,
        receiver: MembershipFailureDetectorReceiver,
        send_udp_message: MembershipCommunicationsSender,
        ping_target_channel: MembershipFailureDetectorPingTargetSender,
    ) -> FailureDectector {
        let protocol_period = Duration::from_secs(10);

        FailureDectector {
            protocol_period,
            list_sender,
            receiver,
            send_udp_message,
            ping_target_channel,
        }
    }

    pub async fn run(
        &mut self,
        shutdown: &mut ShutdownReceiver,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(()) = self.receiver.changed().await {
            // println!("launching membership failure detector!");
            info!("launching membership failure detector!");
        }

        loop {
            tokio::select! {
                biased;
                _ = shutdown.recv() => {
                    // println!("shutting down failure dectector...");
                    info!("shutting down failure dectector...");

                    break
                }
                result = self.probe() => {
                    match result {
                        Ok(()) => {
                            // println!("probe complete!"),
                            info!("probe complete!");
                        }
                        Err(error) => {
                            // println!("probe failed with error -> {:?}", error),
                            error!("probe failed with error -> {:?}", error);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn probe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut receive_ping_target_ack = self.ping_target_channel.subscribe();

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

            match timeout(self.protocol_period, receive_ping_target_ack.recv()).await {
                Ok(Ok(MembershipFailureDetectorPingTarget::Member(ping_target))) => {
                    if address == ping_target {
                        remove_suspected(&self.list_sender, &member).await?;
                        remove_confirmed(&self.list_sender, &member).await?;
                        insert_alive(&self.list_sender, &member).await?;
                    } else {
                        // println!("nodes dont match!");
                        warn!("nodes dont match!");
                    }
                }
                Ok(Err(error)) => {
                    // println!("error with ping target channel -> {:?}", error);
                    error!("error with ping target channel -> {:?}", error);

                    remove_alive(&self.list_sender, &member).await?;
                    remove_confirmed(&self.list_sender, &member).await?;
                    insert_suspected(&self.list_sender, &member).await?;
                }
                Err(error) => {
                    // println!(
                    //     "membership failure detector protocol period expired... {:?}",
                    //     error,
                    // );
                    error!(
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
