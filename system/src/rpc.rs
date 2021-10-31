use flexbuffers::{Builder, BuilderOptions, FlexbufferSerializer, MapBuilder};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpSocket};

pub mod append_entries;
pub mod install_snapshot;
pub mod request_vote;

mod client;
mod server;

pub use client::Client;
pub use server::Server;

pub enum Interface {
    Communications,
    Membership,
}

pub enum Data {
    AppendEntriesArguments,
    InstallSnapshot,
    RequestVoteArguments,
    RequestVoteResults,
}

impl Data {
    pub async fn build(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let flexbuffer_options = BuilderOptions::SHARE_NONE;
        let mut flexbuffers_builder = Builder::new(flexbuffer_options);
        let mut flexbuffers_data = flexbuffers_builder.start_map();

        match self {
            Data::AppendEntriesArguments => {
                let append_entries_arguments = append_entries::Arguments::build().await?;

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
            Data::InstallSnapshot => {
                unimplemented!();
            }
            Data::RequestVoteArguments => {
                let request_vote_arguments = request_vote::Arguments::build().await?;

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
            Data::RequestVoteResults => {
                let request_vote_results = request_vote::Results::build().await?;

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

pub async fn build_ip_address() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
}

pub async fn build_socket_address(ip_address: IpAddr, port: u16) -> SocketAddr {
    SocketAddr::new(ip_address, port)
}

pub async fn build_tcp_socket(
    socket_address: SocketAddr,
) -> Result<TcpSocket, Box<dyn std::error::Error>> {
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
        let test_append_entries_arguments = Data::AppendEntriesArguments.build().await?;

        assert_eq!(test_append_entries_arguments.len(), 157);

        let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        test_append_entries_arguments.push_to_builder(&mut test_flexbuffers_builder);

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
    async fn data_request_vote_arguments() -> Result<(), Box<dyn std::error::Error>> {
        let test_request_vote_arguments = Data::RequestVoteArguments.build().await?;

        assert_eq!(test_request_vote_arguments.len(), 132);

        let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        test_request_vote_arguments.push_to_builder(&mut test_flexbuffers_builder);

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
        let test_request_vote_results = Data::RequestVoteResults.build().await?;

        assert_eq!(test_request_vote_results.len(), 76);

        let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        test_request_vote_results.push_to_builder(&mut test_flexbuffers_builder);

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

    #[tokio::test(flavor = "multi_thread")]
    async fn build_ip_address() -> Result<(), Box<dyn std::error::Error>> {
        let test_ip_address = super::build_ip_address().await;

        assert_eq!(test_ip_address.to_string().as_str(), "127.0.0.1");

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn build_socket_address() -> Result<(), Box<dyn std::error::Error>> {
        let test_ip_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let test_port = 1234;
        let test_socket_address = super::build_socket_address(test_ip_address, test_port).await;

        assert_eq!(test_socket_address.to_string().as_str(), "127.0.0.1:1234");

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn build_tcp_socket() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::from_str("127.0.0.1:1234")?;
        let test_tcp_socket = super::build_tcp_socket(test_socket_address).await?;

        test_tcp_socket.bind(test_socket_address)?;

        let test_local_address = test_tcp_socket.local_addr()?;

        assert_eq!(test_local_address.to_string().as_str(), "127.0.0.1:1234");

        Ok(())
    }
}
