use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::rpc::build_ip_address;
use crate::rpc::build_socket_address;
use crate::rpc::Interface;

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

    pub async fn transmit(&self, data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];
        let mut tcp_stream = TcpStream::connect(self.socket_address).await?;

        tcp_stream.write_all(data).await?;
        tcp_stream.shutdown().await?;

        let received_data = tcp_stream.read(&mut buffer).await?;
        let string = String::from_utf8_lossy(&buffer[0..received_data]).to_string();

        Ok(string)
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

    #[tokio::test(flavor = "multi_thread")]
    async fn connect_communications() -> Result<(), Box<dyn std::error::Error>> {
        let test_server_communications = Server::init(Interface::Communications).await?;
        let test_server_handle = tokio::spawn(async move {
            if let Err(error) = test_server_communications.run().await {
                println!("{:?}", error);
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        let test_client_communications = Client::init(Interface::Communications).await?;
        let test_data = test_client_communications
            .transmit(b"test_client_communications")
            .await?;

        assert_eq!(test_data.as_str(), "test_client_communications");
        assert!(test_server_handle.await.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn connect_membership() -> Result<(), Box<dyn std::error::Error>> {
        let test_server_membership = Server::init(Interface::Membership).await?;
        let test_server_handle = tokio::spawn(async move {
            if let Err(error) = test_server_membership.run().await {
                println!("{:?}", error);
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        let test_client_membership = Client::init(Interface::Membership).await?;
        let test_data = test_client_membership
            .transmit(b"test_member_communications")
            .await?;

        assert_eq!(test_data.as_str(), "test_member_communications");
        assert!(test_server_handle.await.is_ok());

        Ok(())
    }
}
