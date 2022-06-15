use flexbuffers::{Builder, BuilderOptions, Pushable};

use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

// use crate::channel::server::{Leader, LeaderSender};
// use crate::channel::shutdown::ShutdownReceiver;
use crate::channel::state::StateSender;
use crate::channel::state::{append_entries, request_vote};
use crate::channel::transition::ShutdownReceiver;
use crate::rpc::build_tcp_socket;
use crate::rpc::Data;
use crate::rpc::{AppendEntriesArguments, RequestVoteArguments};
use crate::{error, info, warn};

pub struct Server {
    socket_address: SocketAddr,
    state_sender: StateSender,
    // heartbeat: LeaderSender,
    leader_heartbeat: crate::channel::server::LeaderSender,
    shutdown: ShutdownReceiver,
}

impl Server {
    pub async fn init(
        state_sender: StateSender,
        // heartbeat: LeaderSender,
        leader_heartbeat: crate::channel::server::LeaderSender,
        socket_address: SocketAddr,
        shutdown: ShutdownReceiver,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        info!("initialized!");

        Ok(Server {
            socket_address,
            state_sender,
            // heartbeat,
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

                    let state_sender = self.state_sender.to_owned();
                    let heartbeat = self.leader_heartbeat.to_owned();

                    tokio::spawn(async move {
                        let mut buffer = [0; 1024];

                        match tcp_stream.read(&mut buffer).await {
                            Ok(data_length) => {
                                let send_result = match Self::route_incoming(
                                &buffer[0..data_length],
                                &state_sender,
                                &heartbeat,
                            )
                            .await
                        {
                            Ok(send_result) => send_result,
                            Err(error) => {
                                error!("error routing request ! {:?}", error);
                                return;
                            }
                        };

                        if let Err(error) = tcp_stream.write_all(&send_result).await {
                            error!("tcp stream write error ! {:?}", error);
                        };

                        if let Err(error) = tcp_stream.shutdown().await {
                            error!("tcp stream shutdown error! {:?}", error);
                        };
                        }
                            Err(error) => {
                                error!("{:?}", error);
                            }
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

    async fn route_incoming(
        data: &[u8],
        state_sender: &StateSender,
        // heartbeat: &LeaderSender,
        heartbeat: &crate::channel::server::LeaderSender,
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

                    // heartbeat.send(Leader::Heartbeat)?;
                    heartbeat.send(crate::channel::server::Leader::Heartbeat)?;
                }

                let arguments = AppendEntriesArguments {
                    term,
                    leader_id,
                    prev_log_index,
                    prev_log_term,
                    entries,
                    leader_commit,
                };

                let results = append_entries(state_sender, arguments).await?;
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

                let results = request_vote(state_sender, arguments).await?;
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
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, _test_state_receiver) = crate::channel::state::build().await;
        // let test_leader_sender = crate::channel::server::build_leader_heartbeat().await;
        let test_leader_sender = crate::channel::server::Leader::build().await;
        let test_socket_address = SocketAddr::from_str("0.0.0.0:1245")?;
        let test_shutdown_sender = crate::channel::transition::Shutdown::build().await;
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
        assert!(!test_server.state_sender.is_closed());

        Ok(())
    }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn run_communications_append_entries() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_communications_server = Server::init(Interface::Communications).await?;
    //     let test_handle = tokio::spawn(async move {
    //         if let Err(error) = test_communications_server.run().await {
    //             println!("{:?}", error);
    //         }
    //     });

    //     tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    //     let test_communications_client = Client::init(Interface::Communications).await?;
    //     let test_request = Data::AppendEntriesArguments.build().await?;
    //     let test_data = test_communications_client.transmit(&test_request).await?;

    //     let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

    //     test_data.push_to_builder(&mut test_flexbuffers_builder);

    //     let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;
    //     let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

    //     assert_eq!(
    //         test_flexbuffer_root.as_map().idx("data").as_str(),
    //         "append_entries_results",
    //     );
    //     assert_eq!(test_flexbuffers_root_details.idx("term").as_u8(), 0);
    //     assert!(!test_flexbuffers_root_details.idx("success").as_bool());
    //     assert!(test_handle.await.is_ok());

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn run_communications_request_vote() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_communications_server = Server::init(Interface::Communications).await?;
    //     let test_handle = tokio::spawn(async move {
    //         if let Err(error) = test_communications_server.run().await {
    //             println!("{:?}", error);
    //         }
    //     });

    //     tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

    //     let test_communications_client = Client::init(Interface::Communications).await?;
    //     let test_request = Data::RequestVoteArguments.build().await?;
    //     let test_data = test_communications_client.transmit(&test_request).await?;

    //     let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

    //     test_data.push_to_builder(&mut test_flexbuffers_builder);

    //     let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;
    //     let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

    //     assert_eq!(
    //         test_flexbuffer_root.as_map().idx("data").as_str(),
    //         "request_vote_results",
    //     );
    //     assert_eq!(test_flexbuffers_root_details.idx("term").as_u8(), 0);
    //     assert!(!test_flexbuffers_root_details.idx("vote_granted").as_bool());
    //     assert!(test_handle.await.is_ok());

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn run_membership() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_membership_server = Server::init(Interface::Membership).await?;
    //     let test_handle = tokio::spawn(async move {
    //         if let Err(error) = test_membership_server.run().await {
    //             println!("{:?}", error);
    //         }
    //     });
    //
    //     tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    //
    //     let test_membership_client = Client::init(Interface::Membership).await?;
    //     let test_data = test_membership_client
    //         .transmit(b"test_rpc_membership_interface")
    //         .await?;
    //
    //     assert_eq!(test_data.as_str(), "test_rpc_membership_interface");
    //     assert!(test_handle.await.is_ok());
    //
    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn route_incoming_append_entries() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_append_entries_arguments =
    //         crate::rpc::Data::AppendEntriesArguments.build().await?;
    //     let test_append_entries_results =
    //         Server::route_incoming(&test_append_entries_arguments).await?;

    //     let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

    //     test_append_entries_results.push_to_builder(&mut test_flexbuffers_builder);

    //     let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;
    //     let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

    //     assert_eq!(
    //         test_flexbuffer_root.as_map().idx("data").as_str(),
    //         "append_entries_results",
    //     );
    //     assert_eq!(test_flexbuffers_root_details.idx("term").as_u8(), 0);
    //     assert!(!test_flexbuffers_root_details.idx("success").as_bool());

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn route_incoming_request_vote() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_request_vote_arguments = crate::rpc::Data::RequestVoteArguments.build().await?;
    //     let test_request_vote_results =
    //         Server::route_incoming(&test_request_vote_arguments).await?;

    //     let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

    //     test_request_vote_results.push_to_builder(&mut test_flexbuffers_builder);

    //     let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;
    //     let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

    //     assert_eq!(
    //         test_flexbuffer_root.as_map().idx("data").as_str(),
    //         "request_vote_results",
    //     );
    //     assert_eq!(test_flexbuffers_root_details.idx("term").as_u8(), 0);
    //     assert!(!test_flexbuffers_root_details.idx("vote_granted").as_bool());

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn route_incoming_unknown() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_unknown = crate::rpc::Data::RequestVoteResults.build().await?;
    //     let test_data = Server::route_incoming(&test_unknown).await?;

    //     assert_eq!(test_data, "unknown".as_bytes());

    //     Ok(())
    // }
}
