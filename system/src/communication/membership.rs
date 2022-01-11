use std::net::SocketAddr;

use std::net::Ipv4Addr;

use std::net::IpAddr;

use std::str::FromStr;

use tokio::net::UdpSocket;

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
}

pub struct MembershipCommunication {
    socket_address: SocketAddr,
    buffer: [u8; 1024],
    multicast_address: Ipv4Addr,
    multicast_interface: Ipv4Addr,
}

impl MembershipCommunication {
    pub async fn init(
        socket_address: SocketAddr,
    ) -> Result<MembershipCommunication, Box<dyn std::error::Error>> {
        let buffer = [0; 1024];
        let multicast_address = Ipv4Addr::new(239, 0, 0, 1);
        let multicast_interface = Ipv4Addr::from_str(socket_address.ip().to_string().as_str())?;

        Ok(MembershipCommunication {
            socket_address,
            buffer,
            multicast_address,
            multicast_interface,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(self.socket_address).await?;

        socket.join_multicast_v4(self.multicast_address, self.multicast_interface)?;

        loop {
            let (bytes, origin) = socket.recv_from(&mut self.buffer).await?;

            println!("incoming bytes - {:?}", bytes);
            println!("origin - {:?}", origin);

            let len = socket.send_to(&self.buffer[..bytes], origin).await?;
            println!("{:?} bytes sent", len);

            println!(
                "received {:?}",
                String::from_utf8(self.buffer[..bytes].to_vec())?,
            );
        }

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
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8888);
        let test_membership_communication =
            MembershipCommunication::init(test_socket_address).await?;

        assert!(test_membership_communication.socket_address.ip().is_ipv4());
        assert_eq!(test_membership_communication.buffer, [0_u8; 1024]);
        assert!(test_membership_communication
            .multicast_address
            .is_multicast());
        assert_eq!(
            test_membership_communication.multicast_interface,
            Ipv4Addr::UNSPECIFIED,
        );

        Ok(())
    }
}
