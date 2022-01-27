use flexbuffers::{Builder, BuilderOptions, Pushable};

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use uuid::Uuid;

use crate::channel::MembershipSender;
use crate::channel::StateSender;
use crate::channel::{add_member, candidate, cluster_members, get_node, heartbeat};
use crate::channel::{CandidateSender, CandidateTransition};
use crate::channel::{ClientReceiver, ClientRequest, ClientResponse};
use crate::channel::{ServerSender, ServerState};

use crate::rpc::{build_ip_address, build_socket_address};
// use crate::rpc::{Data, Interface, Node, RequestVoteResults};
use crate::rpc::{Data, Node, RequestVoteResults};

pub struct Client {
    // ip_address: IpAddr,
    // port: u16,
    socket_address: SocketAddr,
    receiver: ClientReceiver,
    membership_sender: MembershipSender,
    state_sender: StateSender,
    state_transition: ServerSender,
    candidate_sender: CandidateSender,
}

impl Client {
    pub async fn init(
        // interface: Interface,
        receiver: ClientReceiver,
        membership_sender: MembershipSender,
        state_sender: StateSender,
        state_transition: ServerSender,
        candidate_sender: CandidateSender,
    ) -> Result<Client, Box<dyn std::error::Error>> {
        let ip_address = build_ip_address().await;
        // let port = match interface {
        //     Interface::Communications => 1245,
        //     Interface::Membership => 1246,
        // };
        let port = 1245;

        let socket_address = build_socket_address(ip_address, port).await;

        Ok(Client {
            // ip_address,
            // port,
            socket_address,
            receiver,
            membership_sender,
            state_sender,
            state_transition,
            candidate_sender,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                ClientRequest::JoinCluster(address) => {
                    println!("joining cluster node at {:?}", &address);

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

                            println!("results -> {:?}", &result);

                            if result.vote_granted {
                                vote.push(1);
                            }
                        }

                        if vote.len() == 2 {
                            self.candidate_sender
                                .send(CandidateTransition::Leader)
                                .await?;
                        } else {
                            self.candidate_sender
                                .send(CandidateTransition::Follower)
                                .await?;
                        }
                    }

                    if let Err(error) = response.send(ClientResponse::EndElection(())) {
                        println!(
                            "error sending client start election response -> {:?}",
                            error,
                        );
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
    // use crate::rpc::Server;
    use crate::channel::CandidateTransition;
    use crate::channel::Leader;
    use crate::channel::ServerState;
    use crate::channel::{ClientRequest, ClientResponse};
    use crate::channel::{MembershipRequest, MembershipResponse};
    use crate::channel::{StateRequest, StateResponse};
    use tokio::sync::{broadcast, mpsc, oneshot};

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_client_sender, test_client_receiver) =
            mpsc::channel::<(ClientRequest, oneshot::Sender<ClientResponse>)>(64);
        let (test_membership_sender, _test_membership_receiver) =
            mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);
        let (test_state_sender, _test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);
        let (test_server_transition_sender, _test_server_transition_receiver) =
            broadcast::channel::<ServerState>(64);
        let (test_candidate_sender, _test_candidate_receiver) =
            mpsc::channel::<CandidateTransition>(64);

        let test_client = Client::init(
            test_client_receiver,
            test_membership_sender,
            test_state_sender,
            test_server_transition_sender,
            test_candidate_sender,
        )
        .await?;

        assert_eq!(
            test_client.socket_address.ip().to_string().as_str(),
            "127.0.0.1",
        );
        assert_eq!(test_client.socket_address.port(), 1245);
        assert!(!test_client_sender.is_closed());
        assert!(!test_client.membership_sender.is_closed());
        assert!(!test_client.state_sender.is_closed());
        assert_eq!(test_client.state_transition.receiver_count(), 1);
        assert!(!test_client.candidate_sender.is_closed());

        Ok(())
    }
}
