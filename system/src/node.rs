use std::net::{IpAddr, SocketAddr};
use tokio::sync::broadcast::{channel, Sender};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use crate::external_client_grpc_server::{
    CommunicationsServer as ClientServer, ExternalClientGrpcServer,
};
use crate::external_membership_grpc_server::{
    CommunicationsServer as MembershipServer, ExternalMembershipGrpcServer,
};
use crate::internal_cluster_grpc_server::{
    CommunicationsServer as ClusterServer, InternalClusterGrpcServer,
};
use crate::membership::{ClusterSize, Membership};
use crate::meridian_membership_v010::{JoinClusterRequest, JoinClusterResponse};
use crate::server::Server;
use crate::state::State;
use crate::Actions;

#[derive(Clone)]
pub struct Node {
    id: Uuid,
    address: IpAddr,
    client_port: u16,
    cluster_port: u16,
    membership_port: u16,
}

impl Node {
    pub async fn init(
        address: IpAddr,
        client_port: u16,
        cluster_port: u16,
        membership_port: u16,
    ) -> Result<Node, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4();

        Ok(Node {
            id,
            address,
            client_port,
            cluster_port,
            membership_port,
        })
    }

    pub async fn run(&self, cluster_size: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cluster_size = match cluster_size {
            "1" => ClusterSize::One,
            "3" => ClusterSize::Three,
            "5" => ClusterSize::Five,
            _ => panic!("Expected a cluster size of 1, 3, or 5"),
        };

        let (state_send_handle, _) = channel(64);
        let grpc_send_actions = state_send_handle.clone();
        let server_send_actions = state_send_handle.clone();

        let (state_receive_server_actions, _) = channel(64);
        let (state_receive_grpc_actions, _) = channel(64);
        let state_send_server_actions = state_receive_server_actions.clone();
        let state_send_grpc_actions = state_receive_grpc_actions.clone();

        let (grpc_send_membership_actions, _) = channel(64);
        let (membership_send_grpc_actions, _) = channel(64);
        let membership_receive_grpc_actions = grpc_send_membership_actions.clone();
        let grpc_receive_membership_actions = membership_send_grpc_actions.clone();

        let client_handle = self.run_client_grpc_server().await?;
        let cluster_handle = self
            .run_cluster_grpc_server(state_receive_grpc_actions, grpc_send_actions)
            .await?;
        let membership_grpc_handle = self
            .run_membership_grpc_server(
                grpc_receive_membership_actions,
                grpc_send_membership_actions,
            )
            .await?;
        let server_handle = self
            .run_server(state_receive_server_actions, server_send_actions)
            .await?;
        let membership_run_handle = self
            .run_membership(
                cluster_size,
                membership_send_grpc_actions,
                membership_receive_grpc_actions,
            )
            .await?;
        let state_handle = self
            .run_state(
                state_send_handle,
                state_send_server_actions,
                state_send_grpc_actions,
            )
            .await?;

        tokio::try_join!(
            cluster_handle,
            client_handle,
            server_handle,
            state_handle,
            membership_grpc_handle,
            membership_run_handle,
        )?;

        Ok(())
    }

    async fn run_client_grpc_server(&self) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let external_client_grpc_server = ExternalClientGrpcServer::init().await?;
        let client_service = ClientServer::new(external_client_grpc_server);
        let client_grpc_server = tonic::transport::Server::builder()
            .add_service(client_service)
            .serve(self.build_address(self.client_port).await);

        let client_handle = tokio::spawn(async move {
            println!("starting up client server");
            if let Err(error) = client_grpc_server.await {
                println!(
                    "something went wrong with the client grpc interface - {:?}",
                    error,
                );
            }
        });

        Ok(client_handle)
    }

    async fn run_cluster_grpc_server(
        &self,
        state_receive_grpc_actions: Sender<Actions>,
        grpc_send_actions: Sender<Actions>,
    ) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let internal_cluster_grpc_server =
            InternalClusterGrpcServer::init(state_receive_grpc_actions, grpc_send_actions).await?;

        let cluster_service = ClusterServer::new(internal_cluster_grpc_server);
        let cluster_grpc_server = tonic::transport::Server::builder()
            .add_service(cluster_service)
            .serve(self.build_address(self.cluster_port).await);

        let cluster_handle = tokio::spawn(async move {
            println!("starting up server");
            if let Err(error) = cluster_grpc_server.await {
                println!(
                    "something went wrong with the internal grpc interface - {:?}",
                    error,
                );
            }
        });

        Ok(cluster_handle)
    }

    async fn run_membership_grpc_server(
        &self,
        grpc_receive_membership_actions: Sender<JoinClusterResponse>,
        grpc_send_membership_actions: Sender<JoinClusterRequest>,
    ) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let external_membership_grpc_server = ExternalMembershipGrpcServer::init(
            grpc_receive_membership_actions,
            grpc_send_membership_actions,
        )
        .await?;

        let membership_service = MembershipServer::new(external_membership_grpc_server);
        let membership_grpc_server = tonic::transport::Server::builder()
            .add_service(membership_service)
            .serve(self.build_address(self.membership_port).await);

        let membership_grpc_handle = tokio::spawn(async move {
            println!("starting up membership...");
            if let Err(error) = membership_grpc_server.await {
                println!(
                    "something went wrong with the internal membership - {:?}",
                    error,
                );
            }
        });

        Ok(membership_grpc_handle)
    }

    async fn run_membership(
        &self,
        cluster_size: ClusterSize,
        membership_send_grpc_actions: Sender<JoinClusterResponse>,
        membership_receive_grpc_actions: Sender<JoinClusterRequest>,
    ) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let mut membership = Membership::init(
            // ClusterSize::One,
            cluster_size,
            membership_send_grpc_actions,
            membership_receive_grpc_actions,
        )
        .await?;

        membership.add_node(self.to_owned()).await?;

        let membership_run_handle = tokio::spawn(async move {
            sleep(Duration::from_secs(10)).await;

            if let Err(error) = membership.run().await {
                println!("error with running {:?}", error);
            };
        });

        Ok(membership_run_handle)
    }

    async fn run_server(
        &self,
        state_receive_server_actions: Sender<Actions>,
        server_send_actions: Sender<Actions>,
    ) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let mut server = Server::init(state_receive_server_actions, server_send_actions).await?;

        let server_handle = tokio::spawn(async move {
            sleep(Duration::from_secs(10)).await;

            if let Err(error) = server.run().await {
                println!("error with running {:?}", error);
            };
        });

        Ok(server_handle)
    }

    async fn run_state(
        &self,
        state_send_handle: Sender<Actions>,
        state_send_server_actions: Sender<Actions>,
        state_send_grpc_actions: Sender<Actions>,
    ) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let mut state = State::init(
            state_send_handle,
            state_send_server_actions,
            state_send_grpc_actions,
        )
        .await?;

        let state_handle = tokio::spawn(async move {
            sleep(Duration::from_secs(5)).await;

            if let Err(error) = state.run().await {
                println!("state error! {:?}", error);
            }
        });

        Ok(state_handle)
    }

    async fn build_address(&self, client_port: u16) -> SocketAddr {
        SocketAddr::new(self.address, client_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_node_address = std::net::IpAddr::from_str("0.0.0.0")?;
        let test_node = Node::init(test_node_address, 10000, 15000, 20000).await?;
        assert_eq!(test_node.id.get_version_num(), 4);
        assert_eq!(test_node.address.to_string().as_str(), "0.0.0.0");
        assert_eq!(test_node.client_port, 10000);
        assert_eq!(test_node.cluster_port, 15000);
        assert_eq!(test_node.membership_port, 20000);
        Ok(())
    }
}
