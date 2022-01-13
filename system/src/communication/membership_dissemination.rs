use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;

#[derive(Debug, PartialEq)]
pub enum Message {
    Ack,
    Ping,
    PingReq,
}

impl Message {
    pub async fn build(&self) -> Result<&[u8], Box<dyn std::error::Error>> {
        let message = match self {
            Message::Ack => "ack".as_bytes(),
            Message::Ping => "ping".as_bytes(),
            Message::PingReq => "ping-req".as_bytes(),
        };

        Ok(message)
    }
    pub async fn from_bytes(bytes: &[u8]) -> Result<Message, Box<dyn std::error::Error>> {
        let message = match bytes {
            b"ack" => Message::Ack,
            b"ping" => Message::Ping,
            b"ping-req" => Message::PingReq,
            _ => panic!("cannot build requested bytes into message"),
        };

        Ok(message)
    }
}

pub struct MembershipDissemination {
    socket_address: SocketAddr,
    buffer: [u8; 1024],
}

impl MembershipDissemination {
    pub async fn init(
        socket_address: SocketAddr,
    ) -> Result<MembershipDissemination, Box<dyn std::error::Error>> {
        let buffer = [0; 1024];

        Ok(MembershipDissemination {
            socket_address,
            buffer,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(self.socket_address).await?;

        let socket_receiver = Arc::new(socket);
        let socket_sender = socket_receiver.clone();

        let (sender, mut receiver) = mpsc::channel::<(Message, SocketAddr)>(64);

        tokio::spawn(async move {
            while let Some((message, address)) = receiver.recv().await {
                if let Err(error) =
                    MembershipDissemination::send_message(message, &socket_sender, address).await
                {
                    println!(
                        "error sending membership dissemination message -> {:?}",
                        error,
                    );
                }
            }
        });

        loop {
            let bytes = socket_receiver.recv(&mut self.buffer).await?;

            println!("incoming bytes - {:?}", bytes);

            let message = Message::from_bytes(&self.buffer[..bytes]).await?;

            match message {
                Message::Ack => println!("received ack!"),
                Message::Ping => println!("received ping!"),
                Message::PingReq => println!(" received ping req!"),
            }
        }

        Ok(())
    }

    async fn send_message(
        message: Message,
        socket: &UdpSocket,
        address: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("sending -> {:?}", &message);
        println!("socket -> {:?}", &socket);
        println!("remote address -> {:?}", &address);

        let buffer = message.build().await?;

        socket.connect(address).await?;
        socket.send(buffer).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn message_ack() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_ack = Message::Ack.build().await?;

        assert_eq!(test_message_ack, b"ack");
        assert_eq!(test_message_ack.len(), 3);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_ping() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_ping = Message::Ping.build().await?;

        assert_eq!(test_message_ping, b"ping");
        assert_eq!(test_message_ping.len(), 4);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_ping_req() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_ping_req = Message::PingReq.build().await?;

        assert_eq!(test_message_ping_req, b"ping-req");
        assert_eq!(test_message_ping_req.len(), 8);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_ack_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_ack_bytes = b"ack"; //bar
        let test_message_ack = Message::from_bytes(test_ack_bytes).await?;

        assert_eq!(test_message_ack, Message::Ack);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_ping_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_ping_bytes = b"ping";
        let test_message_ping = Message::from_bytes(test_ping_bytes).await?;

        assert_eq!(test_message_ping, Message::Ping);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_ping_req_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_ping_req_bytes = b"ping-req";
        let test_message_ping_req = Message::from_bytes(test_ping_req_bytes).await?;

        assert_eq!(test_message_ping_req, Message::PingReq);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8888);
        let test_membership_dissemination =
            MembershipDissemination::init(test_socket_address).await?;

        assert!(test_membership_dissemination.socket_address.ip().is_ipv4());
        assert_eq!(test_membership_dissemination.buffer, [0_u8; 1024]);

        Ok(())
    }
}
