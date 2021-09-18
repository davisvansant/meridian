use tokio::sync::broadcast::Sender;

use crate::node::Node;
use crate::{JoinClusterRequest, JoinClusterResponse};

#[derive(Debug, PartialEq)]
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

pub struct Membership {
    cluster_size: ClusterSize,
    members: Vec<Node>,
    grpc_server_send_actions: Sender<JoinClusterResponse>,
    grpc_server_receive_actions: Sender<JoinClusterRequest>,
}

impl Membership {
    pub async fn init(
        cluster_size: ClusterSize,
        grpc_server_send_actions: Sender<JoinClusterResponse>,
        grpc_server_receive_actions: Sender<JoinClusterRequest>,
    ) -> Result<Membership, Box<dyn std::error::Error>> {
        let members = cluster_size.members().await;

        Ok(Membership {
            cluster_size,
            members,
            grpc_server_send_actions,
            grpc_server_receive_actions,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut receiver = self.grpc_server_receive_actions.subscribe();

        while let Ok(action) = receiver.recv().await {
            println!("incoming action - {:?}", &action);

            let response = JoinClusterResponse {
                success: String::from("true"),
                details: String::from("node successfully joined cluster!"),
                members: Vec::with_capacity(0),
            };

            if let Err(error) = self.grpc_server_send_actions.send(response) {
                println!("{:?}", error);
            }
        }

        Ok(())
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
