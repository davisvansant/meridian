use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use uuid::Uuid;

// use crate::grpc::membership_client::ExternalMembershipGrpcClient;
use crate::node::Node;
use crate::{MembershipNode, NodeStatus, Nodes};

// use crate::runtime::sync::membership_receive_task::ChannelMembershipReceiveTask;
// use crate::runtime::sync::membership_receive_task::MembershipReceiveTask;
// use crate::runtime::sync::membership_send_grpc_task::ChannelMembershipSendGrpcTask;
// use crate::runtime::sync::membership_send_grpc_task::MembershipSendGrpcTask;
// use crate::runtime::sync::membership_send_preflight_task::ChannelMembershipSendPreflightTask;
// use crate::runtime::sync::membership_send_preflight_task::MembershipSendPreflightTask;
// use crate::runtime::sync::membership_send_server_task::ChannelMembershipSendServerTask;
// use crate::runtime::sync::membership_send_server_task::MembershipSendServerTask;

use crate::channel::{MembershipReceiver, MembershipRequest, MembershipResponse, MembershipSender};

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
    members: HashMap<Uuid, Node>,
    // receive_task: ChannelMembershipReceiveTask,
    // send_grpc_task: ChannelMembershipSendGrpcTask,
    // send_preflight_task: ChannelMembershipSendPreflightTask,
    // send_server_task: ChannelMembershipSendServerTask,
    receiver: MembershipReceiver,
}

impl Membership {
    pub async fn init(
        cluster_size: ClusterSize,
        server: Node,
        // receive_task: ChannelMembershipReceiveTask,
        // send_grpc_task: ChannelMembershipSendGrpcTask,
        // send_preflight_task: ChannelMembershipSendPreflightTask,
        // send_server_task: ChannelMembershipSendServerTask,
        receiver: MembershipReceiver,
    ) -> Result<Membership, Box<dyn std::error::Error>> {
        let members = cluster_size.members().await;

        Ok(Membership {
            cluster_size,
            server,
            members,
            // send_grpc_task,
            // receive_task,
            // send_preflight_task,
            // send_server_task,
            receiver,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // let mut receiver = self.receive_task.subscribe();

        println!("membership initialized and running...");

        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                MembershipRequest::AddMember(node) => {
                    println!("adding new member...");

                    match self.members.insert(node.id, node) {
                        Some(value) => println!("updated node! {:?}", value),
                        None => println!("added node !"),
                    }

                    if let Err(error) = response.send(MembershipResponse::Ok) {
                        println!("error sending membership response -> {:?}", error);
                    }
                }
                MembershipRequest::Members(service) => {
                    println!("received members request!");

                    let mut members = Vec::with_capacity(self.members.len());

                    for m in self.members.values() {
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
                    let length = match self.members.len() {
                        0 => String::from("0"),
                        1 => String::from("1"),
                        2 => String::from("2"),
                        3 => String::from("3"),
                        _ => panic!("unexpected number of peers!"),
                    };

                    let system = NodeStatus { status: length };

                    if let Err(error) = response.send(MembershipResponse::Status(system.status)) {
                        println!("error sending membership response -> {:?}", error);
                    }
                }
            }
        }

        Ok(())
    }

    async fn action_join_cluster_request(
        &mut self,
        node: MembershipNode,
    ) -> Result<MembershipNode, Box<dyn std::error::Error>> {
        let peer = Self::build_node(node).await?;

        match self.members.insert(peer.id, peer) {
            Some(value) => println!("updated node! {:?}", value),
            None => println!("added node !"),
        }

        let response = MembershipNode {
            id: self.server.id.to_string(),
            address: self.server.address.to_string(),
            client_port: self.server.client_port.to_string(),
            cluster_port: self.server.cluster_port.to_string(),
            membership_port: self.server.membership_port.to_string(),
        };

        Ok(response)
    }

    // async fn build_node(node: MembershipNode) -> Result<Node, Box<dyn std::error::Error>> {
    async fn build_node(node: MembershipNode) -> Result<Node, Box<dyn std::error::Error>> {
        let id = Uuid::from_str(&node.id)?;
        let address = IpAddr::from_str(&node.address)?;
        let client_port = u16::from_str(&node.client_port)?;
        let cluster_port = u16::from_str(&node.cluster_port)?;
        let membership_port = u16::from_str(&node.membership_port)?;

        Ok(Node {
            id,
            address,
            client_port,
            cluster_port,
            membership_port,
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn init_one() -> Result<(), Box<dyn std::error::Error>> {
//         let test_membership = Membership::init(ClusterSize::One).await?;
//         assert_eq!(test_membership.cluster_size, ClusterSize::One);
//         assert_eq!(test_membership.members.len(), 0);
//         assert_eq!(test_membership.members.capacity(), 1);
//         Ok(())
//     }
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn init_three() -> Result<(), Box<dyn std::error::Error>> {
//         let test_membership = Membership::init(ClusterSize::Three).await?;
//         assert_eq!(test_membership.cluster_size, ClusterSize::Three);
//         assert_eq!(test_membership.members.len(), 0);
//         assert_eq!(test_membership.members.capacity(), 3);
//         Ok(())
//     }
//
//     #[tokio::test(flavor = "multi_thread")]
//     async fn init_five() -> Result<(), Box<dyn std::error::Error>> {
//         let test_membership = Membership::init(ClusterSize::Five).await?;
//         assert_eq!(test_membership.cluster_size, ClusterSize::Five);
//         assert_eq!(test_membership.members.len(), 0);
//         assert_eq!(test_membership.members.capacity(), 5);
//         Ok(())
//     }
// }
