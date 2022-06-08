use flexbuffers::{Builder, BuilderOptions, Pushable};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpSocket;

use crate::channel::membership::MembershipSender;
use crate::channel::membership::{cluster_members, node};
use crate::channel::rpc_client::{RpcClientReceiver, RpcClientRequest};
use crate::channel::server::{CandidateSender, CandidateTransition};
use crate::channel::state::StateSender;
use crate::channel::state::{candidate, heartbeat};
use crate::info;
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
        while let Some(request) = self.receiver.recv().await {
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

                            info!("results -> {:?}", &result);

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
                }
                RpcClientRequest::SendHeartbeat => {
                    info!("sending heartbeat");

                    let cluster_member = cluster_members(&self.membership_sender).await?;

                    for follower in cluster_member {
                        let socket_address = follower.build_address(follower.cluster_port).await;
                        self.send_heartbeat(socket_address).await?;
                    }
                }
                RpcClientRequest::Shutdown => {
                    info!("shutting down client...");

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
        // let candidate_id = get_node(&self.membership_sender).await?;
        let node = node(&self.membership_sender).await?;
        // let request_vote_arguments =
        //     candidate(&self.state_sender, candidate_id.id.to_string()).await?;
        let request_vote_arguments = candidate(&self.state_sender, node.id.to_string()).await?;
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
        // let leader = get_node(&self.membership_sender).await?;
        let node = node(&self.membership_sender).await?;
        // let heartbeat = heartbeat(&self.state_sender, leader.id.to_string()).await?;
        let heartbeat = heartbeat(&self.state_sender, node.id.to_string()).await?;
        let data = Data::AppendEntriesArguments(heartbeat).build().await?;

        Client::transmit(socket_address, &data).await?;

        Ok(())
    }

    async fn transmit(
        socket_address: SocketAddr,
        data: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];

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

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_client_sender, test_client_receiver) = crate::channel::rpc_client::build().await;
        let (test_membership_sender, _test_membership_receiver) =
            crate::channel::membership::build().await;
        let (test_state_sender, _test_state_receiver) = crate::channel::state::build().await;
        let test_candidate_sender = crate::channel::server::build_candidate_transition().await;
        let _test_candidate_receiver = test_candidate_sender.subscribe();

        let test_client = Client::init(
            test_client_receiver,
            test_membership_sender,
            test_state_sender,
            test_candidate_sender,
        )
        .await?;

        assert!(!test_client_sender.is_closed());
        assert!(!test_client.membership_sender.is_closed());
        assert!(!test_client.state_sender.is_closed());

        Ok(())
    }
}
