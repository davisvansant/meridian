use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::channel::MembershipDynamicJoinShutdown;

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
    shutdown: MembershipDynamicJoinShutdown,
}

impl MembershipDynamicJoin {
    pub async fn init(
        socket_address: SocketAddr,
        shutdown: MembershipDynamicJoinShutdown,
    ) -> Result<MembershipDynamicJoin, Box<dyn std::error::Error>> {
        let buffer = [0; 1024];
        let multicast_address = Ipv4Addr::new(239, 0, 0, 1);
        let multicast_interface = Ipv4Addr::from_str(socket_address.ip().to_string().as_str())?;

        Ok(MembershipDynamicJoin {
            socket_address,
            buffer,
            multicast_address,
            multicast_interface,
            shutdown,
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
            let mut attempts = 0;

            while attempts <= 2 {
                if let Err(error) =
                    MembershipDynamicJoin::send_request(&socket_sender, target).await
                {
                    println!("error sending multicast join request -> {:?}", error);
                }

                attempts += 1;
            }
        });

        // loop {
        while self.shutdown.recv().await.is_some() {
            MembershipDynamicJoin::receive_request(&socket_receiver, &mut self.buffer).await?;
        }

        Ok(())
    }

    async fn receive_request(
        socket_receiver: &UdpSocket,
        buffer: &mut [u8; 1024],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (bytes, origin) = socket_receiver.recv_from(buffer).await?;

        let message = Message::from_bytes(&buffer[..bytes]).await;

        match message {
            Message::Join => {
                println!("incoming joing request from the following...{:?}", origin);

                //add membership send request and join node to cluster...
            }
        }

        Ok(())
    }

    async fn send_request(
        socket_sender: &UdpSocket,
        target: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let join = Message::Join.build().await;

        socket_sender.send_to(join, target).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

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
        let (test_send_shutdown, test_receive_shutdown) = mpsc::channel::<bool>(1);
        let test_membership_dynamic_join =
            MembershipDynamicJoin::init(test_socket_address, test_receive_shutdown).await?;

        assert!(test_membership_dynamic_join.socket_address.ip().is_ipv4());
        assert_eq!(test_membership_dynamic_join.buffer, [0_u8; 1024]);
        assert!(test_membership_dynamic_join
            .multicast_address
            .is_multicast());
        assert_eq!(
            test_membership_dynamic_join.multicast_interface,
            Ipv4Addr::UNSPECIFIED,
        );
        assert!(!test_send_shutdown.is_closed());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn send_and_receive_request() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8888);
        let (
            _test_send_memberhsip_dynamic_join_shutdown,
            test_receive_membership_dynamic_join_shutdown,
        ) = mpsc::channel::<bool>(1);
        let mut test_membership_dynamic_join = MembershipDynamicJoin::init(
            test_socket_address,
            test_receive_membership_dynamic_join_shutdown,
        )
        .await?;

        let test_receive_multicast = tokio::spawn(async move {
            let test_socket = UdpSocket::bind(test_membership_dynamic_join.socket_address)
                .await
                .unwrap();

            test_socket
                .join_multicast_v4(
                    test_membership_dynamic_join.multicast_address,
                    test_membership_dynamic_join.multicast_interface,
                )
                .unwrap();

            let test_receive_join_request = MembershipDynamicJoin::receive_request(
                &test_socket,
                &mut test_membership_dynamic_join.buffer,
            )
            .await;

            assert!(test_receive_join_request.is_ok());
        });

        let test_sender_socket_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8889);
        let (_test_sender_send_dynamic_join_shutdown, test_sender_receive_dynamic_join_shutdown) =
            mpsc::channel::<bool>(1);
        let test_sender_membership_dynamic_join = MembershipDynamicJoin::init(
            test_socket_address,
            test_sender_receive_dynamic_join_shutdown,
        )
        .await?;
        let test_sender_socket = UdpSocket::bind(test_sender_socket_address).await?;
        // let test_message = Message::Join.build().await;
        let test_target = SocketAddr::new(
            IpAddr::from(test_sender_membership_dynamic_join.multicast_address),
            test_sender_membership_dynamic_join.socket_address.port(),
        );

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        let test_send_multicast =
            MembershipDynamicJoin::send_request(&test_sender_socket, test_target).await;

        assert!(test_receive_multicast.await.is_ok());
        assert!(test_send_multicast.is_ok());

        Ok(())
    }
}
