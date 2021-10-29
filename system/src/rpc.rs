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
    use std::str::FromStr;

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
