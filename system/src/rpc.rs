use flexbuffers::{Builder, BuilderOptions};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpSocket;

// use crate::node::Node;
use crate::rpc::append_entries::{AppendEntriesArguments, AppendEntriesResults};
use crate::rpc::request_vote::{RequestVoteArguments, RequestVoteResults};

pub use client::Client;
pub use server::Server;

mod client;
mod server;

pub mod append_entries;
pub mod install_snapshot;
// pub mod membership;
pub mod request_vote;

pub enum Data {
    AppendEntriesArguments(AppendEntriesArguments),
    AppendEntriesResults(AppendEntriesResults),
    _InstallSnapshot,
    RequestVoteArguments(RequestVoteArguments),
    RequestVoteResults(RequestVoteResults),
}

impl Data {
    pub async fn build(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let flexbuffer_options = BuilderOptions::SHARE_NONE;
        let mut flexbuffers_builder = Builder::new(flexbuffer_options);
        let mut flexbuffers_data = flexbuffers_builder.start_map();

        match self {
            Data::AppendEntriesArguments(append_entries_arguments) => {
                flexbuffers_data.push("data", "append_entries_arguments");

                let mut details = flexbuffers_data.start_map("details");

                details.push("term", append_entries_arguments.term);
                details.push("leader_id", append_entries_arguments.leader_id.as_str());
                details.push("prev_log_index", append_entries_arguments.prev_log_index);
                details.push("prev_log_term", append_entries_arguments.prev_log_term);
                details.push("entries", &append_entries_arguments.entries);
                details.push("leader_commit", append_entries_arguments.leader_commit);
                details.end_map();

                flexbuffers_data.end_map();

                Ok(flexbuffers_builder.take_buffer())
            }
            Data::AppendEntriesResults(append_entries_results) => {
                flexbuffers_data.push("data", "append_entries_results");

                let mut details = flexbuffers_data.start_map("details");

                details.push("term", append_entries_results.term);
                details.push("success", append_entries_results.success);
                details.end_map();

                flexbuffers_data.end_map();

                Ok(flexbuffers_builder.take_buffer())
            }
            Data::_InstallSnapshot => {
                unimplemented!();
            }
            Data::RequestVoteArguments(request_vote_arguments) => {
                flexbuffers_data.push("data", "request_vote_arguments");

                let mut details = flexbuffers_data.start_map("details");

                details.push("term", request_vote_arguments.term);
                details.push("candidate_id", request_vote_arguments.candidate_id.as_str());
                details.push("last_log_index", request_vote_arguments.last_log_index);
                details.push("last_log_term", request_vote_arguments.last_log_term);
                details.end_map();

                flexbuffers_data.end_map();

                Ok(flexbuffers_builder.take_buffer())
            }
            Data::RequestVoteResults(request_vote_results) => {
                flexbuffers_data.push("data", "request_vote_results");

                let mut details = flexbuffers_data.start_map("details");

                details.push("term", request_vote_results.term);
                details.push("vote_granted", request_vote_results.vote_granted);
                details.end_map();

                flexbuffers_data.end_map();

                Ok(flexbuffers_builder.take_buffer())
            }
        }
    }
}

// pub async fn build_ip_address() -> IpAddr {
//     IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
// }

// pub async fn build_socket_address(ip_address: IpAddr, port: u16) -> SocketAddr {
//     SocketAddr::new(ip_address, port)
// }

