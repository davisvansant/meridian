// use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::sync::{mpsc, oneshot};

// use uuid::Uuid;

use crate::channel::{get_alive, shutdown_membership_list};
use crate::channel::{MembershipListReceiver, MembershipListRequest, MembershipListResponse};
use crate::channel::{MembershipReceiver, MembershipRequest, MembershipResponse};
use crate::node::Node;

use communications::MembershipCommunications;
use failure_detector::FailureDectector;
use list::List;

mod communications;
mod failure_detector;
mod list;
mod static_join;

#[derive(Clone, Debug, PartialEq)]
pub enum ClusterSize {
    One,
    Three,
    Five,
}

// impl ClusterSize {
//     pub async fn members(&self) -> HashMap<Uuid, Node> {
//         match self {
//             ClusterSize::One => HashMap::with_capacity(1),
//             ClusterSize::Three => HashMap::with_capacity(3),
//             ClusterSize::Five => HashMap::with_capacity(5),
//         }
//     }
// }

// #[derive(Clone, Debug)]
pub struct Membership {
    cluster_size: ClusterSize,
    server: Node,
    receiver: MembershipReceiver,
}

impl Membership {
    pub async fn init(
        cluster_size: ClusterSize,
        server: Node,
        receiver: MembershipReceiver,
    ) -> Result<Membership, Box<dyn std::error::Error>> {
        Ok(Membership {
            cluster_size,
            server,
            receiver,
        })
    }

    pub async fn run(
        &mut self,
        launch_nodes: Vec<SocketAddr>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (list_sender, list_receiver) = mpsc::channel::<(
            MembershipListRequest,
            oneshot::Sender<MembershipListResponse>,
        )>(64);

        let mut list = List::init(launch_nodes, list_receiver).await?;

        tokio::spawn(async move {
            if let Err(error) = list.run().await {
                println!("membership list error -> {:?}", error);
            }
        });

        let membership_port = self.server.membership_address().await;
        let mut communications = MembershipCommunications::init(membership_port).await;

        tokio::spawn(async move {
            if let Err(error) = communications.run().await {
                println!("error with membership communications -> {:?}", error);
            }
        });

        let mut failure_detector = FailureDectector::init().await;

        tokio::spawn(async move {
            if let Err(error) = failure_detector.run().await {
                println!("error with membership failure dector -> {:?}", error);
            }
        });

        println!("membership initialized and running...");

        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                MembershipRequest::Members => {
                    println!("received members request!");

                    let members = get_alive(&list_sender).await?;

                    if let Err(error) = response.send(MembershipResponse::Members(members)) {
                        println!("error sending membership response -> {:?}", error);
                    }
                }
                MembershipRequest::Node => {
                    let node = self.server;

                    if let Err(error) = response.send(MembershipResponse::Node(node)) {
                        println!("error sending membership response -> {:?}", error);
                    }
                }
                MembershipRequest::Status => {
                    let members = get_alive(&list_sender).await?;
                    let connected_nodes = match members.len() {
                        0 => 0,
                        1 => 1,
                        2 => 2,
                        3 => 3,
                        _ => panic!("unexpected number of peers!"),
                    };

                    if let Err(error) = response.send(MembershipResponse::Status(connected_nodes)) {
                        println!("error sending membership response -> {:?}", error);
                    }
                }
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
