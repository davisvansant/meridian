use std::net::IpAddr;
use std::str::FromStr;

use tokio::time::{sleep, timeout, Duration};

use crate::channel::membership_communications::send_message;
use crate::channel::membership_communications::MembershipCommunicationsSender;
use crate::channel::membership_failure_detector::{
    MembershipFailureDetectorPingTarget, MembershipFailureDetectorPingTargetSender,
    MembershipFailureDetectorReceiver,
};
use crate::channel::membership_list::MembershipListSender;
use crate::channel::membership_list::{
    get_alive, get_confirmed, get_node, get_suspected, insert_alive, insert_suspected,
    remove_alive, remove_confirmed, remove_suspected,
};
use crate::channel::transition::ShutdownReceiver;
use crate::membership::Message;
use crate::{error, info, warn};

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

        info!("initialized!");

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
            info!("launching membership failure detector!");
        }

        loop {
            tokio::select! {
                biased;
                _ = shutdown.recv() => {
                    info!("shutting down...");

                    break
                }
                result = self.probe() => {
                    match result {
                        Ok(()) => {
                            info!("probe complete!");
                        }
                        Err(error) => {
                            error!("probe failed with error -> {:?}", error);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn probe(&self) -> Result<(), Box<dyn std::error::Error>> {
        sleep(Duration::from_secs(5)).await;

        let mut receive_ping_target_ack = self.ping_target_channel.subscribe();

        let alive_list = get_alive(&self.list_sender).await?;

        for member in alive_list {
            let mut address = member.membership_address().await;
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

            // For local testing, incoming UDP socket recv_from socket address is 127.0.0.1
            address.set_ip(IpAddr::from_str("127.0.0.1")?);

            info!("sending message to -> {:?}", &address);

            send_message(&self.send_udp_message, &ping, address).await?;

            match timeout(self.protocol_period, receive_ping_target_ack.recv()).await {
                Ok(Ok(MembershipFailureDetectorPingTarget::Member(ping_target))) => {
                    if address == ping_target {
                        remove_suspected(&self.list_sender, &member).await?;
                        remove_confirmed(&self.list_sender, &member).await?;
                        insert_alive(&self.list_sender, &member).await?;
                    } else {
                        warn!("nodes dont match!");
                        warn!(
                            "sent address = {:?} | received address {:?}",
                            &address, &ping_target,
                        );
                    }
                }
                Ok(Err(error)) => {
                    error!("error with ping target channel -> {:?}", error);

                    remove_alive(&self.list_sender, &member).await?;
                    remove_confirmed(&self.list_sender, &member).await?;
                    insert_suspected(&self.list_sender, &member).await?;
                }
                Err(error) => {
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
