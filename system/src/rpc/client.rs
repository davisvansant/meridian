use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::rpc::build_ip_address;
use crate::rpc::build_socket_address;
use crate::rpc::Interface;

use flexbuffers::singleton;

use crate::rpc::Node;

use crate::rpc::Data;

use std::str::FromStr;

use flexbuffers::Pushable;

use flexbuffers::{Builder, BuilderOptions};

use uuid::Uuid;

use crate::rpc::membership::Connected;

use crate::rpc::membership::MembershipNode;

pub struct Client {
    ip_address: IpAddr,
    port: u16,
    socket_address: SocketAddr,
}

impl Client {
    pub async fn init(interface: Interface) -> Result<Client, Box<dyn std::error::Error>> {
        let ip_address = build_ip_address().await;
        let port = match interface {
            Interface::Communications => 1245,
            Interface::Membership => 1246,
        };

        let socket_address = build_socket_address(ip_address, port).await;

        Ok(Client {
            ip_address,
            port,
            socket_address,
        })
    }

    pub async fn join_cluster(&self) -> Result<(), Box<dyn std::error::Error>> {
        let test_node_address = std::net::IpAddr::from_str("0.0.0.0")?;
        let test_node = Node::init(test_node_address, 10000, 15000, 20000).await?;
        let request = Data::JoinClusterRequest(test_node).build().await?;
        let response = self.transmit(&request).await?;

        let mut flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        response.push_to_builder(&mut flexbuffers_builder);

        let flexbuffer_root = flexbuffers::Reader::get_root(flexbuffers_builder.view())?;

        // let test_flexbuffers_root_details = test_flexbuffer_root.as_map().idx("details").as_map();
        // let flexbuffer = singleton(response);
        // let
        // let remote_node = singleton(&response);
        // let root = remote_node.get_root()?;

        // // println!("response -> {:?}", String::from_utf8(response)?);
        // for value in remote_node.as_map().idx("details").as_map().iter_values() {
        //     println!("some value here - {:?}", value.as_str());
        // }
        // for key in flexbuffer_root.as_map().idx("details").as_map().iter_keys() {
        //     println!("some keys from the buffer -> {:?}", key)
        // }

        // for value in flexbuffer_root.as_map().idx("details").as_map().iter_values() {
        //     println!("some values from the buffer -> {:?}", value.as_str())
        // }

        Ok(())
    }

    pub async fn get_connected(&self) -> Result<Vec<MembershipNode>, Box<dyn std::error::Error>> {
        let request = Data::ConnectedRequest.build().await?;
        let response = self.transmit(&request).await?;

        let mut another_test_flexbuffers_builder = Builder::new(BuilderOptions::SHARE_NONE);

        response.push_to_builder(&mut another_test_flexbuffers_builder);

        let another_test_flexbuffer_root =
            flexbuffers::Reader::get_root(another_test_flexbuffers_builder.view())?;

        println!("Im still alive");

        let flexbuffers_root = another_test_flexbuffer_root
            .as_map()
            .idx("details")
            .as_map()
            .idx("nodes")
            .as_vector();
        let mut connected = Connected::build().await?;

        println!("{:?}", &flexbuffers_root.len());

        println!("building the sclak");

        match flexbuffers_root.is_empty() {
            true => {
                println!("empty nodes...");
            }
            false => {
                for node in flexbuffers_root.iter() {
                    println!("are we the errorrer");
                    println!("{:?}", node.as_map().idx("id").as_str());
                    let id =
                        Uuid::parse_str(node.as_map().idx("details").as_map().idx("id").as_str())?;
                    println!("are we happy?");
                    let address = IpAddr::from_str(
                        node.as_map()
                            .idx("details")
                            .as_map()
                            .idx("address")
                            .as_str(),
                    )?;
                    println!("are we dead?");
                    let client_port = u16::from_str(
                        node.as_map()
                            .idx("details")
                            .as_map()
                            .idx("client_port")
                            .as_str(),
                    )?;
                    let cluster_port = u16::from_str(
                        node.as_map()
                            .idx("details")
                            .as_map()
                            .idx("cluster_port")
                            .as_str(),
                    )?;
                    let membership_port = u16::from_str(
                        node.as_map()
                            .idx("details")
                            .as_map()
                            .idx("membership_port")
                            .as_str(),
                    )?;

                    let connected_node = MembershipNode {
                        id: id.to_string(),
                        address: address.to_string(),
                        client_port: client_port.to_string(),
                        cluster_port: cluster_port.to_string(),
                        membership_port: membership_port.to_string(),
                    };

                    connected.nodes.push(connected_node);
                }
            }
        }
        Ok(connected.nodes)
    }

    pub async fn transmit(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];
        let mut tcp_stream = TcpStream::connect(self.socket_address).await?;

        tcp_stream.write_all(data).await?;
        tcp_stream.shutdown().await?;

        let received_data = tcp_stream.read(&mut buffer).await?;

        Ok(buffer[0..received_data].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::Server;

    #[tokio::test(flavor = "multi_thread")]
    async fn init_communications() -> Result<(), Box<dyn std::error::Error>> {
        let test_client_communications = Client::init(Interface::Communications).await?;

        assert_eq!(
            test_client_communications.ip_address.to_string().as_str(),
            "127.0.0.1",
        );
        assert_eq!(test_client_communications.port, 1245);
        assert_eq!(
            test_client_communications
                .socket_address
                .to_string()
                .as_str(),
            "127.0.0.1:1245",
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init_membership() -> Result<(), Box<dyn std::error::Error>> {
        let test_client_membership = Client::init(Interface::Membership).await?;

        assert_eq!(
            test_client_membership.ip_address.to_string().as_str(),
            "127.0.0.1",
        );
        assert_eq!(test_client_membership.port, 1246);
        assert_eq!(
            test_client_membership.socket_address.to_string().as_str(),
            "127.0.0.1:1246",
        );

        Ok(())
    }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn connect_communications() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_server_communications = Server::init(Interface::Communications).await?;
    //     let test_server_handle = tokio::spawn(async move {
    //         if let Err(error) = test_server_communications.run().await {
    //             println!("{:?}", error);
    //         }
    //     });
    //
    //     tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    //
    //     let test_client_communications = Client::init(Interface::Communications).await?;
    //     let test_data = test_client_communications
    //         .transmit(b"test_client_communications")
    //         .await?;
    //
    //     assert_eq!(test_data.as_str(), "test_client_communications");
    //     assert!(test_server_handle.await.is_ok());
    //
    //     Ok(())
    // }
    //
    // #[tokio::test(flavor = "multi_thread")]
    // async fn connect_membership() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_server_membership = Server::init(Interface::Membership).await?;
    //     let test_server_handle = tokio::spawn(async move {
    //         if let Err(error) = test_server_membership.run().await {
    //             println!("{:?}", error);
    //         }
    //     });
    //
    //     tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    //
    //     let test_client_membership = Client::init(Interface::Membership).await?;
    //     let test_data = test_client_membership
    //         .transmit(b"test_member_communications")
    //         .await?;
    //
    //     assert_eq!(test_data.as_str(), "test_member_communications");
    //     assert!(test_server_handle.await.is_ok());
    //
    //     Ok(())
    // }
}
