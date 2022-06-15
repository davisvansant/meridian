use std::net::SocketAddr;

use crate::channel::membership::{MembershipReceiver, MembershipRequest, MembershipResponse};
use crate::channel::membership_failure_detector::{build, build_ping_target, launch};
use crate::channel::membership_list::{get_alive, shutdown};
// use crate::channel::shutdown::ShutdownSender;
use crate::channel::transition::ShutdownSender;
use crate::node::Node;
use crate::{error, info};

use communications::MembershipCommunications;
use failure_detector::FailureDectector;
use list::List;
use message::Message;
use static_join::StaticJoin;

mod communications;
mod failure_detector;
mod list;
mod message;
mod static_join;

#[derive(Clone, Debug, PartialEq)]
pub enum ClusterSize {
    One,
    Three,
    Five,
}

impl ClusterSize {
    pub async fn from_str(size: &str) -> ClusterSize {
        match size {
            "1" => ClusterSize::One,
            "3" => ClusterSize::Three,
            "5" => ClusterSize::Five,
            _ => panic!("Expected a cluster size of 1, 3, or 5"),
        }
    }

    pub async fn majority(&self) -> usize {
        match self {
            ClusterSize::One => 0,
            ClusterSize::Three => 2,
            ClusterSize::Five => 3,
        }
    }
}

pub struct Membership {
    cluster_size: ClusterSize,
    receiver: MembershipReceiver,
    shutdown: ShutdownSender,
}

impl Membership {
    pub async fn init(
        cluster_size: ClusterSize,
        receiver: MembershipReceiver,
        shutdown: ShutdownSender,
    ) -> Result<Membership, Box<dyn std::error::Error>> {
        info!("initialized!");

        Ok(Membership {
            cluster_size,
            receiver,
            shutdown,
        })
    }

