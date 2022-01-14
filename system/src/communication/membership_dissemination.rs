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
    pub async fn build(&self) -> &[u8] {
        match self {
            Message::Ack => "ack".as_bytes(),
            Message::Ping => "ping".as_bytes(),
            Message::PingReq => "ping-req".as_bytes(),
        }
    }
    pub async fn from_bytes(bytes: &[u8]) -> Message {
        match bytes {
            b"ack" => Message::Ack,
            b"ping" => Message::Ping,
            b"ping-req" => Message::PingReq,
            _ => panic!("cannot build requested bytes into message"),
        }
    }
}

enum Suspicion {
    Alive,
    Confirm,
    Suspect,
}

struct GroupMember {
    suspicion: Suspicion,
    incarnation: u32,
}

impl GroupMember {
    async fn init() -> GroupMember {
        let suspicion = Suspicion::Alive;
        let incarnation = 0;

        GroupMember {
            suspicion,
            incarnation,
        }
    }

    async fn mark_suspicison(&mut self, suspicion: Suspicion) {
        self.suspicion = suspicion;

        self.incarnation += 1
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

        let incoming_udp_message = Arc::new(socket);
        let failure_detector = incoming_udp_message.clone();

        tokio::spawn(async move {
            // set timer
            // choose random member
            let mut attemps = 0;

            while attemps <= 2 {
                let random_member = SocketAddr::from_str("0.0.0.0:250200").unwrap(); //for now
                let ping = Message::Ping;

                match MembershipDissemination::send_message(ping, &failure_detector, random_member)
                    .await
                {
                    Ok(()) => {
                        println!("member is alive!");

                        break;
                    }
                    Err(error) => {
                        println!("error receiving ping from member -> {:?}", error);
                        println!("sending ping-req to another member to check...");
                    }
                }

                let another_member = SocketAddr::from_str("0.0.0.0:250202").unwrap(); //for now
                let ping_req = Message::PingReq;

                if let Err(error) = MembershipDissemination::send_message(
                    ping_req,
                    &failure_detector,
                    another_member,
                )
                .await
                {
                    println!("error sening ping-req to {:}, {:?}", another_member, error);
                }

                attemps += 1;
            }
        });

        loop {
            let (bytes, origin) = incoming_udp_message.recv_from(&mut self.buffer).await?;

            println!("incoming bytes - {:?}", bytes);
            println!("origin - {:?}", origin);

            let message = Message::from_bytes(&self.buffer[..bytes]).await;

            match message {
                Message::Ack => println!("received ack!"),
                Message::Ping => {
                    println!("received ping!");

                    // sender.send((Message::Ack, origin)).await?;
                    let ack = Message::Ack.build().await;

                    incoming_udp_message.send_to(ack, origin).await?;
                }
                Message::PingReq => {
                    println!("received ping req!");

                    let suspected = SocketAddr::from_str("0.0.0.0:25055")?;

                    // sender.send((Message::Ping, suspected)).await?;
                    MembershipDissemination::send_message(
                        Message::Ping,
                        &incoming_udp_message,
                        suspected,
                    )
                    .await?;

                    let received_ack = Message::Ack.build().await; // for now...

                    incoming_udp_message.send_to(received_ack, origin).await?;
                }
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

        let data = message.build().await;

        socket.send_to(data, address).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn message_ack() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_ack = Message::Ack.build().await;

        assert_eq!(test_message_ack, b"ack");
        assert_eq!(test_message_ack.len(), 3);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_ping() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_ping = Message::Ping.build().await;

        assert_eq!(test_message_ping, b"ping");
        assert_eq!(test_message_ping.len(), 4);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_ping_req() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_ping_req = Message::PingReq.build().await;

        assert_eq!(test_message_ping_req, b"ping-req");
        assert_eq!(test_message_ping_req.len(), 8);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_ack_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_ack_bytes = b"ack"; //bar
        let test_message_ack = Message::from_bytes(test_ack_bytes).await;

        assert_eq!(test_message_ack, Message::Ack);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_ping_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_ping_bytes = b"ping";
        let test_message_ping = Message::from_bytes(test_ping_bytes).await;

        assert_eq!(test_message_ping, Message::Ping);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_ping_req_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_ping_req_bytes = b"ping-req";
        let test_message_ping_req = Message::from_bytes(test_ping_req_bytes).await;

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

    #[tokio::test(flavor = "multi_thread")]
    async fn send_message() -> Result<(), Box<dyn std::error::Error>> {
        let test_message = Message::Ping;
        let test_local_socket_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888);
        let test_remote_socket_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8889);

        let test_receiver = tokio::spawn(async move {
            let test_remote_socket = UdpSocket::bind(test_remote_socket_address).await.unwrap();
            let mut test_buffer = [0; 1024];

            let (test_bytes, test_origin) = test_remote_socket
                .recv_from(&mut test_buffer)
                .await
                .unwrap();

            let test_data = Message::from_bytes(&test_buffer[..test_bytes]).await;

            assert_eq!(test_data, Message::Ping);
            assert_eq!(test_origin.to_string().as_str(), "127.0.0.1:8888");
        });

        let test_local_socket = UdpSocket::bind(test_local_socket_address).await?;

        MembershipDissemination::send_message(
            Message::Ping,
            &test_local_socket,
            test_remote_socket_address,
        )
        .await?;

        assert!(test_receiver.await.is_ok());

        Ok(())
    }
}