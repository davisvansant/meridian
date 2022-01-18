use std::collections::HashMap;
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

#[derive(Debug, PartialEq)]
enum Suspicion {
    Alive,
    Confirm,
    Suspect,
}

#[derive(Debug)]
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

    async fn alive(&mut self) {
        self.suspicion = Suspicion::Alive
    }

    async fn confirm(&mut self) {
        self.suspicion = Suspicion::Confirm
    }

    async fn suspect(&mut self) {
        self.suspicion = Suspicion::Suspect;

        self.incarnation += 1
    }
}

pub struct MembershipDissemination {
    socket_address: SocketAddr,
    buffer: [u8; 1024],
    suspected: HashMap<SocketAddr, GroupMember>,
    confirmed: HashMap<SocketAddr, GroupMember>,
}

impl MembershipDissemination {
    pub async fn init(
        socket_address: SocketAddr,
    ) -> Result<MembershipDissemination, Box<dyn std::error::Error>> {
        let buffer = [0; 1024];
        let suspected = HashMap::with_capacity(10);
        let confirmed = HashMap::with_capacity(10);

        Ok(MembershipDissemination {
            socket_address,
            buffer,
            suspected,
            confirmed,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(self.socket_address).await?;

        // let incoming_udp_message = Arc::new(socket);
        let incoming_udp_socket = Arc::new(socket);
        let failure_detector = incoming_udp_socket.clone();
        // let failure_detector = incoming_udp_message.clone();

        let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(64);

        tokio::spawn(async move {
            let mut attemps = 0;

            while attemps <= 2 {
                if let Err(error) =
                    MembershipDissemination::send_udp_message(&failure_detector).await
                {
                    println!("error sending udp message -> {:?}", error);
                }

                attemps += 1;
            }
        });

        let mut buffer = self.buffer;

        tokio::spawn(async move {
            let mut attemps = 0;

            while attemps <= 5 {
                if let Err(error) =
                    MembershipDissemination::receive_udp_message(&incoming_udp_socket, &mut buffer)
                        .await
                {
                    println!("error receiving udp message -> {:?}", error);
                }

                attemps += 1;
            }
        });

        while let Some((bytes, origin)) = rx.recv().await {
            println!("do stuff with bytes");
            println!("do stuff with origin");
        }

        Ok(())
    }

    async fn add_confirmed(&mut self, socket_address: SocketAddr, group_member: GroupMember) {
        println!(
            "adding socket address {:?} as suspect {:?}",
            &socket_address, &group_member,
        );

        if let Some(new_confirmed) = self.confirmed.insert(socket_address, group_member) {
            println!("already marked as confirmed!");
        } else {
            println!("suspected confirmed!");
        }
    }

    async fn add_suspect(&mut self, socket_address: SocketAddr, group_member: GroupMember) {
        println!(
            "adding socket address {:?} as suspect {:?}",
            &socket_address, &group_member,
        );

        if let Some(new_suspect) = self.suspected.insert(socket_address, group_member) {
            println!("already marked as suspect!");
        } else {
            println!("suspected updated!");
        }
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

    async fn remove_confirmed(&mut self, socket_address: &SocketAddr) {
        if let Some(remove_confirmed) = self.confirmed.remove(socket_address) {
            println!("removed from confirmed group - > {:?}", remove_confirmed);
        }
    }

    async fn remove_suspected(&mut self, socket_address: &SocketAddr) {
        if let Some(remove_suspected) = self.suspected.remove(socket_address) {
            println!("removed from suspected group - > {:?}", remove_suspected);
        }
    }

    async fn receive_udp_message(
        incoming: &UdpSocket,
        buffer: &mut [u8; 1024],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (bytes, origin) = incoming.recv_from(buffer).await?;

        println!("incoming bytes - {:?}", bytes);
        println!("origin - {:?}", origin);

        let message = Message::from_bytes(&buffer[..bytes]).await;

        match message {
            Message::Ack => println!("received ack!"),
            Message::Ping => {
                println!("received ping!");

                // sender.send((Message::Ack, origin)).await?;
                let ack = Message::Ack.build().await;

                incoming.send_to(ack, origin).await?;
            }
            Message::PingReq => {
                println!("received ping req!");

                let suspected = SocketAddr::from_str("0.0.0.0:25055")?;

                // sender.send((Message::Ping, suspected)).await?;
                MembershipDissemination::send_message(Message::Ping, &incoming, suspected).await?;

                let received_ack = Message::Ack.build().await; // for now...

                incoming.send_to(received_ack, origin).await?;
            }
        }

        Ok(())
    }

    async fn send_udp_message(
        failure_detector: &UdpSocket,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let random_member = SocketAddr::from_str("0.0.0.0:250200")?;
        let ping = Message::Ping;

        match MembershipDissemination::send_message(ping, &failure_detector, random_member).await {
            Ok(()) => {
                println!("member is alive!");

                return Ok(());
            }
            Err(error) => {
                println!("error receiving ping from member -> {:?}", error);
                println!("sending ping-req to another member...");
            }
        }

        let another_member = SocketAddr::from_str("0.0.0.0:250202")?; //for now
        let ping_req = Message::PingReq;

        MembershipDissemination::send_message(ping_req, &failure_detector, another_member).await?;

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
    async fn group_member_init() -> Result<(), Box<dyn std::error::Error>> {
        let test_group_member = GroupMember::init().await;

        assert_eq!(test_group_member.suspicion, Suspicion::Alive);
        assert_eq!(test_group_member.incarnation, 0);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8888);
        let test_membership_dissemination =
            MembershipDissemination::init(test_socket_address).await?;

        assert!(test_membership_dissemination.socket_address.ip().is_ipv4());
        assert_eq!(test_membership_dissemination.buffer, [0_u8; 1024]);
        assert!(test_membership_dissemination.suspected.is_empty());
        assert!(test_membership_dissemination.confirmed.is_empty());

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
