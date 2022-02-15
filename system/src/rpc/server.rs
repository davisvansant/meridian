use flexbuffers::{Builder, BuilderOptions, Pushable};

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal::unix::{signal, SignalKind};

use uuid::Uuid;

// use crate::channel::{add_member, append_entries, cluster_members, get_node, request_vote, status};
use crate::channel::{append_entries, cluster_members, get_node, request_vote, status};
use crate::channel::{Leader, LeaderSender};
use crate::channel::{MembershipSender, StateSender};
// use crate::rpc::{build_ip_address, build_tcp_socket};
use crate::rpc::build_tcp_socket;
use crate::rpc::{AppendEntriesArguments, RequestVoteArguments};
use crate::rpc::{Data, Node};

pub struct Server {
    // ip_address: IpAddr,
    // port: u16,
    socket_address: SocketAddr,
    membership_sender: MembershipSender,
    state_sender: StateSender,
    heartbeat: LeaderSender,
}

impl Server {
    pub async fn init(
        membership_sender: MembershipSender,
        state_sender: StateSender,
        heartbeat: LeaderSender,
        socket_address: SocketAddr,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        // let ip_address = build_ip_address().await;
        // let port = match interface {
        //     Interface::Communications => 1245,
        //     Interface::Membership => 1246,
        // };

        // let socket_address = build_socket_address(ip_address, port).await;

        Ok(Server {
            // ip_address,
            // port,
            socket_address,
            membership_sender,
            state_sender,
            heartbeat,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let tcp_socket = build_tcp_socket().await?;

        tcp_socket.set_reuseaddr(true)?;
        tcp_socket.set_reuseport(true)?;
        tcp_socket.bind(self.socket_address)?;

        let backlog = 1024;
        let tcp_listener = tcp_socket.listen(backlog)?;

        let mut interrupt = signal(SignalKind::interrupt())?;

        loop {
            tokio::select! {
                biased;
                _ = interrupt.recv() => {
                    println!("shutting down rpc server interface...");

                    break
                }

                Ok((mut tcp_stream, socket_address)) = tcp_listener.accept() => {
                    println!("processing incoming connection...");
                    println!("stream -> {:?}", &tcp_stream);
                    println!("socket address -> {:?}", &socket_address);

                    let membership_sender = self.membership_sender.to_owned();
                    let state_sender = self.state_sender.to_owned();
                    let heartbeat = self.heartbeat.to_owned();

                    tokio::spawn(async move {
                        let mut buffer = [0; 1024];

                        match tcp_stream.read(&mut buffer).await {
                            Ok(data_length) => {
                                let send_result = match Self::route_incoming(
                                &buffer[0..data_length],
                                &membership_sender,
                                &state_sender,
                                &heartbeat,
                            )
                            .await
                        {
                            Ok(send_result) => send_result,
                            Err(error) => {
                                println!("error routing request ! {:?}", error);
                                return;
                            }
                        };

                        if let Err(error) = tcp_stream.write_all(&send_result).await {
                            println!("tcp stream write error ! {:?}", error);
                        };

                        if let Err(error) = tcp_stream.shutdown().await {
                            println!("tcp stream shutdown error! {:?}", error);
                        };
                        }
                            Err(error) => {
                                println!("{:?}", error);
                            }
                        }
                    });
                }
                Err(error) = tcp_listener.accept() => {
                    println!("error with tcp listener -> {:?}", error);
                }
            }
        }

        Ok(())
    }

    async fn route_incoming(
        data: &[u8],
        membership_sender: &MembershipSender,
        state_sender: &StateSender,
        heartbeat: &LeaderSender,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        data.push_to_builder(&mut flexbuffers_builder);

        let flexbuffers_root = flexbuffers::Reader::get_root(flexbuffers_builder.view())?;

        match flexbuffers_root.as_map().idx("data").as_str() {
            "append_entries_arguments" => {
                println!("received append entries arguments!");

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
                    println!("sending heartbeat from server ...");

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

                let results = append_entries(state_sender, arguments).await?;
                let append_entries_results = Data::AppendEntriesResults(results).build().await?;

                Ok(append_entries_results)
            }
            // "connected" => {
            //     println!("received connected nodes request!");

            //     let connected_nodes = cluster_members(membership_sender).await?;
            //     let connected_response = Data::ConnectedResponse(connected_nodes).build().await?;

            //     Ok(connected_response)
            // }
            // "join_cluster_request" => {
            //     println!("received join cluster request!");

            //     let request_details = flexbuffers_root.as_map().idx("details").as_map();
            //     let id = Uuid::from_str(request_details.idx("id").as_str())?;
            //     let address = IpAddr::from_str(request_details.idx("address").as_str())?;
            //     let client_port = request_details.idx("client_port").as_u16();
            //     let cluster_port = request_details.idx("cluster_port").as_u16();
            //     let membership_port = request_details.idx("membership_port").as_u16();

            //     let candidate_node = Node {
            //         id,
            //         address,
            //         client_port,
            //         cluster_port,
            //         membership_port,
            //     };

            //     add_member(membership_sender, candidate_node).await?;

            //     let server_node = get_node(membership_sender).await?;
            //     let join_cluster_response = Data::JoinClusterRequest(server_node).build().await?;

            //     Ok(join_cluster_response)
            // }
            "request_vote_arguments" => {
                println!("received request vote arguments!");

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
            // "status" => {
            //     println!("received status request!");

            //     let results = status(membership_sender).await?;
            //     let status_response = Data::StatusResponse(results).build().await?;

            //     Ok(status_response)
            // }
            _ => {
                println!("currently unknown ...");

                Ok(String::from("unknown").as_bytes().to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::rpc::Client;
    use crate::channel::Leader;
    use crate::channel::{MembershipRequest, MembershipResponse};
    use crate::channel::{StateRequest, StateResponse};
    use tokio::sync::{broadcast, mpsc, oneshot};

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_membership_sender, _test_membership_receiver) =
            mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);
        let (test_state_sender, _test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);
        let (test_leader_sender, _test_leader_receiver) = broadcast::channel::<Leader>(64);
        let test_socket_address = SocketAddr::from_str("0.0.0.0:1245")?;

        let test_server = Server::init(
            test_membership_sender,
            test_state_sender,
            test_leader_sender,
            test_socket_address,
        )
        .await?;

        assert_eq!(
            test_server.socket_address.ip().to_string().as_str(),
            "0.0.0.0",
        );
        assert_eq!(test_server.socket_address.port(), 1245);
        assert!(!test_server.membership_sender.is_closed());
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
