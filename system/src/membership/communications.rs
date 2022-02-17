use std::net::SocketAddr;
// use std::str::FromStr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;

use crate::channel::{
    MembershipCommunicationsMessage, MembershipCommunicationsReceiver,
    MembershipCommunicationsSender,
};

pub struct MembershipCommunications {
    socket_address: SocketAddr,
}

impl MembershipCommunications {
    pub async fn init(socket_address: SocketAddr) -> MembershipCommunications {
        MembershipCommunications { socket_address }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(self.socket_address).await?;
        let mut buffer = [0; 1024];

        let receiving_udp_socket = Arc::new(socket);
        let sending_udp_socket = receiving_udp_socket.clone();

        let (sender, mut receiver) = mpsc::channel::<MembershipCommunicationsMessage>(64);
        let shutdown = sender.clone();

        tokio::spawn(async move {
            while let Some(incoming_message) = receiver.recv().await {
                match incoming_message {
                    MembershipCommunicationsMessage::Send(bytes, target) => {
                        println!("do send message");

                        if let Err(error) = MembershipCommunications::send_bytes(
                            &sending_udp_socket,
                            &bytes,
                            target,
                        )
                        .await
                        {
                            println!("error sending message -> {:?}", error);
                        }
                    }
                    MembershipCommunicationsMessage::Shutdown => {
                        println!("shutting down receiver");

                        receiver.close();
                    }
                }
            }
        });

        let mut signal = signal(SignalKind::interrupt())?;

        loop {
            tokio::select! {
                biased;
                _ = signal.recv() => {
                    println!("shutting down membership interface..");

                    shutdown.send(MembershipCommunicationsMessage::Shutdown).await?;

                    break
                }
                result = receiving_udp_socket.recv_from(&mut buffer) => {
                    match result {
                        Ok((bytes, origin)) => {
                            println!("received bytes -> {:?}", bytes);
                            println!("received from origin -> {:?}", origin);

                            MembershipCommunications::receive_bytes(
                                &sender,
                                &buffer[..bytes],
                                origin,
                            )
                            .await?;
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

    async fn receive_bytes(
        sender: &MembershipCommunicationsSender,
        bytes: &[u8],
        origin: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match bytes {
            b"ack" => println!("received ack!"),
            b"ping" => {
                println!("received ping!");

                let ack = b"ack".to_vec();

                sender
                    .send(MembershipCommunicationsMessage::Send(ack, origin))
                    .await?;
            }
            b"ping-req" => {
                println!("received ping request!");

                let ping = b"ping".to_vec();

                sender
                    .send(MembershipCommunicationsMessage::Send(ping, origin))
                    .await?;
            }
            _ => panic!("received unexpected bytes!"),
        }

        Ok(())
    }

    async fn send_bytes(
        socket: &UdpSocket,
        bytes: &[u8],
        target: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        socket.send_to(bytes, target).await?;

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
        let test_membership_communications =
            MembershipCommunications::init(test_socket_address).await;

        assert_eq!(
            &test_membership_communications.socket_address.to_string(),
            "0.0.0.0:25000",
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn receive_bytes_ack() -> Result<(), Box<dyn std::error::Error>> {
        let (test_sender, _test_receiver) = mpsc::channel::<MembershipCommunicationsMessage>(1);
        let test_bytes = b"ack".to_vec();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000")?;

        let test_receive_bytes =
            MembershipCommunications::receive_bytes(&test_sender, &test_bytes, test_origin).await;

        assert!(test_receive_bytes.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn receive_bytes_ping() -> Result<(), Box<dyn std::error::Error>> {
        let (test_sender, _test_receiver) = mpsc::channel::<MembershipCommunicationsMessage>(1);
        let test_bytes = b"ping".to_vec();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000")?;

        let test_receive_bytes =
            MembershipCommunications::receive_bytes(&test_sender, &test_bytes, test_origin).await;

        assert!(test_receive_bytes.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn receive_bytes_ping_req() -> Result<(), Box<dyn std::error::Error>> {
        let (test_sender, _test_receiver) = mpsc::channel::<MembershipCommunicationsMessage>(1);
        let test_bytes = b"ping-req".to_vec();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000")?;

        let test_receive_bytes =
            MembershipCommunications::receive_bytes(&test_sender, &test_bytes, test_origin).await;

        assert!(test_receive_bytes.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[should_panic]
    async fn receive_bytes_panic() {
        let (test_sender, _test_receiver) = mpsc::channel::<MembershipCommunicationsMessage>(1);
        let test_bytes = b"something to panic!".to_vec();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000").unwrap();

        let test_receive_bytes =
            MembershipCommunications::receive_bytes(&test_sender, &test_bytes, test_origin).await;

        assert!(!test_receive_bytes.is_ok());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn send_bytes_ack() -> Result<(), Box<dyn std::error::Error>> {
        let test_receiver = tokio::spawn(async move {
            let test_socket_address = SocketAddr::from_str("0.0.0.0:25000").unwrap();
            let test_socket = UdpSocket::bind(test_socket_address).await.unwrap();

            let mut test_buffer = [0; 1024];

            let (test_bytes, test_origin) = test_socket.recv_from(&mut test_buffer).await.unwrap();

            assert_eq!(test_bytes, 3);
            assert_eq!(&test_origin.to_string(), "127.0.0.1:25001");
        });

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        let test_socket_address = SocketAddr::from_str("0.0.0.0:25001").unwrap();
        let test_socket = UdpSocket::bind(test_socket_address).await.unwrap();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000").unwrap();

        let test_send_bytes =
            MembershipCommunications::send_bytes(&test_socket, b"ack", test_origin).await;

        assert!(test_receiver.await.is_ok());
        assert!(test_send_bytes.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn send_bytes_ping() -> Result<(), Box<dyn std::error::Error>> {
        let test_receiver = tokio::spawn(async move {
            let test_socket_address = SocketAddr::from_str("0.0.0.0:25000").unwrap();
            let test_socket = UdpSocket::bind(test_socket_address).await.unwrap();

            let mut test_buffer = [0; 1024];

            let (test_bytes, test_origin) = test_socket.recv_from(&mut test_buffer).await.unwrap();

            assert_eq!(test_bytes, 4);
            assert_eq!(&test_origin.to_string(), "127.0.0.1:25001");
        });

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        let test_socket_address = SocketAddr::from_str("0.0.0.0:25001").unwrap();
        let test_socket = UdpSocket::bind(test_socket_address).await.unwrap();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000").unwrap();

        let test_send_bytes =
            MembershipCommunications::send_bytes(&test_socket, b"ping", test_origin).await;

        assert!(test_receiver.await.is_ok());
        assert!(test_send_bytes.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn send_bytes_ping_req() -> Result<(), Box<dyn std::error::Error>> {
        let test_receiver = tokio::spawn(async move {
            let test_socket_address = SocketAddr::from_str("0.0.0.0:25000").unwrap();
            let test_socket = UdpSocket::bind(test_socket_address).await.unwrap();

            let mut test_buffer = [0; 1024];

            let (test_bytes, test_origin) = test_socket.recv_from(&mut test_buffer).await.unwrap();

            assert_eq!(test_bytes, 8);
            assert_eq!(&test_origin.to_string(), "127.0.0.1:25001");
        });

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        let test_socket_address = SocketAddr::from_str("0.0.0.0:25001").unwrap();
        let test_socket = UdpSocket::bind(test_socket_address).await.unwrap();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000").unwrap();

        let test_send_bytes =
            MembershipCommunications::send_bytes(&test_socket, b"ping-req", test_origin).await;

        assert!(test_receiver.await.is_ok());
        assert!(test_send_bytes.is_ok());

        Ok(())
    }
}
