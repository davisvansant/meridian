use std::collections::HashMap;
use std::net::SocketAddr;

use uuid::Uuid;

use crate::channel::{MembershipReceiver, MembershipRequest, MembershipResponse};
use crate::node::Node;

use communications::MembershipCommunications;
use list::List;

mod communications;
mod list;

#[derive(Clone, Debug, PartialEq)]
pub enum ClusterSize {
    One,
    Three,
    Five,
}

impl ClusterSize {
    pub async fn members(&self) -> HashMap<Uuid, Node> {
        match self {
            ClusterSize::One => HashMap::with_capacity(1),
            ClusterSize::Three => HashMap::with_capacity(3),
            ClusterSize::Five => HashMap::with_capacity(5),
        }
    }
}

// #[derive(Clone, Debug)]
pub struct Membership {
    cluster_size: ClusterSize,
    server: Node,
    list: List,
    // launch_nodes: Vec<SocketAddr>,
    // members: HashMap<Uuid, Node>,
    receiver: MembershipReceiver,
}

impl Membership {
    pub async fn init(
        cluster_size: ClusterSize,
        launch_nodes: Vec<SocketAddr>,
        server: Node,
        receiver: MembershipReceiver,
    ) -> Result<Membership, Box<dyn std::error::Error>> {
        // let members = cluster_size.members().await;
        let list = List::init(launch_nodes).await?;

        Ok(Membership {
            cluster_size,
            // launch_nodes,
            server,
            list,
            // members,
            receiver,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let membership_port = self.server.membership_address().await;
        let mut communications = MembershipCommunications::init(membership_port).await;

        tokio::spawn(async move {
            if let Err(error) = communications.run().await {
                println!("error with membership communications -> {:?}", error);
            }
        });

        println!("membership initialized and running...");

        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                MembershipRequest::AddMember(node) => {
                    println!("adding new member...");

                    if self.server == node {
                        println!("not adding self to members");
                    } else {
                        // match self.members.insert(node.id, node) {
                        // match self.list.insert_alive(node.id, node) {
                        //     Some(value) => println!("updated node! {:?}", value),
                        //     None => println!("added node !"),
                        // }
                        self.list.insert_alive(node).await?;
                    }

                    if let Err(error) = response.send(MembershipResponse::Ok) {
                        println!("error sending membership response -> {:?}", error);
                    }
                }
                MembershipRequest::LaunchNodes => {
                    println!("retrieving initial launch nodes...");

                    // let mut launch_nodes = Vec::with_capacity(self.launch_nodes.len());

                    let mut launch_nodes = Vec::with_capacity(self.list.initial.len());

                    // for node in &self.launch_nodes {
                    //     launch_nodes.push(node.to_owned());
                    // }
                    for node in &self.list.initial {
                        launch_nodes.push(node.to_owned());
                    }

                    if let Err(error) = response.send(MembershipResponse::LaunchNodes(launch_nodes))
                    {
                        println!("error sending membership response -> {:?}", error);
                    }
                }
                MembershipRequest::Members => {
                    println!("received members request!");

                    // let mut members = Vec::with_capacity(self.members.len());
                    let mut members = Vec::with_capacity(self.list.alive.len());

                    // for m in self.members.values() {
                    //     println!("peers !{:?}", m);

                    //     members.push(m.to_owned());
                    // }
                    for m in self.list.alive.values() {
                        println!("peers !{:?}", m);

                        members.push(m.to_owned());
                    }

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
                MembershipRequest::RemoveMember => {
                    println!("removing member...");

                    if let Err(error) = response.send(MembershipResponse::Ok) {
                        println!("error sending membership response -> {:?}", error);
                    }
                }
                MembershipRequest::Status => {
                    // let connected_nodes = match self.members.len() {
                    let connected_nodes = match self.list.alive.len() {
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
