use std::net::SocketAddr;

use tokio::sync::{broadcast, mpsc, oneshot};

use crate::channel::MembershipCommunicationsMessage;
// use crate::channel::ShutdownReceiver;
use crate::channel::ShutdownSender;
use crate::channel::{get_alive, shutdown_membership_list};
use crate::channel::{MembershipListRequest, MembershipListResponse};
use crate::channel::{MembershipReceiver, MembershipRequest, MembershipResponse};
use crate::node::Node;

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

    pub async fn len(&self) -> usize {
        match self {
            ClusterSize::One => 1,
            ClusterSize::Three => 3,
            ClusterSize::Five => 5,
        }
    }
}

pub struct Membership {
    cluster_size: ClusterSize,
    // server: Node,
    receiver: MembershipReceiver,
    shutdown: ShutdownSender,
}

impl Membership {
    pub async fn init(
        cluster_size: ClusterSize,
        // server: Node,
        receiver: MembershipReceiver,
        shutdown: ShutdownSender,
    ) -> Result<Membership, Box<dyn std::error::Error>> {
        Ok(Membership {
            cluster_size,
            // server,
            receiver,
            shutdown,
        })
    }

    pub async fn run(
        &mut self,
        server: Node,
        launch_nodes: Vec<SocketAddr>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (list_sender, list_receiver) = mpsc::channel::<(
            MembershipListRequest,
            oneshot::Sender<MembershipListResponse>,
        )>(64);
        let static_join_send_list = list_sender.to_owned();
        let communications_list_sender = list_sender.to_owned();
        let failure_detector_list_sender = list_sender.to_owned();

        let mut list = List::init(server, launch_nodes, list_receiver).await?;

        tokio::spawn(async move {
            if let Err(error) = list.run().await {
                println!("membership list error -> {:?}", error);
            }
        });

        let (send_udp_message, _) = broadcast::channel::<MembershipCommunicationsMessage>(64);
        let membership_communications_sender = send_udp_message.clone();
        let static_join_send_udp_message = send_udp_message.clone();

        // let membership_port = self.server.membership_address().await;
        let membership_port = server.membership_address().await;

        let mut communications = MembershipCommunications::init(
            membership_port,
            communications_list_sender,
            membership_communications_sender,
        )
        .await;

        let mut communications_shutdown = self.shutdown.subscribe();
        let mut failure_detector_shutdown = self.shutdown.subscribe();

        drop(self.shutdown.to_owned());

        tokio::spawn(async move {
            if let Err(error) = communications.run(&mut communications_shutdown).await {
                println!("error with membership communications -> {:?}", error);
            }
        });

        let mut failure_detector = FailureDectector::init(failure_detector_list_sender).await;

        tokio::spawn(async move {
            if let Err(error) = failure_detector.run(&mut failure_detector_shutdown).await {
                println!("error with membership failure dector -> {:?}", error);
            }
        });

        let mut static_join =
            StaticJoin::init(static_join_send_udp_message, static_join_send_list).await;

        println!("membership initialized and running...");

        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                MembershipRequest::FailureDectector => {
                    println!("launch failure dector");
                }
                MembershipRequest::Members => {
                    println!("received members request!");

                    let members = get_alive(&list_sender).await?;

                    if let Err(error) = response.send(MembershipResponse::Members(members)) {
                        println!("error sending membership response -> {:?}", error);
                    }
                }
                MembershipRequest::Node => {
                    // let node = self.server;
                    let node = server;

                    if let Err(error) = response.send(MembershipResponse::Node(node)) {
                        println!("error sending membership response -> {:?}", error);
                    }
                }
                MembershipRequest::StaticJoin => {
                    static_join.run().await?;

                    let alive = get_alive(&list_sender).await?;
                    let expected = self.cluster_size.len().await;

                    if let Err(error) =
                        response.send(MembershipResponse::Status((alive.len(), expected)))
                    {
                        println!("error sending membership response -> {:?}", error);
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
                    println!("shutting down membership...");

                    shutdown_membership_list(&list_sender).await?;

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
