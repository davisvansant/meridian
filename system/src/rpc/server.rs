use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::rpc::build_ip_address;
use crate::rpc::build_socket_address;
use crate::rpc::build_tcp_socket;
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
                println!(
                    "received - {:?}",
                    String::from_utf8_lossy(&buffer[0..data_length]),
                );

                tcp_stream.write_all(&buffer[0..data_length]).await?;
                tcp_stream.shutdown().await?;
            }
            Err(error) => println!("{:?}", error),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn run_communications() -> Result<(), Box<dyn std::error::Error>> {
        let test_interface_communications = Server::init(Interface::Communications).await?;
        let test_handle = tokio::spawn(async move {
            if let Err(error) = test_interface_communications.run().await {
                println!("{:?}", error);
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        let mut test_stream = tokio::net::TcpStream::connect("127.0.0.1:1245").await?;

        test_stream
            .write_all(b"test_rpc_communications_interface")
            .await?;
        test_stream.shutdown().await?;

        let mut test_buffer = [0; 1024];
        let test_data = test_stream.read(&mut test_buffer).await?;
        let test_response = String::from_utf8_lossy(&test_buffer[0..test_data]);

        assert_eq!(
            test_response.to_string().as_str(),
            "test_rpc_communications_interface",
        );
        assert!(test_handle.await.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn run_membership() -> Result<(), Box<dyn std::error::Error>> {
        let test_interface_membership = Server::init(Interface::Membership).await?;
        let test_handle = tokio::spawn(async move {
            if let Err(error) = test_interface_membership.run().await {
                println!("{:?}", error);
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        let mut test_stream = tokio::net::TcpStream::connect("127.0.0.1:1246").await?;

        test_stream
            .write_all(b"test_rpc_membership_interface")
            .await?;
        test_stream.shutdown().await?;

        let mut test_buffer = [0; 1024];
        let test_data = test_stream.read(&mut test_buffer).await?;
        let test_response = String::from_utf8_lossy(&test_buffer[0..test_data]);

        assert_eq!(
            test_response.to_string().as_str(),
            "test_rpc_membership_interface",
        );
        assert!(test_handle.await.is_ok());

        Ok(())
    }
}
