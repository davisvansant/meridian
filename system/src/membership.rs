use tokio::sync::broadcast::Sender;

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
    grpc_server_send_actions: Sender<JoinClusterResponse>,
    grpc_server_receive_actions: Sender<JoinClusterRequest>,
    server_send_action: Sender<MembershipAction>,
    server_receive_action: Sender<u8>,
}

impl Membership {
    pub async fn init(
        cluster_size: ClusterSize,
        server: Node,
        grpc_server_send_actions: Sender<JoinClusterResponse>,
        grpc_server_receive_actions: Sender<JoinClusterRequest>,
        server_send_action: Sender<MembershipAction>,
        server_receive_action: Sender<u8>,
    ) -> Result<Membership, Box<dyn std::error::Error>> {
        let members = cluster_size.members().await;

        Ok(Membership {
            cluster_size,
            server,
            members,
            grpc_server_send_actions,
            grpc_server_receive_actions,
            server_send_action,
            server_receive_action,
        })
    }

    pub async fn add_node(&mut self, node: Node) -> Result<(), Box<dyn std::error::Error>> {
        self.members.push(node);

        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut grpc_server_receiver = self.grpc_server_receive_actions.subscribe();
        let grpc_server_sender = self.grpc_server_send_actions.clone();

        let mut server_receiver = self.server_receive_action.subscribe();
        let server_sender = self.server_send_action.clone();

        let node = self.server.to_owned();
        let members = self.members.to_owned();

        let grpc_server = tokio::spawn(async move {
            println!("awaiting membership grpc server actions!");
            while let Ok(action) = grpc_server_receiver.recv().await {
                println!("incoming action - {:?}", &action);

                let response = JoinClusterResponse {
                    success: String::from("true"),
                    details: String::from("node successfully joined cluster!"),
                    members: Vec::with_capacity(0),
                };

                if let Err(error) = grpc_server_sender.send(response) {
                    println!("{:?}", error);
                }
            }
        });

        let server = tokio::spawn(async move {
            while let Ok(action) = server_receiver.recv().await {
                println!("receive action from server {:?}", &action);

                match action {
                    1 => {
                        if let Err(error) = server_sender.send(MembershipAction::Node(node)) {
                            println!("error sending! {:?}", error);
                        }
                    }
                    2 => {
                        if let Err(error) =
                            server_sender.send(MembershipAction::Members(members.to_owned()))
                        {
                            println!("error sending ! {:?}", error);
                        }
                    }
                    _ => panic!("received unsupported action!"),
                }
            }
        });

        tokio::try_join!(grpc_server, server)?;

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
