use flexbuffers::{Builder, BuilderOptions, Pushable};

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use uuid::Uuid;

use crate::channel::MembershipSender;
use crate::channel::StateSender;
use crate::channel::{add_member, candidate, cluster_members, get_node, heartbeat};
use crate::channel::{ClientReceiver, ClientRequest, ClientResponse};
use crate::channel::{ServerSender, ServerState};

use crate::rpc::{build_ip_address, build_socket_address};
use crate::rpc::{Data, Interface, Node, RequestVoteResults};

pub struct Client {
    ip_address: IpAddr,
    port: u16,
    socket_address: SocketAddr,
    receiver: ClientReceiver,
    membership_sender: MembershipSender,
    state_sender: StateSender,
    state_transition: ServerSender,
}

impl Client {
    pub async fn init(
        interface: Interface,
        receiver: ClientReceiver,
        membership_sender: MembershipSender,
        state_sender: StateSender,
        state_transition: ServerSender,
    ) -> Result<Client, Box<dyn std::error::Error>> {
        let ip_address = build_ip_address().await;
        let port = match interface {
            Interface::Communications => 1245,
            Interface::Membership => 1246,
        };

        let socket_address = build_socket_address(ip_address, port).await;

        Ok(Client {
            ip_address,
            port,
            socket_address,
            receiver,
            membership_sender,
            state_sender,
            state_transition,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                ClientRequest::JoinCluster(address) => {
                    println!("received join cluster request!");

                    let joined_node = self.join_cluster(address).await?;
                    let socket_address = joined_node.build_address(joined_node.cluster_port).await;

                    if let Err(error) = response.send(ClientResponse::JoinCluster(socket_address)) {
                        println!("error sending client response -> {:?}", error);
                    }
                }
                ClientRequest::PeerNodes(socket_address) => {
                    println!("received peer nodes client request");

                    let connected_nodes = self.get_connected(socket_address).await?;

                    if let Err(error) = response.send(ClientResponse::Nodes(connected_nodes)) {
                        println!("error sending client peer nodes response -> {:?}", error);
                    }
                }
                ClientRequest::PeerStatus(socket_address) => {
                    println!("received get peer status");

                    let status = self.status(socket_address).await?;

                    if let Err(error) = response.send(ClientResponse::Status(status)) {
                        println!("error sending client peer status response -> {:?}", error);
                    }
                }
                ClientRequest::StartElection => {
                    let mut vote = Vec::with_capacity(2);

                    let peers = cluster_members(&self.membership_sender).await?;

                    if peers.is_empty() {
                        self.state_transition.send(ServerState::Leader)?;
                    } else {
                        for peer in peers {
                            let socket_address = peer.build_address(peer.cluster_port).await;
                            let result = self.request_vote(socket_address).await?;

                            if result.vote_granted {
                                vote.push(1);
                            }
                        }
                    }
                }
                ClientRequest::SendHeartbeat => {
                    println!("sending heartbeat");

                    let cluster_member = cluster_members(&self.membership_sender).await?;

                    for follower in cluster_member {
                        let socket_address = follower.build_address(follower.cluster_port).await;
                        self.send_heartbeat(socket_address).await?;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn join_cluster(
        &self,
        socket_address: SocketAddr,
    ) -> Result<Node, Box<dyn std::error::Error>> {
        let node = get_node(&self.membership_sender).await?;
        let request = Data::JoinClusterRequest(node).build().await?;
        let response = Client::transmit(socket_address, &request).await?;

        let mut flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        response.push_to_builder(&mut flexbuffers_builder);

        let flexbuffer_root = flexbuffers::Reader::get_root(flexbuffers_builder.view())?;

        let response_details = flexbuffer_root.as_map().idx("details").as_map();
        let id = Uuid::from_str(response_details.idx("id").as_str())?;
        let address = IpAddr::from_str(response_details.idx("address").as_str())?;
        let client_port = response_details.idx("client_port").as_u16();
        let cluster_port = response_details.idx("cluster_port").as_u16();
        let membership_port = response_details.idx("membership_port").as_u16();

        let joined_node = Node {
            id,
            address,
            client_port,
            cluster_port,
            membership_port,
        };

        add_member(&self.membership_sender, joined_node).await?;

        Ok(joined_node)
    }

    pub async fn get_connected(
        &self,
        socket_address: SocketAddr,
    ) -> Result<Vec<SocketAddr>, Box<dyn std::error::Error>> {
        let request = Data::ConnectedRequest.build().await?;
        let response = Client::transmit(socket_address, &request).await?;

        let root = flexbuffers::Reader::get_root(&*response)?;

        // let mut flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        // response.push_to_builder(&mut flexbuffers_builder);

        // let flexbuffer_root = flexbuffers::Reader::get_root(flexbuffers_builder.view())?;

        // let response_details = flexbuffer_root
        //     .as_map()
        //     .idx("details")
        //     .as_map()
        //     .idx("nodes")
        //     .as_vector();
        let response_details = root
            .as_map()
            .idx("details")
            .as_map()
            .idx("nodes")
            .as_vector();

        let mut connected_nodes = Vec::with_capacity(response_details.len());

        for node in response_details.iter() {
            let id = Uuid::from_str(node.as_map().idx("id").as_str())?;
            let address = IpAddr::from_str(node.as_map().idx("address").as_str())?;
            let client_port = node.as_map().idx("client_port").as_u16();
            let cluster_port = node.as_map().idx("cluster_port").as_u16();
            let membership_port = node.as_map().idx("membership_port").as_u16();

            let connected_node = Node {
                id,
                address,
                client_port,
                cluster_port,
                membership_port,
            };

            let socket_address = connected_node
                .build_address(connected_node.cluster_port)
                .await;

            // connected_nodes.push(connected_node)
            connected_nodes.push(socket_address)
        }

        Ok(connected_nodes)
    }

    pub async fn request_vote(
        &self,
        socket_address: SocketAddr,
    ) -> Result<RequestVoteResults, Box<dyn std::error::Error>> {
        let candidate_id = get_node(&self.membership_sender).await?;
        let request_vote_arguments =
            candidate(&self.state_sender, candidate_id.id.to_string()).await?;
        let data = Data::RequestVoteArguments(request_vote_arguments)
            .build()
            .await?;

        let response = Client::transmit(socket_address, &data).await?;

        let mut flexbuffer_builder = Builder::new(BuilderOptions::SHARE_NONE);

        response.push_to_builder(&mut flexbuffer_builder);

        let flexbuffer_root = flexbuffers::Reader::get_root(flexbuffer_builder.view())?;
        let flexbuffer_root_details = flexbuffer_root.as_map().idx("details").as_map();

        let request_vote_results = RequestVoteResults {
            term: flexbuffer_root_details.idx("term").as_u32(),
            vote_granted: flexbuffer_root_details.idx("vote_granted").as_bool(),
        };

        Ok(request_vote_results)
    }

    pub async fn send_heartbeat(
        &self,
        socket_address: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let leader = get_node(&self.membership_sender).await?;
        let heartbeat = heartbeat(&self.state_sender, leader.id.to_string()).await?;
        let data = Data::AppendEntriesArguments(heartbeat).build().await?;

        Client::transmit(socket_address, &data).await?;

        Ok(())
    }

    pub async fn status(
        &self,
        socket_address: SocketAddr,
    ) -> Result<u8, Box<dyn std::error::Error>> {
        let data = Data::StatusRequest.build().await?;
        let response = Client::transmit(socket_address, &data).await?;

        let mut flexbuffer_builder = Builder::new(BuilderOptions::SHARE_NONE);

        response.push_to_builder(&mut flexbuffer_builder);

        let flexbuffer_root = flexbuffers::Reader::get_root(flexbuffer_builder.view())?;
        let flexbuffer_root_details = flexbuffer_root.as_map().idx("details").as_map();
        let status = flexbuffer_root_details.idx("status").as_u8();

        Ok(status)
    }

    async fn transmit(
        socket_address: SocketAddr,
        data: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];
        let mut tcp_stream = TcpStream::connect(socket_address).await?;

        tcp_stream.write_all(data).await?;
        tcp_stream.shutdown().await?;

        let received_data = tcp_stream.read(&mut buffer).await?;

        Ok(buffer[0..received_data].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::Server;

    // #[tokio::test(flavor = "multi_thread")]
    // async fn init_communications() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_client_communications = Client::init(Interface::Communications).await?;

    //     assert_eq!(
    //         test_client_communications.ip_address.to_string().as_str(),
    //         "127.0.0.1",
    //     );
    //     assert_eq!(test_client_communications.port, 1245);
    //     assert_eq!(
    //         test_client_communications
    //             .socket_address
    //             .to_string()
    //             .as_str(),
    //         "127.0.0.1:1245",
    //     );

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn init_membership() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_client_membership = Client::init(Interface::Membership).await?;

    //     assert_eq!(
    //         test_client_membership.ip_address.to_string().as_str(),
    //         "127.0.0.1",
    //     );
    //     assert_eq!(test_client_membership.port, 1246);
    //     assert_eq!(
    //         test_client_membership.socket_address.to_string().as_str(),
    //         "127.0.0.1:1246",
    //     );

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn connect_communications() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_server_communications = Server::init(Interface::Communications).await?;
    //     let test_server_handle = tokio::spawn(async move {
    //         if let Err(error) = test_server_communications.run().await {
    //             println!("{:?}", error);
    //         }
    //     });
    //
    //     tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    //
    //     let test_client_communications = Client::init(Interface::Communications).await?;
    //     let test_data = test_client_communications
    //         .transmit(b"test_client_communications")
    //         .await?;
    //
    //     assert_eq!(test_data.as_str(), "test_client_communications");
    //     assert!(test_server_handle.await.is_ok());
    //
    //     Ok(())
    // }
    //
    // #[tokio::test(flavor = "multi_thread")]
    // async fn connect_membership() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_server_membership = Server::init(Interface::Membership).await?;
    //     let test_server_handle = tokio::spawn(async move {
    //         if let Err(error) = test_server_membership.run().await {
    //             println!("{:?}", error);
    //         }
    //     });
    //
    //     tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    //
    //     let test_client_membership = Client::init(Interface::Membership).await?;
    //     let test_data = test_client_membership
    //         .transmit(b"test_member_communications")
    //         .await?;
    //
    //     assert_eq!(test_data.as_str(), "test_member_communications");
    //     assert!(test_server_handle.await.is_ok());
    //
    //     Ok(())
    // }
}
