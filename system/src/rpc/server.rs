use flexbuffers::{Builder, BuilderOptions, Pushable};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::rpc::build_ip_address;
use crate::rpc::build_socket_address;
use crate::rpc::build_tcp_socket;
use crate::rpc::Data;
use crate::rpc::Interface;

pub struct Server {
    ip_address: IpAddr,
    port: u16,
    socket_address: SocketAddr,
}

impl Server {
    pub async fn init(interface: Interface) -> Result<Server, Box<dyn std::error::Error>> {
        let ip_address = build_ip_address().await;
        let port = match interface {
            Interface::Communications => 1245,
            Interface::Membership => 1246,
        };

        let socket_address = build_socket_address(ip_address, port).await;

        Ok(Server {
            ip_address,
            port,
            socket_address,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let tcp_socket = build_tcp_socket(self.socket_address).await?;

        tcp_socket.set_reuseport(true)?;
        tcp_socket.bind(self.socket_address)?;

        let backlog = 1024;
        let tcp_listener = tcp_socket.listen(backlog)?;

        let (mut tcp_stream, socket_address) = tcp_listener.accept().await?;

        println!("running on {:?}", socket_address);

        let mut buffer = [0; 1024];

        match tcp_stream.read(&mut buffer).await {
            Ok(data_length) => {
                let result = self.route_incoming(&buffer[0..data_length]).await?;

                tcp_stream.write_all(&result).await?;
                tcp_stream.shutdown().await?;
            }
            Err(error) => println!("{:?}", error),
        }

        Ok(())
    }

    async fn route_incoming(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        data.push_to_builder(&mut flexbuffers_builder);

        let flexbuffers_root = flexbuffers::Reader::get_root(flexbuffers_builder.view())?;

        match flexbuffers_root.as_map().idx("data").as_str() {
            "append_entries_arguments" => {
                println!("received append entries arguments!");

                let append_entries_results = Data::AppendEntriesResults.build().await?;

                Ok(append_entries_results)
            }
            "request_vote_arguments" => {
                println!("received request vote arguments!");

                let request_vote_results = Data::RequestVoteResults.build().await?;

                Ok(request_vote_results)
            }
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
    use crate::rpc::Client;

    #[tokio::test(flavor = "multi_thread")]
    async fn init_communications() -> Result<(), Box<dyn std::error::Error>> {
        let test_interface_communications = Server::init(Interface::Communications).await?;

        assert_eq!(
            test_interface_communications
                .ip_address
                .to_string()
                .as_str(),
            "127.0.0.1",
        );
        assert_eq!(test_interface_communications.port, 1245);
        assert_eq!(
            test_interface_communications
                .socket_address
                .to_string()
                .as_str(),
            "127.0.0.1:1245",
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init_membership() -> Result<(), Box<dyn std::error::Error>> {
        let test_interface_membership = Server::init(Interface::Membership).await?;

        assert_eq!(
            test_interface_membership.ip_address.to_string().as_str(),
            "127.0.0.1",
        );
        assert_eq!(test_interface_membership.port, 1246);
        assert_eq!(
            test_interface_membership
                .socket_address
                .to_string()
                .as_str(),
            "127.0.0.1:1246",
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn run_communications_append_entries() -> Result<(), Box<dyn std::error::Error>> {
        let test_communications_server = Server::init(Interface::Communications).await?;
        let test_handle = tokio::spawn(async move {
            if let Err(error) = test_communications_server.run().await {
                println!("{:?}", error);
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        let test_communications_client = Client::init(Interface::Communications).await?;
        let test_request = Data::AppendEntriesArguments.build().await?;
        let test_data = test_communications_client.transmit(&test_request).await?;

        let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        test_data.push_to_builder(&mut test_flexbuffers_builder);

        let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;
        let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

        assert_eq!(
            test_flexbuffer_root.as_map().idx("data").as_str(),
            "append_entries_results",
        );
        assert_eq!(test_flexbuffers_root_details.idx("term").as_u8(), 0);
        assert!(!test_flexbuffers_root_details.idx("success").as_bool());
        assert!(test_handle.await.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn run_communications_request_vote() -> Result<(), Box<dyn std::error::Error>> {
        let test_communications_server = Server::init(Interface::Communications).await?;
        let test_handle = tokio::spawn(async move {
            if let Err(error) = test_communications_server.run().await {
                println!("{:?}", error);
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        let test_communications_client = Client::init(Interface::Communications).await?;
        let test_request = Data::RequestVoteArguments.build().await?;
        let test_data = test_communications_client.transmit(&test_request).await?;

        let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        test_data.push_to_builder(&mut test_flexbuffers_builder);

        let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;
        let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

        assert_eq!(
            test_flexbuffer_root.as_map().idx("data").as_str(),
            "request_vote_results",
        );
        assert_eq!(test_flexbuffers_root_details.idx("term").as_u8(), 0);
        assert!(!test_flexbuffers_root_details.idx("vote_granted").as_bool());
        assert!(test_handle.await.is_ok());

        Ok(())
    }

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

    #[tokio::test(flavor = "multi_thread")]
    async fn route_incoming_append_entries() -> Result<(), Box<dyn std::error::Error>> {
        let test_membership_server = Server::init(Interface::Membership).await?;
        let test_append_entries_arguments =
            crate::rpc::Data::AppendEntriesArguments.build().await?;
        let test_append_entries_results = test_membership_server
            .route_incoming(&test_append_entries_arguments)
            .await?;

        let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        test_append_entries_results.push_to_builder(&mut test_flexbuffers_builder);

        let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;
        let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

        assert_eq!(
            test_flexbuffer_root.as_map().idx("data").as_str(),
            "append_entries_results",
        );
        assert_eq!(test_flexbuffers_root_details.idx("term").as_u8(), 0);
        assert!(!test_flexbuffers_root_details.idx("success").as_bool());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn route_incoming_request_vote() -> Result<(), Box<dyn std::error::Error>> {
        let test_membership_server = Server::init(Interface::Membership).await?;
        let test_request_vote_arguments = crate::rpc::Data::RequestVoteArguments.build().await?;
        let test_request_vote_results = test_membership_server
            .route_incoming(&test_request_vote_arguments)
            .await?;

        let mut test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        test_request_vote_results.push_to_builder(&mut test_flexbuffers_builder);

        let test_flexbuffer_root = flexbuffers::Reader::get_root(test_flexbuffers_builder.view())?;
        let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();

        assert_eq!(
            test_flexbuffer_root.as_map().idx("data").as_str(),
            "request_vote_results",
        );
        assert_eq!(test_flexbuffers_root_details.idx("term").as_u8(), 0);
        assert!(!test_flexbuffers_root_details.idx("vote_granted").as_bool());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn route_incoming_unknown() -> Result<(), Box<dyn std::error::Error>> {
        let test_membership_server = Server::init(Interface::Membership).await?;
        let test_unknown = crate::rpc::Data::RequestVoteResults.build().await?;
        let test_data = test_membership_server.route_incoming(&test_unknown).await?;

        assert_eq!(test_data, "unknown".as_bytes());

        Ok(())
    }
}
