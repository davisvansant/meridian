use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

use crate::channel::MembershipListSender;
use crate::channel::ShutdownReceiver;
use crate::channel::{
    get_alive, get_confirmed, get_node, get_suspected, insert_alive, insert_confirmed,
    insert_suspected, remove_alive, remove_confirmed, remove_suspected,
};
use crate::channel::{MembershipCommunicationsMessage, MembershipCommunicationsSender};
use crate::channel::{
    MembershipFailureDetectorPingTarget, MembershipFailureDetectorPingTargetSender,
};
use crate::membership::Message;

pub struct MembershipCommunications {
    socket_address: SocketAddr,
    list_sender: MembershipListSender,
    receiver: MembershipCommunicationsSender,
    ping_target_channel: MembershipFailureDetectorPingTargetSender,
}

impl MembershipCommunications {
    pub async fn init(
        socket_address: SocketAddr,
        list_sender: MembershipListSender,
        receiver: MembershipCommunicationsSender,
        ping_target_channel: MembershipFailureDetectorPingTargetSender,
    ) -> MembershipCommunications {
        MembershipCommunications {
            socket_address,
            list_sender,
            receiver,
            ping_target_channel,
        }
    }

    pub async fn run(
        &mut self,
        shutdown: &mut ShutdownReceiver,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(self.socket_address).await?;
        let mut buffer = [0; 1024];

        let receiving_udp_socket = Arc::new(socket);
        let sending_udp_socket = receiving_udp_socket.clone();

        let mut receiver = self.receiver.subscribe();

        let send_message_list_sender = self.list_sender.to_owned();

        tokio::spawn(async move {
            while let Ok(incoming_message) = receiver.recv().await {
                match incoming_message {
                    MembershipCommunicationsMessage::Send(bytes, target) => {
                        if let Err(error) = MembershipCommunications::send_bytes(
                            &send_message_list_sender,
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

        loop {
            tokio::select! {
                biased;
                _ = shutdown.recv() => {
                    println!("shutting down membership interface..");

                    break
                }
                result = receiving_udp_socket.recv_from(&mut buffer) => {
                    match result {
                        Ok((bytes, origin)) => {
                            println!("received bytes -> {:?}", bytes);
                            println!("received from origin -> {:?}", origin);

                            let send_message = self.receiver.to_owned();
                            let list_sender = self.list_sender.to_owned();
                            let message = buffer[..bytes].to_owned();
                            let ping_target_sender = self.ping_target_channel.to_owned();

                            tokio::spawn(async move {
                                if let Err(error) = MembershipCommunications::receive_bytes(
                                    &send_message,
                                    &list_sender,
                                    &message,
                                    origin,
                                    &ping_target_sender,
                                )
                                .await {
                                    println!("error with running receive bytes -> {:?}", error);
                                }
                            });
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
        ping_target_sender: &MembershipFailureDetectorPingTargetSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (message, origin_node, peer_active_list, peer_suspected_list, peer_confirmed_list) =
            Message::from_list(bytes).await?;

        match message {
            Message::Ack => {
                println!("received ack!");

                insert_alive(list_sender, &origin_node).await?;

                for alive_node in &peer_active_list {
                    insert_alive(list_sender, alive_node).await?;
                }
            }
            Message::Ping => {
                println!("received ping!");

                let node = get_node(list_sender).await?;
                let local_alive_list = get_alive(list_sender).await?;
                let local_suspected_list = get_suspected(list_sender).await?;
                let local_confirmed_list = get_confirmed(list_sender).await?;

                let ack = Message::Ack
                    .build_list(
                        &node,
                        &local_alive_list,
                        &local_suspected_list,
                        &local_confirmed_list,
                    )
                    .await;

                sender.send(MembershipCommunicationsMessage::Send(ack, origin))?;

                ping_target_sender.send(MembershipFailureDetectorPingTarget::Member(origin))?;

                remove_confirmed(list_sender, &origin_node).await?;
                remove_suspected(list_sender, &origin_node).await?;
                insert_alive(list_sender, &origin_node).await?;

                for alive_node in &peer_active_list {
                    insert_alive(list_sender, alive_node).await?;
                }
            }
            Message::PingReq => {
                println!("received ping request!");

                let node = get_node(list_sender).await?;
                let local_alive_list = get_alive(list_sender).await?;
                let local_suspected_list = get_suspected(list_sender).await?;
                let local_confirmed_list = get_confirmed(list_sender).await?;

                let ping = Message::Ack
                    .build_list(
                        &node,
                        &local_alive_list,
                        &local_suspected_list,
                        &local_confirmed_list,
                    )
                    .await;

                sender.send(MembershipCommunicationsMessage::Send(ping, origin))?;

                for suspected_node in &peer_suspected_list {
                    remove_alive(list_sender, suspected_node).await?;
                    remove_confirmed(list_sender, suspected_node).await?;
                    insert_suspected(list_sender, suspected_node).await?;
                }

                for confirmed_node in &peer_confirmed_list {
                    remove_alive(list_sender, confirmed_node).await?;
                    remove_suspected(list_sender, confirmed_node).await?;
                    insert_confirmed(list_sender, confirmed_node).await?;
                }
            }
        }

        Ok(())
    }

    async fn send_bytes(
        list_sender: &MembershipListSender,
        socket: &UdpSocket,
        bytes: &[u8],
        target: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _alive = get_alive(list_sender).await?;
        let _suspected = get_suspected(list_sender).await?;
        let _confirmed = get_confirmed(list_sender).await?;

        socket.send_to(bytes, target).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::build_failure_detector_ping_target_channel;
    use crate::channel::{MembershipListRequest, MembershipListResponse};
    use crate::node::Node;
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
        let test_ping_target_sender = build_failure_detector_ping_target_channel().await;

        let test_membership_communications = MembershipCommunications::init(
            test_socket_address,
            test_list_sender,
            test_sender,
            test_ping_target_sender,
        )
        .await;

        assert_eq!(
            &test_membership_communications.socket_address.to_string(),
            "0.0.0.0:25000",
        );

        Ok(())
    }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn receive_bytes_ack() -> Result<(), Box<dyn std::error::Error>> {
    //     let (test_list_sender, _test_list_receiver) = mpsc::channel::<(
    //         MembershipListRequest,
    //         oneshot::Sender<MembershipListResponse>,
    //     )>(1);
    //     let (test_sender, _test_receiver) =
    //         broadcast::channel::<MembershipCommunicationsMessage>(1);
    //     let test_bytes = b"ack".to_vec();
    //     let test_origin = SocketAddr::from_str("0.0.0.0:25000")?;

    //     let test_receive_bytes = MembershipCommunications::receive_bytes(
    //         &test_sender,
    //         &test_list_sender,
    //         &test_bytes,
    //         test_origin,
    //     )
    //     .await;

    //     assert!(test_receive_bytes.is_err());

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn receive_bytes_ping() -> Result<(), Box<dyn std::error::Error>> {
    //     let (test_sender, _test_receiver) =
    //         broadcast::channel::<MembershipCommunicationsMessage>(1);
    //     let (test_list_sender, _test_list_receiver) = mpsc::channel::<(
    //         MembershipListRequest,
    //         oneshot::Sender<MembershipListResponse>,
    //     )>(64);
    //     let test_bytes = b"ping".to_vec();
    //     let test_origin = SocketAddr::from_str("0.0.0.0:25000")?;

    //     let test_receive_bytes = MembershipCommunications::receive_bytes(
    //         &test_sender,
    //         &test_list_sender,
    //         &test_bytes,
    //         test_origin,
    //     )
    //     .await;

    //     assert!(test_receive_bytes.is_err());

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn receive_bytes_ping_req() -> Result<(), Box<dyn std::error::Error>> {
    //     let (test_sender, _test_receiver) =
    //         broadcast::channel::<MembershipCommunicationsMessage>(1);
    //     let (test_list_sender, _test_list_receiver) = mpsc::channel::<(
    //         MembershipListRequest,
    //         oneshot::Sender<MembershipListResponse>,
    //     )>(64);
    //     let test_bytes = b"ping-req".to_vec();
    //     let test_origin = SocketAddr::from_str("0.0.0.0:25000")?;

    //     let test_receive_bytes = MembershipCommunications::receive_bytes(
    //         &test_sender,
    //         &test_list_sender,
    //         &test_bytes,
    //         test_origin,
    //     )
    //     .await;

    //     assert!(test_receive_bytes.is_err());

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // #[should_panic]
    // async fn receive_bytes_panic() {
    //     let (test_sender, _test_receiver) =
    //         broadcast::channel::<MembershipCommunicationsMessage>(1);
    //     let (test_list_sender, _test_list_receiver) = mpsc::channel::<(
    //         MembershipListRequest,
    //         oneshot::Sender<MembershipListResponse>,
    //     )>(1);
    //     let test_bytes = b"something to panic!".to_vec();
    //     let test_origin = SocketAddr::from_str("0.0.0.0:25000").unwrap();

    //     let test_receive_bytes = MembershipCommunications::receive_bytes(
    //         &test_sender,
    //         &test_list_sender,
    //         &test_bytes,
    //         test_origin,
    //     )
    //     .await;

    //     assert!(test_receive_bytes.is_err());
    // }

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

        let (test_list_sender, mut test_list_receiver) = mpsc::channel::<(
            MembershipListRequest,
            oneshot::Sender<MembershipListResponse>,
        )>(64);

        tokio::spawn(async move {
            while let Some((test_request, test_response)) = test_list_receiver.recv().await {
                match test_request {
                    MembershipListRequest::GetAlive => {
                        let test_alive: Vec<Node> = Vec::with_capacity(0);

                        test_response
                            .send(MembershipListResponse::Alive(test_alive))
                            .unwrap();
                    }
                    MembershipListRequest::GetConfirmed => {
                        let test_confirmed: Vec<Node> = Vec::with_capacity(0);

                        test_response
                            .send(MembershipListResponse::Confirmed(test_confirmed))
                            .unwrap();
                    }
                    MembershipListRequest::GetSuspected => {
                        let test_suspected: Vec<Node> = Vec::with_capacity(0);

                        test_response
                            .send(MembershipListResponse::Suspected(test_suspected))
                            .unwrap();
                    }
                    _ => panic!("send bytes ack test"),
                }
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        let test_socket_address = SocketAddr::from_str("0.0.0.0:25001").unwrap();
        let test_socket = UdpSocket::bind(test_socket_address).await.unwrap();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000").unwrap();

        let test_send_bytes = MembershipCommunications::send_bytes(
            &test_list_sender,
            &test_socket,
            b"ack",
            test_origin,
        )
        .await;

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

        let (test_list_sender, mut test_list_receiver) = mpsc::channel::<(
            MembershipListRequest,
            oneshot::Sender<MembershipListResponse>,
        )>(64);

        tokio::spawn(async move {
            while let Some((test_request, test_response)) = test_list_receiver.recv().await {
                match test_request {
                    MembershipListRequest::GetAlive => {
                        let test_alive: Vec<Node> = Vec::with_capacity(0);

                        test_response
                            .send(MembershipListResponse::Alive(test_alive))
                            .unwrap();
                    }
                    MembershipListRequest::GetConfirmed => {
                        let test_confirmed: Vec<Node> = Vec::with_capacity(0);

                        test_response
                            .send(MembershipListResponse::Confirmed(test_confirmed))
                            .unwrap();
                    }
                    MembershipListRequest::GetSuspected => {
                        let test_suspected: Vec<Node> = Vec::with_capacity(0);

                        test_response
                            .send(MembershipListResponse::Suspected(test_suspected))
                            .unwrap();
                    }
                    _ => panic!("send bytes ping test"),
                }
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        let test_socket_address = SocketAddr::from_str("0.0.0.0:25001").unwrap();
        let test_socket = UdpSocket::bind(test_socket_address).await.unwrap();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000").unwrap();

        let test_send_bytes = MembershipCommunications::send_bytes(
            &test_list_sender,
            &test_socket,
            b"ping",
            test_origin,
        )
        .await;

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

        let (test_list_sender, mut test_list_receiver) = mpsc::channel::<(
            MembershipListRequest,
            oneshot::Sender<MembershipListResponse>,
        )>(64);

        tokio::spawn(async move {
            while let Some((test_request, test_response)) = test_list_receiver.recv().await {
                match test_request {
                    MembershipListRequest::GetAlive => {
                        let test_alive: Vec<Node> = Vec::with_capacity(0);

                        test_response
                            .send(MembershipListResponse::Alive(test_alive))
                            .unwrap();
                    }
                    MembershipListRequest::GetConfirmed => {
                        let test_confirmed: Vec<Node> = Vec::with_capacity(0);

                        test_response
                            .send(MembershipListResponse::Confirmed(test_confirmed))
                            .unwrap();
                    }
                    MembershipListRequest::GetSuspected => {
                        let test_suspected: Vec<Node> = Vec::with_capacity(0);

                        test_response
                            .send(MembershipListResponse::Suspected(test_suspected))
                            .unwrap();
                    }
                    _ => panic!("send bytes ping test"),
                }
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        let test_socket_address = SocketAddr::from_str("0.0.0.0:25001").unwrap();
        let test_socket = UdpSocket::bind(test_socket_address).await.unwrap();
        let test_origin = SocketAddr::from_str("0.0.0.0:25000").unwrap();

        let test_send_bytes = MembershipCommunications::send_bytes(
            &test_list_sender,
            &test_socket,
            b"ping-req",
            test_origin,
        )
        .await;

        assert!(test_receiver.await.is_ok());
        assert!(test_send_bytes.is_ok());

        Ok(())
    }
}