    pub async fn run(
        &mut self,
        server: Node,
        launch_nodes: Vec<SocketAddr>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (list_sender, list_receiver) = crate::channel::membership_list::build().await;
        let static_join_send_list = list_sender.to_owned();
        let communications_list_sender = list_sender.to_owned();
        let failure_detector_list_sender = list_sender.to_owned();

        let failure_detector_ping_target_sender = build_ping_target().await;
        let communications_ping_target_sender = failure_detector_ping_target_sender.to_owned();

        let mut list = List::init(server, launch_nodes, list_receiver).await?;

        tokio::spawn(async move {
            if let Err(error) = list.run().await {
                error!("membership list -> {:?}", error);
            }
        });

        let send_udp_message = crate::channel::membership_communications::build().await;
        let membership_communications_sender = send_udp_message.clone();
        let static_join_send_udp_message = send_udp_message.clone();
        let failure_detector_send_udp_message = send_udp_message.clone();

        let membership_port = server.membership_address().await;

        let mut communications = MembershipCommunications::init(
            membership_port,
            communications_list_sender,
            membership_communications_sender,
            communications_ping_target_sender,
        )
        .await;

        let mut communications_shutdown = self.shutdown.subscribe();
        let mut failure_detector_shutdown = self.shutdown.subscribe();

        drop(self.shutdown.to_owned());

        tokio::spawn(async move {
            if let Err(error) = communications.run(&mut communications_shutdown).await {
                error!("membership communications -> {:?}", error);
            }
        });

        let (failure_detector_sender, failure_detector_reciever) = build().await;

        let mut failure_detector = FailureDectector::init(
            failure_detector_list_sender,
            failure_detector_reciever,
            failure_detector_send_udp_message,
            failure_detector_ping_target_sender,
        )
        .await;

        tokio::spawn(async move {
            if let Err(error) = failure_detector.run(&mut failure_detector_shutdown).await {
                error!("membership failure detector -> {:?}", error);
            }
        });

        let mut static_join =
            StaticJoin::init(static_join_send_udp_message, static_join_send_list).await;

        info!("running...");

        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                MembershipRequest::FailureDectector => {
                    launch(&failure_detector_sender).await?;
                }
                MembershipRequest::Members => {
                    info!("received members request!");

                    let members = get_alive(&list_sender).await?;

                    if let Err(error) = response.send(MembershipResponse::Members(members)) {
                        error!("error sending membership response -> {:?}", error);
                    }
                }
                MembershipRequest::Node => {
                    let node = server;

                    if let Err(error) = response.send(MembershipResponse::Node(node)) {
                        error!("error sending membership response -> {:?}", error);
                    }
                }
                MembershipRequest::StaticJoin => {
                    static_join.run().await?;

                    let alive = get_alive(&list_sender).await?;
                    let expected = self.cluster_size.majority().await;

                    if let Err(error) =
                        response.send(MembershipResponse::Status((alive.len(), expected)))
                    {
                        error!("error sending membership response -> {:?}", error);
                    }
                }
                // MembershipRequest::Status => {
                //     let members = get_alive(&list_sender).await?;
                //     let connected_nodes = match members.len() {
                //         0 => 0,
                //         1 => 1,
                //         2 => 2,
                //         3 => 3,
                //         _ => panic!("unexpected number of peers!"),
                //     };

                //     if let Err(error) = response.send(MembershipResponse::Status(connected_nodes)) {
                //         println!("error sending membership response -> {:?}", error);
                //     }
                // }
                MembershipRequest::Shutdown => {
                    info!("shutting down...");

                    shutdown(&list_sender).await?;

                    self.receiver.close();
                }
            }
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::str::FromStr;
//     use tokio::sync::{mpsc, oneshot};

//     #[tokio::test(flavor = "multi_thread")]
//     async fn init_one() -> Result<(), Box<dyn std::error::Error>> {
//         let test_launch_nodes = Vec::with_capacity(0);
//         let test_ip_address = SocketAddr::from_str("0.0.0.0:0")?.ip();
//         let test_server = Node::init(test_ip_address, 1000, 2000, 3000).await?;
//         let (test_tx, test_rx) =
//             mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);
//         let test_membership =
//             Membership::init(ClusterSize::One, test_launch_nodes, test_server, test_rx).await?;
//         assert_eq!(test_membership.cluster_size, ClusterSize::One);
//         assert_eq!(test_membership.members.len(), 0);
//         assert!(test_membership.members.capacity() >= 1);
//         assert!(!test_tx.is_closed());
//         Ok(())
//     }

//     #[tokio::test(flavor = "multi_thread")]
//     async fn init_three() -> Result<(), Box<dyn std::error::Error>> {
//         let test_launch_nodes = Vec::with_capacity(0);
//         let test_ip_address = SocketAddr::from_str("0.0.0.0:0")?.ip();
//         let test_server = Node::init(test_ip_address, 1000, 2000, 3000).await?;
//         let (test_tx, test_rx) =
//             mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);
//         let test_membership =
//             Membership::init(ClusterSize::Three, test_launch_nodes, test_server, test_rx).await?;
//         assert_eq!(test_membership.cluster_size, ClusterSize::Three);
//         assert_eq!(test_membership.members.len(), 0);
//         assert!(test_membership.members.capacity() >= 3);
//         assert!(!test_tx.is_closed());
//         Ok(())
//     }

//     #[tokio::test(flavor = "multi_thread")]
//     async fn init_five() -> Result<(), Box<dyn std::error::Error>> {
//         let test_launch_nodes = Vec::with_capacity(0);
//         let test_ip_address = SocketAddr::from_str("0.0.0.0:0")?.ip();
//         let test_server = Node::init(test_ip_address, 1000, 2000, 3000).await?;
//         let (test_tx, test_rx) =
//             mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);
//         let test_membership =
//             Membership::init(ClusterSize::Five, test_launch_nodes, test_server, test_rx).await?;
//         assert_eq!(test_membership.cluster_size, ClusterSize::Five);
//         assert_eq!(test_membership.members.len(), 0);
//         assert!(test_membership.members.capacity() >= 5);
//         assert!(!test_tx.is_closed());
//         Ok(())
//     }
// }
