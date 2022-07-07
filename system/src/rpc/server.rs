use flexbuffers::{Builder, BuilderOptions, Pushable};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::channel::server::{Leader, LeaderHeartbeatSender};
use crate::channel::state::StateChannel;
use crate::channel::transition::ShutdownReceiver;
use crate::rpc::build_tcp_socket;
use crate::rpc::{AppendEntriesArguments, Data, RequestVoteArguments};
use crate::{error, info, warn};

pub struct Server {
    socket_address: SocketAddr,
    state: StateChannel,
    leader_heartbeat: LeaderHeartbeatSender,
    shutdown: ShutdownReceiver,
}

impl Server {
    pub async fn init(
        state: StateChannel,
        leader_heartbeat: LeaderHeartbeatSender,
        socket_address: SocketAddr,
        shutdown: ShutdownReceiver,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        info!("initialized!");

        Ok(Server {
            socket_address,
            state,
            leader_heartbeat,
            shutdown,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("running!");

        let tcp_socket = build_tcp_socket().await?;

        tcp_socket.set_reuseaddr(true)?;
        tcp_socket.set_reuseport(true)?;
        tcp_socket.bind(self.socket_address)?;

        let backlog = 1024;
        let tcp_listener = tcp_socket.listen(backlog)?;

        loop {
            tokio::select! {
                biased;
                 _ = self.shutdown.recv() => {
                    info!("shutting down...");

                    break
                }

                Ok((mut tcp_stream, socket_address)) = tcp_listener.accept() => {
                    info!("processing incoming connection...");
                    info!("stream -> {:?}", &tcp_stream);
                    info!("socket address -> {:?}", &socket_address);

                    let state = self.state.to_owned();
                    let heartbeat = self.leader_heartbeat.to_owned();

                    tokio::spawn(async move {
                        if let Err(error) = Self::process_tcp_stream(&state, &heartbeat, &mut tcp_stream).await {
                            error!("tcp stream error -> {:?}", error);
                        }
                    });
                }
                Err(error) = tcp_listener.accept() => {
                    error!("error with tcp listener -> {:?}", error);
                }
            }
        }

        Ok(())
    }

    async fn process_tcp_stream(
        state: &StateChannel,
        heartbeat: &LeaderHeartbeatSender,
        tcp_stream: &mut TcpStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];

        let rpc_request_bytes = tcp_stream.read(&mut buffer).await?;
        let rpc_response_bytes =
            Self::process_rpc_request(&buffer[0..rpc_request_bytes], state, heartbeat).await?;

        tcp_stream.write_all(&rpc_response_bytes).await?;
        tcp_stream.shutdown().await?;

        Ok(())
    }

    async fn process_rpc_request(
        data: &[u8],
        state: &StateChannel,
        heartbeat: &LeaderHeartbeatSender,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        data.push_to_builder(&mut flexbuffers_builder);

        let flexbuffers_root = flexbuffers::Reader::get_root(flexbuffers_builder.view())?;

        match flexbuffers_root.as_map().idx("data").as_str() {
            "append_entries_arguments" => {
                info!("received append entries arguments!");

                let request_details = flexbuffers_root.as_map().idx("details").as_map();
                let term = request_details.idx("term").as_u32();
                let leader_id = request_details.idx("leader_id").as_str().to_string();
                let prev_log_index = request_details.idx("prev_log_index").as_u32();
                let prev_log_term = request_details.idx("prev_log_term").as_u32();

                let mut entries =
                    Vec::with_capacity(request_details.idx("entries").as_vector().len());

                for entry in request_details.idx("entries").as_vector().iter() {
                    entries.push(entry.as_u32());
                }

                let leader_commit = request_details.idx("leader_commit").as_u32();

                if entries.is_empty() {
                    info!("sending heartbeat from server ...");

                    heartbeat.send(Leader::Heartbeat)?;
                }

                let arguments = AppendEntriesArguments {
                    term,
                    leader_id,
                    prev_log_index,
                    prev_log_term,
                    entries,
                    leader_commit,
                };

                let results = state.append_entries_arguments(arguments).await?;

                if term > results.term && heartbeat.receiver_count() > 0 {
                    info!("request term is higher than current term!");

                    heartbeat.send(Leader::Heartbeat)?;
                }

                let append_entries_results = Data::AppendEntriesResults(results).build().await?;

                Ok(append_entries_results)
            }
            "request_vote_arguments" => {
                info!("received request vote arguments!");

                let request_details = flexbuffers_root.as_map().idx("details").as_map();
                let term = request_details.idx("term").as_u32();
                let candidate_id = request_details.idx("candidate_id").as_str().to_string();
                let last_log_index = request_details.idx("last_log_index").as_u32();
                let last_log_term = request_details.idx("last_log_term").as_u32();

                let arguments = RequestVoteArguments {
                    term,
                    candidate_id,
                    last_log_index,
                    last_log_term,
                };

                let results = state.request_vote_arguments(arguments).await?;

                if term > results.term && heartbeat.receiver_count() > 0 {
                    info!("request term is higher than current term!");

                    heartbeat.send(Leader::Heartbeat)?;
                }

                let request_vote_results = Data::RequestVoteResults(results).build().await?;

                Ok(request_vote_results)
            }
            _ => {
                warn!("currently unknown ...");

                Ok(String::from("unknown").as_bytes().to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::transition::Shutdown;
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, _test_state_receiver) = StateChannel::init().await;
        let test_leader_sender = Leader::build().await;
        let test_socket_address = SocketAddr::from_str("0.0.0.0:1245")?;
        let test_shutdown_sender = Shutdown::build().await;
        let test_shutdown_receiver = test_shutdown_sender.subscribe();

        drop(test_shutdown_sender);

        let test_server = Server::init(
            test_state_sender,
            test_leader_sender,
            test_socket_address,
            test_shutdown_receiver,
        )
        .await?;

        assert_eq!(
            test_server.socket_address.ip().to_string().as_str(),
            "0.0.0.0",
        );
        assert_eq!(test_server.socket_address.port(), 1245);

        Ok(())
    }
}
