use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use tokio::net::UdpSocket;

#[derive(Debug, PartialEq)]
enum Message {
    Join,
}

impl Message {
    pub async fn build(&self) -> &[u8] {
        match self {
            Message::Join => "join".as_bytes(),
        }
    }

    pub async fn from_bytes(bytes: &[u8]) -> Message {
        match bytes {
            b"join" => Message::Join,
            _ => panic!("cannot build requested bytes into message"),
        }
    }
}

pub struct MembershipDynamicJoin {
    socket_address: SocketAddr,
    buffer: [u8; 1024],
    multicast_address: Ipv4Addr,
    multicast_interface: Ipv4Addr,
}

impl MembershipDynamicJoin {
    pub async fn init(
        socket_address: SocketAddr,
    ) -> Result<MembershipDynamicJoin, Box<dyn std::error::Error>> {
        let buffer = [0; 1024];
        let multicast_address = Ipv4Addr::new(239, 0, 0, 1);
        let multicast_interface = Ipv4Addr::from_str(socket_address.ip().to_string().as_str())?;

        Ok(MembershipDynamicJoin {
            socket_address,
            buffer,
            multicast_address,
            multicast_interface,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(self.socket_address).await?;

        socket.join_multicast_v4(self.multicast_address, self.multicast_interface)?;
        // socket.set_multicast_loop_v4(false)?;

        let socket_receiver = Arc::new(socket);
        let socket_sender = socket_receiver.clone();

        let target = SocketAddr::new(
            IpAddr::from(self.multicast_address),
            self.socket_address.port(),
        );

        tokio::spawn(async move {
            let join = Message::Join.build().await;

            let mut attempts = 0;

            while attempts <= 2 {
                println!("sending join request to target -> {:?}", target);
                if let Err(error) = socket_sender.send_to(join, target).await {
                    println!("error sending join UDP multicast request!");
                }

                attempts += 1;
            }
        });

        loop {
            let (bytes, origin) = socket_receiver.recv_from(&mut self.buffer).await?;

            let message = Message::from_bytes(&self.buffer[..bytes]).await;

            match message {
                Message::Join => {
                    println!("incoming joing request from the following...{:?}", origin);

                    //add membership send request and join node to cluster...
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn message_join() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_join = Message::Join.build().await;

        assert_eq!(test_message_join, b"join");
        assert_eq!(test_message_join.len(), 4);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_join_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_join_bytes = b"join";
        let test_message_join = Message::from_bytes(test_join_bytes).await;

        assert_eq!(test_message_join, Message::Join);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8888);
        let test_membership_dynamic_join = MembershipDynamicJoin::init(test_socket_address).await?;

        assert!(test_membership_dynamic_join.socket_address.ip().is_ipv4());
        assert_eq!(test_membership_dynamic_join.buffer, [0_u8; 1024]);
        assert!(test_membership_dynamic_join
            .multicast_address
            .is_multicast());
        assert_eq!(
            test_membership_dynamic_join.multicast_interface,
            Ipv4Addr::UNSPECIFIED,
        );

        Ok(())
    }
}
