use flexbuffers::{Builder, BuilderOptions, Pushable};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpSocket;

use crate::rpc::{
    AppendEntriesArguments, AppendEntriesResults, Data, RequestVoteArguments, RequestVoteResults,
};

pub struct Client {
    socket_address: SocketAddr,
}

impl Client {
    pub async fn init(socket_address: SocketAddr) -> Client {
        Client { socket_address }
    }

    pub async fn send_append_entries(
        &mut self,
        append_entries_arguments: AppendEntriesArguments,
    ) -> Result<AppendEntriesResults, Box<dyn std::error::Error>> {
        let data = Data::AppendEntriesArguments(append_entries_arguments)
            .build()
            .await?;

        let response = self.transmit(&data).await?;

        let mut flexbuffer_builder = Builder::new(BuilderOptions::SHARE_NONE);

        response.push_to_builder(&mut flexbuffer_builder);

        let flexbuffer_root = flexbuffers::Reader::get_root(flexbuffer_builder.view())?;
        let flexbuffer_root_details = flexbuffer_root.as_map().idx("details").as_map();

        let append_entries_results = AppendEntriesResults {
            term: flexbuffer_root_details.idx("term").as_u32(),
            success: flexbuffer_root_details.idx("success").as_bool(),
        };

        Ok(append_entries_results)
    }

    pub async fn send_request_vote(
        &mut self,
        request_vote_arguments: RequestVoteArguments,
    ) -> Result<RequestVoteResults, Box<dyn std::error::Error>> {
        let data = Data::RequestVoteArguments(request_vote_arguments)
            .build()
            .await?;

        let response = self.transmit(&data).await?;

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

    async fn transmit(&mut self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];
        let tcp_socket = TcpSocket::new_v4()?;

        tcp_socket.set_reuseaddr(false)?;
        tcp_socket.set_reuseport(false)?;

        let mut tcp_stream = tcp_socket.connect(self.socket_address).await?;

        tcp_stream.write_all(data).await?;
        tcp_stream.shutdown().await?;

        let received_data = tcp_stream.read(&mut buffer).await?;

        Ok(buffer[0..received_data].to_vec())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test(flavor = "multi_thread")]
//     async fn init() -> Result<(), Box<dyn std::error::Error>> {
//         let (test_client_sender, test_client_receiver) = crate::channel::rpc_client::build().await;
//         let (test_membership_sender, _test_membership_receiver) =
//             crate::channel::membership::build().await;
//         let (test_state_sender, _test_state_receiver) = crate::channel::state::build().await;
//         // let test_candidate_sender = crate::channel::server::build_candidate_transition().await;
//         // let _test_candidate_receiver = test_candidate_sender.subscribe();

//         let test_client = Client::init(
//             test_client_receiver,
//             test_membership_sender,
//             test_state_sender,
//             // test_candidate_sender,
//         )
//         .await?;

//         assert!(!test_client_sender.is_closed());
//         assert!(!test_client.membership_sender.is_closed());
//         assert!(!test_client.state_sender.is_closed());

//         Ok(())
//     }
// }
