use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::signal::unix::{signal, SignalKind};

pub struct MembershipServer {
    socket_address: SocketAddr,
}

impl MembershipServer {
    pub async fn init(socket_address: SocketAddr) -> MembershipServer {
        MembershipServer { socket_address }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(self.socket_address).await?;
        let mut buffer = [0; 1024];

        let mut signal = signal(SignalKind::interrupt())?;

        loop {
            tokio::select! {
                biased;
                _ = signal.recv() => {
                    println!("shutting down membership interface..");

                    break
                }
                result = socket.recv_from(&mut buffer) => {
                    match result {
                        Ok((bytes, origin)) => {
                            println!("received bytes -> {:?}", bytes);
                            println!("received from origin -> {:?}", origin);

                            MembershipServer::receive_bytes(&buffer[..bytes]).await?;
                        }
                        Err(error) => {
                            println!("error receiving UDP message -> {:?}", error);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn receive_bytes(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        match bytes {
            b"ack" => println!("received ack!"),
            b"ping" => println!("received ping!"),
            b"ping-req" => println!("received ping request!"),
            _ => panic!("received unexpected bytes!"),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::from_str("0.0.0.0:25000")?;
        let test_membership_server = MembershipServer::init(test_socket_address).await;

        assert_eq!(
            &test_membership_server.socket_address.to_string(),
            "0.0.0.0:25000",
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn receive_bytes_ack() -> Result<(), Box<dyn std::error::Error>> {
        let test_receive_bytes = MembershipServer::receive_bytes(b"ack").await;

        assert!(test_receive_bytes.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn receive_bytes_ping() -> Result<(), Box<dyn std::error::Error>> {
        let test_receive_bytes = MembershipServer::receive_bytes(b"ping").await;

        assert!(test_receive_bytes.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn receive_bytes_ping_req() -> Result<(), Box<dyn std::error::Error>> {
        let test_receive_bytes = MembershipServer::receive_bytes(b"ping-req").await;

        assert!(test_receive_bytes.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[should_panic]
    async fn receive_bytes_panic() {
        let test_receive_bytes = MembershipServer::receive_bytes(b"something to panic!").await;

        assert!(!test_receive_bytes.is_ok());
    }
}
