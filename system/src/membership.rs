use std::net::IpAddr;
use std::str::FromStr;
use uuid::Uuid;

use crate::grpc::membership_client::ExternalMembershipGrpcClient;
use crate::node::Node;
use crate::{JoinClusterRequest, JoinClusterResponse};

use crate::runtime::sync::launch::ChannelLaunch;
use crate::runtime::sync::membership_receive_task::ChannelMembershipReceiveAction;
use crate::runtime::sync::membership_receive_task::MembershipReceiveAction;
use crate::runtime::sync::membership_send_grpc_task::ChannelMembershipSendGrpcAction;
use crate::runtime::sync::membership_send_grpc_task::MembershipSendGrpcAction;
use crate::runtime::sync::membership_send_server_task::ChannelMembershipSendServerAction;
use crate::runtime::sync::membership_send_server_task::MembershipSendServerAction;

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
    receive_action: ChannelMembershipReceiveAction,
    send_grpc_action: ChannelMembershipSendGrpcAction,
    send_server_action: ChannelMembershipSendServerAction,
}

impl Membership {
    pub async fn init(
        cluster_size: ClusterSize,
        server: Node,
        receive_action: ChannelMembershipReceiveAction,
        send_grpc_action: ChannelMembershipSendGrpcAction,
        send_server_action: ChannelMembershipSendServerAction,
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

    pub async fn run(
        &mut self,
        peers: Vec<String>,
        send_launch_action: ChannelLaunch,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut receiver = self.receive_action.subscribe();

        for member in peers {
            println!("{:?}", member);

            let request = JoinClusterRequest {
                id: self.server.id.to_string(),
                address: self.server.address.to_string(),
                client_port: self.server.client_port.to_string(),
                cluster_port: self.server.cluster_port.to_string(),
                membership_port: self.server.membership_port.to_string(),
            };

            let mut endpoint = String::with_capacity(20);

            endpoint.push_str("http://");
            endpoint.push_str(&member);
            endpoint.shrink_to_fit();

            println!("sending join request ... {:?}", &endpoint);

            let mut client = ExternalMembershipGrpcClient::init(endpoint).await?;
            let response = client.join_cluster(request).await?;

            println!("join response - {:?}", response);
        }

        if send_launch_action.send(()).is_ok() {
            println!("sending launch action");
        }

        while let Ok(action) = receiver.recv().await {
            match action {
                MembershipReceiveAction::JoinClusterRequest(request) => {
                    println!("received join request");

                    let response = self.action_join_cluster_request(request).await?;

                    self.send_grpc_action
                        .send(MembershipSendGrpcAction::JoinClusterResponse(response))?;
                }
                MembershipReceiveAction::Node => {
                    println!("received node request!");

                    let node = self.server;

                    self.send_server_action
                        .send(MembershipSendServerAction::NodeResponse(node))?;
                }
                MembershipReceiveAction::Members => {
                    println!("received members request!");

                    let members = &self.members;

                    self.send_server_action
                        .send(MembershipSendServerAction::MembersResponse(
                            members.to_vec(),
                        ))?;
                }
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

        let mut members = Vec::with_capacity(self.members.len());

        for member in &self.members {
            let address = member.address;
            let port = member.cluster_port;
            let mut node = String::with_capacity(15);

            node.push_str(&address.to_string());
            node.push(':');
            node.push_str(&port.to_string());
            node.shrink_to_fit();

            members.push(node)
        }

        let mut node = String::with_capacity(15);

        node.push_str(&self.server.address.to_string());
        node.push(':');
        node.push_str(&self.server.cluster_port.to_string());
        node.shrink_to_fit();

        println!("{:?}", &node);

        members.push(node);

        members.dedup();

        for m in &members {
            println!("peers !{:?}", m);
        }

        let response = JoinClusterResponse {
            success: String::from("true"),
            details: String::from("node successfully joined cluster!"),
            members,
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