pub async fn build_tcp_socket() -> Result<TcpSocket, Box<dyn std::error::Error>> {
    let tcp_socket = TcpSocket::new_v4()?;

    Ok(tcp_socket)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flexbuffers::Pushable;
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread")]
    async fn data_append_entries_arguments() -> Result<(), Box<dyn std::error::Error>> {
        let test_append_entries_arguments = AppendEntriesArguments {
            term: 0,
            leader_id: String::from("some_leader_id"),
            prev_log_index: 0,
            prev_log_term: 0,
            entries: Vec::with_capacity(0),
            leader_commit: 0,
        };

        let test_append_entries_arguments_data =
            Data::AppendEntriesArguments(test_append_entries_arguments)
                .build()
                .await?;

        assert_eq!(test_append_entries_arguments_data.len(), 157);

        let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        test_append_entries_arguments_data.push_to_builder(&mut test_flexbuffers_builder);

        let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;
        let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

        assert!(test_flexbuffer_root.is_aligned());
        assert_eq!(test_flexbuffer_root.bitwidth().n_bytes(), 1);
        assert_eq!(test_flexbuffer_root.length(), 2);
        assert_eq!(
            test_flexbuffer_root.as_map().idx("data").as_str(),
            "append_entries_arguments",
        );
        assert_eq!(test_flexbuffers_root_details.idx("term").as_u8(), 0);
        assert_eq!(
            test_flexbuffers_root_details.idx("leader_id").as_str(),
            "some_leader_id",
        );
        assert_eq!(
            test_flexbuffers_root_details.idx("prev_log_index").as_u8(),
            0,
        );
        assert_eq!(
            test_flexbuffers_root_details.idx("prev_log_term").as_u8(),
            0,
        );
        assert_eq!(
            test_flexbuffers_root_details
                .idx("entries")
                .as_vector()
                .len(),
            0,
        );
        assert_eq!(
            test_flexbuffers_root_details.idx("leader_commit").as_u8(),
            0,
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn data_append_entries_results() -> Result<(), Box<dyn std::error::Error>> {
        let test_append_entries_results = AppendEntriesResults {
            term: 0,
            success: false,
        };

        let test_append_entries_results_data =
            Data::AppendEntriesResults(test_append_entries_results)
                .build()
                .await?;

        assert_eq!(test_append_entries_results_data.len(), 73);

        let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        test_append_entries_results_data.push_to_builder(&mut test_flexbuffers_builder);

        let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;
        let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

        assert!(test_flexbuffer_root.is_aligned());
        assert_eq!(test_flexbuffer_root.bitwidth().n_bytes(), 1);
        assert_eq!(test_flexbuffer_root.length(), 2);
        assert_eq!(
            test_flexbuffer_root.as_map().idx("data").as_str(),
            "append_entries_results",
        );
        assert_eq!(test_flexbuffers_root_details.idx("term").as_u8(), 0);
        assert!(!test_flexbuffers_root_details.idx("success").as_bool());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn data_request_vote_arguments() -> Result<(), Box<dyn std::error::Error>> {
        let test_request_vote_arguments = RequestVoteArguments {
            term: 0,
            candidate_id: String::from("some_candidate_id"),
            last_log_index: 0,
            last_log_term: 0,
        };

        let test_request_vote_arguments_data =
            Data::RequestVoteArguments(test_request_vote_arguments)
                .build()
                .await?;

        assert_eq!(test_request_vote_arguments_data.len(), 132);

        let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        test_request_vote_arguments_data.push_to_builder(&mut test_flexbuffers_builder);

        let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;
        let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

        assert!(test_flexbuffer_root.is_aligned());
        assert_eq!(test_flexbuffer_root.bitwidth().n_bytes(), 1);
        assert_eq!(test_flexbuffer_root.length(), 2);
        assert_eq!(
            test_flexbuffer_root.as_map().idx("data").as_str(),
            "request_vote_arguments",
        );
        assert_eq!(test_flexbuffers_root_details.idx("term").as_u8(), 0);
        assert_eq!(
            test_flexbuffers_root_details.idx("candidate_id").as_str(),
            "some_candidate_id",
        );
        assert_eq!(
            test_flexbuffers_root_details.idx("last_log_index").as_u8(),
            0,
        );
        assert_eq!(
            test_flexbuffers_root_details.idx("last_log_term").as_u8(),
            0,
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn data_request_vote_results() -> Result<(), Box<dyn std::error::Error>> {
        let test_request_vote_results = RequestVoteResults {
            term: 0,
            vote_granted: false,
        };

        let test_request_vote_results_data = Data::RequestVoteResults(test_request_vote_results)
            .build()
            .await?;

        assert_eq!(test_request_vote_results_data.len(), 76);

        let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        test_request_vote_results_data.push_to_builder(&mut test_flexbuffers_builder);

        let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;
        let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

        assert!(test_flexbuffer_root.is_aligned());
        assert_eq!(test_flexbuffer_root.bitwidth().n_bytes(), 1);
        assert_eq!(test_flexbuffer_root.length(), 2);
        assert_eq!(
            test_flexbuffer_root.as_map().idx("data").as_str(),
            "request_vote_results",
        );
        assert_eq!(test_flexbuffers_root_details.idx("term").as_u8(), 0);
        assert!(!test_flexbuffers_root_details.idx("vote_granted").as_bool());

        Ok(())
    }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn build_ip_address() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_ip_address = super::build_ip_address().await;

    //     assert_eq!(test_ip_address.to_string().as_str(), "127.0.0.1");

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn build_socket_address() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_ip_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    //     let test_port = 1234;
    //     let test_socket_address = super::build_socket_address(test_ip_address, test_port).await;

    //     assert_eq!(test_socket_address.to_string().as_str(), "127.0.0.1:1234");

    //     Ok(())
    // }

    #[tokio::test(flavor = "multi_thread")]
    async fn build_tcp_socket() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::from_str("127.0.0.1:1234")?;
        let test_tcp_socket = super::build_tcp_socket().await?;

        test_tcp_socket.bind(test_socket_address)?;

        let test_local_address = test_tcp_socket.local_addr()?;

        assert_eq!(test_local_address.to_string().as_str(), "127.0.0.1:1234");

        Ok(())
    }
}
