// use flexbuffers::{Builder, BuilderOptions, Pushable};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpSocket;

use crate::rpc::{Request, Response};

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

    // pub async fn send_append_entries(
    //     &mut self,
    //     append_entries_arguments: AppendEntriesArguments,
    // ) -> Result<AppendEntriesResults, Box<dyn std::error::Error>> {
    //     let data = Data::AppendEntriesArguments(append_entries_arguments)
    //         .build()
    //         .await?;

    //     let response = self.transmit(&data).await?;

    //     let mut flexbuffer_builder = Builder::new(BuilderOptions::SHARE_NONE);

    //     response.push_to_builder(&mut flexbuffer_builder);

    //     let flexbuffer_root = flexbuffers::Reader::get_root(flexbuffer_builder.view())?;
    //     let flexbuffer_root_details = flexbuffer_root.as_map().idx("details").as_map();

    //     let append_entries_results = AppendEntriesResults {
    //         term: flexbuffer_root_details.idx("term").as_u32(),
    //         success: flexbuffer_root_details.idx("success").as_bool(),
    //     };

    //     Ok(append_entries_results)
    // }
    pub async fn send_append_entries(
        &mut self,
        arguments: AppendEntriesArguments,
    ) -> Result<AppendEntriesResults, Box<dyn std::error::Error>> {
        let request_bytes = Request::AppendEntries(arguments).to()?;
        let mut response_bytes = self.transmit(&request_bytes).await?;

        match Response::from(&mut response_bytes)? {
            Response::AppendEntries(results) => Ok(results),
            Response::RequestVote(_) => panic!("expeced append entries results"),
        }
    }

    pub async fn send_request_vote(
        &mut self,
        arguments: RequestVoteArguments,
    ) -> Result<RequestVoteResults, Box<dyn std::error::Error>> {
        let request_bytes = Request::RequestVote(arguments).to()?;
        let mut response_bytes = self.transmit(&request_bytes).await?;

        match Response::from(&mut response_bytes)? {
            Response::AppendEntries(_) => panic!("expected request vote results"),
            Response::RequestVote(results) => Ok(results),
        }
    }

    // pub async fn send_request_vote(
    //     &mut self,
    //     request_vote_arguments: RequestVoteArguments,
    // ) -> Result<RequestVoteResults, Box<dyn std::error::Error>> {
    //     let data = Data::RequestVoteArguments(request_vote_arguments)
    //         .build()
    //         .await?;

    //     let response = self.transmit(&data).await?;

    //     let mut flexbuffer_builder = Builder::new(BuilderOptions::SHARE_NONE);

    //     response.push_to_builder(&mut flexbuffer_builder);

    //     let flexbuffer_root = flexbuffers::Reader::get_root(flexbuffer_builder.view())?;
    //     let flexbuffer_root_details = flexbuffer_root.as_map().idx("details").as_map();

    //     let request_vote_results = RequestVoteResults {
    //         term: flexbuffer_root_details.idx("term").as_u32(),
    //         vote_granted: flexbuffer_root_details.idx("vote_granted").as_bool(),
    //     };

    //     Ok(request_vote_results)
    // }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::from_str("127.0.0.1:10000")?;
        let test_client = Client::init(test_socket_address).await;

        assert!(test_client.socket_address.is_ipv4());
        assert_eq!(
            test_client.socket_address.ip().to_string().as_str(),
            "127.0.0.1",
        );
        assert_eq!(test_client.socket_address.port(), 10000);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn transmit() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::from_str("127.0.0.1:10000")?;
        let mut test_client = Client::init(test_socket_address).await;
        let test_tcp_socket = TcpSocket::new_v4()?;

        test_tcp_socket.bind(test_socket_address)?;

        let test_tcp_listener = test_tcp_socket.listen(1024)?;

        tokio::spawn(async move {
            match test_tcp_listener.accept().await {
                Ok((mut test_tcp_stream, _test_socket_address)) => {
                    let mut test_buffer = [0; 1024];

                    let test_request_bytes = test_tcp_stream.read(&mut test_buffer).await.unwrap();
                    test_tcp_stream
                        .write_all(&test_buffer[0..test_request_bytes])
                        .await
                        .unwrap();
                    test_tcp_stream.shutdown().await.unwrap();
                }
                Err(_) => panic!("testing transmit error..."),
            }
        });

        let test_response = test_client.transmit(b"test_data").await?;

        assert_eq!(test_response, b"test_data");

        Ok(())
    }
}
