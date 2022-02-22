use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::signal::unix::{signal, SignalKind};

use crate::channel::MembershipListSender;
use crate::channel::{
    insert_alive, insert_suspected, remove_alive, remove_confirmed, remove_suspected,
};
use crate::channel::{MembershipCommunicationsMessage, MembershipCommunicationsSender};
use crate::membership::Message;
use crate::node::Node;

pub struct MembershipCommunications {
    socket_address: SocketAddr,
    list_sender: MembershipListSender,
    receiver: MembershipCommunicationsSender,
}

impl MembershipCommunications {
    pub async fn init(
        socket_address: SocketAddr,
        list_sender: MembershipListSender,
        receiver: MembershipCommunicationsSender,
    ) -> MembershipCommunications {
        MembershipCommunications {
            socket_address,
            list_sender,
            receiver,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(self.socket_address).await?;
        let mut buffer = [0; 1024];

        let receiving_udp_socket = Arc::new(socket);
        let sending_udp_socket = receiving_udp_socket.clone();

        let sender = self.receiver.to_owned();
        let mut receiver = self.receiver.subscribe();

        tokio::spawn(async move {
            while let Ok(incoming_message) = receiver.recv().await {
                match incoming_message {
                    MembershipCommunicationsMessage::Send(bytes, target) => {
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
                }
            }
        });

        let mut signal = signal(SignalKind::interrupt())?;

        loop {
            tokio::select! {
                biased;
                _ = signal.recv() => {
                    println!("shutting down membership interface..");

                    drop(sender);

                    break
                }
                result = receiving_udp_socket.recv_from(&mut buffer) => {
                    match result {
                        Ok((bytes, origin)) => {
                            println!("received bytes -> {:?}", bytes);
                            println!("received from origin -> {:?}", origin);

                            MembershipCommunications::receive_bytes(
                                &sender,
                                &self.list_sender,
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
        list_sender: &MembershipListSender,
        bytes: &[u8],
        origin: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match Message::from_bytes(bytes).await {
            Message::Ack => println!("received ack!"),
            Message::Ping => {
                println!("received ping!");

                let ack = Message::Ack.build().await.to_vec();

                sender.send(MembershipCommunicationsMessage::Send(ack, origin))?;

                let placeholder_node =
                    Node::init(origin.ip(), origin.port(), origin.port(), origin.port()).await?;

                remove_confirmed(list_sender, placeholder_node).await?;
                remove_suspected(list_sender, placeholder_node).await?;
                insert_alive(list_sender, placeholder_node).await?;
            }
            Message::PingReq => {
                println!("received ping request!");

                let ping = Message::Ping.build().await.to_vec();

                sender.send(MembershipCommunicationsMessage::Send(ping, origin))?;

                let placeholder_node =
                    Node::init(origin.ip(), origin.port(), origin.port(), origin.port()).await?;

                remove_alive(list_sender, placeholder_node).await?;
                insert_suspected(list_sender, placeholder_node).await?;
            }
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
    use crate::channel::{MembershipListRequest, MembershipListResponse};
    use std::str::FromStr;
    use tokio::sync::{broadcast, mpsc, oneshot};

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::from_str("0.0.0.0:25000")?;
        let (test_list_sender, _test_list_receiver) = mpsc::channel::<(
            MembershipListRequest,
            oneshot::Sender<MembershipListResponse>,
        )>(1);
        let (test_sender, _test_receiver) =
            broadcast::channel::<MembershipCommunicationsMessage>(1);
        let test_membership_communications =
            MembershipCommunications::init(test_socket_address, test_list_sender, test_sender)
                .await;

        assert_eq!(
            &test_membership_communications.socket_address.to_string(),
            "0.0.0.0:25000",
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn receive_bytes_ack() -> Result<(), Box<dyn std::error::Error>> {
        let (test_list_sender, _test_list_receiver) = mpsc::channel::<(
            MembershipListRequest,
            oneshot::Sender<MembershipListResponse>,
        )>(1);
        let (test_sender, _test_receiver) =
            broadcast::channel::<MembershipCommunicationsMessage>(1);
        let test_bytes = b"ack".to_vec();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000")?;

        let test_receive_bytes = MembershipCommunications::receive_bytes(
            &test_sender,
            &test_list_sender,
            &test_bytes,
            test_origin,
        )
        .await;

        assert!(test_receive_bytes.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn receive_bytes_ping() -> Result<(), Box<dyn std::error::Error>> {
        let (test_sender, _test_receiver) =
            broadcast::channel::<MembershipCommunicationsMessage>(1);
        let (test_list_sender, _test_list_receiver) = mpsc::channel::<(
            MembershipListRequest,
            oneshot::Sender<MembershipListResponse>,
        )>(64);
        let test_bytes = b"ping".to_vec();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000")?;

        let test_receive_bytes = MembershipCommunications::receive_bytes(
            &test_sender,
            &test_list_sender,
            &test_bytes,
            test_origin,
        )
        .await;

        assert!(test_receive_bytes.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn receive_bytes_ping_req() -> Result<(), Box<dyn std::error::Error>> {
        let (test_sender, _test_receiver) =
            broadcast::channel::<MembershipCommunicationsMessage>(1);
        let (test_list_sender, _test_list_receiver) = mpsc::channel::<(
            MembershipListRequest,
            oneshot::Sender<MembershipListResponse>,
        )>(64);
        let test_bytes = b"ping-req".to_vec();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000")?;

        let test_receive_bytes = MembershipCommunications::receive_bytes(
            &test_sender,
            &test_list_sender,
            &test_bytes,
            test_origin,
        )
        .await;

        assert!(test_receive_bytes.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[should_panic]
    async fn receive_bytes_panic() {
        let (test_sender, _test_receiver) =
            broadcast::channel::<MembershipCommunicationsMessage>(1);
        let (test_list_sender, _test_list_receiver) = mpsc::channel::<(
            MembershipListRequest,
            oneshot::Sender<MembershipListResponse>,
        )>(1);
        let test_bytes = b"something to panic!".to_vec();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000").unwrap();

        let test_receive_bytes = MembershipCommunications::receive_bytes(
            &test_sender,
            &test_list_sender,
            &test_bytes,
            test_origin,
        )
        .await;

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
