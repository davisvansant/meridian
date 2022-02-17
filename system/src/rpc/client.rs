use flexbuffers::{Builder, BuilderOptions, Pushable};

// use std::net::{IpAddr, SocketAddr};
use std::net::SocketAddr;
// use std::str::FromStr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::{TcpSocket, TcpStream};
use tokio::net::TcpSocket;

// use uuid::Uuid;

use crate::channel::MembershipSender;
use crate::channel::StateSender;
use crate::channel::{candidate, cluster_members, get_node, heartbeat};
use crate::channel::{CandidateSender, CandidateTransition};
use crate::channel::{RpcClientReceiver, RpcClientRequest, RpcClientResponse};
// use crate::rpc::{Data, Node, RequestVoteResults};
use crate::rpc::{Data, RequestVoteResults};

pub struct Client {
    receiver: RpcClientReceiver,
    membership_sender: MembershipSender,
    state_sender: StateSender,
    candidate_sender: CandidateSender,
}

impl Client {
    pub async fn init(
        receiver: RpcClientReceiver,
        membership_sender: MembershipSender,
        state_sender: StateSender,
        candidate_sender: CandidateSender,
    ) -> Result<Client, Box<dyn std::error::Error>> {
        Ok(Client {
            receiver,
            membership_sender,
            state_sender,
            candidate_sender,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                RpcClientRequest::StartElection => {
                    let mut vote = Vec::with_capacity(2);

                    let peers = cluster_members(&self.membership_sender).await?;

                    if peers.is_empty() {
                        self.candidate_sender.send(CandidateTransition::Leader)?;
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
                            self.candidate_sender.send(CandidateTransition::Leader)?;
                        } else {
                            self.candidate_sender.send(CandidateTransition::Follower)?;
                        }
                    }

                    if let Err(error) = response.send(RpcClientResponse::EndElection(())) {
                        println!(
                            "error sending client start election response -> {:?}",
                            error,
                        );
                    }
                }
                RpcClientRequest::SendHeartbeat => {
                    println!("sending heartbeat");

                    let cluster_member = cluster_members(&self.membership_sender).await?;

                    for follower in cluster_member {
                        let socket_address = follower.build_address(follower.cluster_port).await;
                        self.send_heartbeat(socket_address).await?;
                    }
                }
                RpcClientRequest::Shutdown => {
                    println!("shutting down client...");

                    self.receiver.close();
                }
            }
        }

        Ok(())
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

    async fn transmit(
        socket_address: SocketAddr,
        data: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];
        // let mut tcp_stream = TcpStream::connect(socket_address).await?;
        let tcp_socket = TcpSocket::new_v4()?;

        tcp_socket.set_reuseaddr(true)?;
        tcp_socket.set_reuseport(true)?;

        let mut tcp_stream = tcp_socket.connect(socket_address).await?;

        tcp_stream.write_all(data).await?;
        tcp_stream.shutdown().await?;

        let received_data = tcp_stream.read(&mut buffer).await?;

        Ok(buffer[0..received_data].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::CandidateTransition;
    use crate::channel::{MembershipRequest, MembershipResponse};
    use crate::channel::{RpcClientRequest, RpcClientResponse};
    use crate::channel::{StateRequest, StateResponse};
    use tokio::sync::{broadcast, mpsc, oneshot};

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_client_sender, test_client_receiver) =
            mpsc::channel::<(RpcClientRequest, oneshot::Sender<RpcClientResponse>)>(64);
        let (test_membership_sender, _test_membership_receiver) =
            mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);
        let (test_state_sender, _test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);
        let (test_candidate_sender, _test_candidate_receiver) =
            broadcast::channel::<CandidateTransition>(64);

        let test_client = Client::init(
            test_client_receiver,
            test_membership_sender,
            test_state_sender,
            test_candidate_sender,
        )
        .await?;

        // assert_eq!(
        //     test_client.socket_address.ip().to_string().as_str(),
        //     "127.0.0.1",
        // );
        // assert_eq!(test_client.socket_address.port(), 1245);
        assert!(!test_client_sender.is_closed());
        assert!(!test_client.membership_sender.is_closed());
        assert!(!test_client.state_sender.is_closed());

        Ok(())
    }
}
