use std::net::IpAddr;
use std::str::FromStr;
use tokio::sync::broadcast::Sender;
use uuid::Uuid;

use crate::node::Node;
use crate::{JoinClusterRequest, JoinClusterResponse, MembershipAction};

#[derive(Clone, Debug, PartialEq)]
pub enum ClusterSize {
    One,
    Three,
    Five,
}

impl ClusterSize {
    pub async fn members(&self) -> Vec<Node> {
        match self {
            ClusterSize::One => Vec::with_capacity(1),
            ClusterSize::Three => Vec::with_capacity(3),
            ClusterSize::Five => Vec::with_capacity(5),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Membership {
    cluster_size: ClusterSize,
    server: Node,
    members: Vec<Node>,
    receive_action: Sender<MembershipAction>,
    send_grpc_action: Sender<MembershipAction>,
    send_server_action: Sender<MembershipAction>,
}

impl Membership {
    pub async fn init(
        cluster_size: ClusterSize,
        server: Node,
        receive_action: Sender<MembershipAction>,
        send_grpc_action: Sender<MembershipAction>,
        send_server_action: Sender<MembershipAction>,
    ) -> Result<Membership, Box<dyn std::error::Error>> {
        let members = cluster_size.members().await;

        Ok(Membership {
            cluster_size,
            server,
            members,
            send_grpc_action,
            receive_action,
            send_server_action,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut receiver = self.receive_action.subscribe();

        while let Ok(action) = receiver.recv().await {
            match action {
                MembershipAction::JoinClusterRequest(request) => {
                    println!("received join request");

                    let response = self.action_join_cluster_request(request).await?;

                    self.send_grpc_action
                        .send(MembershipAction::JoinClusterResponse(response))?;
                }
                MembershipAction::JoinClusterResponse(_) => println!("received join response!"),
                MembershipAction::NodeRequest => {
                    println!("received node request!");

                    let node = self.server;

                    self.send_server_action
                        .send(MembershipAction::NodeResponse(node))?;
                }
                MembershipAction::NodeResponse(_) => println!("received node response!"),
                MembershipAction::MembersRequest => {
                    println!("received members request!");

                    let members = &self.members;

                    self.send_server_action
                        .send(MembershipAction::MembersResponse(members.to_vec()))?;
                }
                MembershipAction::MembersResponse(_) => println!("received members response!"),
            }
        }

        Ok(())
    }

    async fn action_join_cluster_request(
        &mut self,
        request: JoinClusterRequest,
    ) -> Result<JoinClusterResponse, Box<dyn std::error::Error>> {
        let peer = Self::build_node(request).await?;

        self.members.push(peer);

        let response = JoinClusterResponse {
            success: String::from("true"),
            details: String::from("node successfully joined cluster!"),
            members: Vec::with_capacity(0),
        };

        Ok(response)
    }

    async fn build_node(request: JoinClusterRequest) -> Result<Node, Box<dyn std::error::Error>> {
        let id = Uuid::from_str(&request.id)?;
        let address = IpAddr::from_str(&request.address)?;
        let client_port = u16::from_str(&request.client_port)?;
        let cluster_port = u16::from_str(&request.cluster_port)?;
        let membership_port = u16::from_str(&request.membership_port)?;

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
