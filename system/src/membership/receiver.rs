use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

use crate::channel::membership::failure_detector::{PingTarget, PingTargetSender};
use crate::channel::membership::list::ListChannel;
use crate::channel::membership::sender::{Dissemination, DisseminationSender};
use crate::channel::server_state::shutdown::Shutdown;
use crate::membership::Message;
use crate::{error, info, warn};

pub struct Receiver {
    udp_socket: Arc<UdpSocket>,
    list: ListChannel,
    dissemination: DisseminationSender,
    failure_detector: PingTargetSender,
    shutdown: Shutdown,
}

impl Receiver {
    pub async fn init(
        udp_socket: Arc<UdpSocket>,
        list: ListChannel,
        dissemination: DisseminationSender,
        failure_detector: PingTargetSender,
        shutdown: Shutdown,
    ) -> Receiver {
        info!("initialized!");

        Receiver {
            udp_socket,
            list,
            dissemination,
            failure_detector,
            shutdown,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];
        let mut shutdown = self.shutdown.subscribe();

        info!("running!");

        loop {
            tokio::select! {
                biased;
                _ = shutdown.recv() => {
                    info!("shutting down...");

                    break
                }
                result = self.udp_socket.recv_from(&mut buffer) => {
                    match result {
                        Ok((bytes, origin)) => {
                            let udp_message = UdpMessage::init(
                                self.list.to_owned(),
                                self.dissemination.to_owned(),
                                self.failure_detector.to_owned(),
                            )
                            .await;

                            tokio::spawn(async move {
                                if let Err(error) = udp_message.process(&buffer[..bytes], origin).await {
                                    error!("process udp message -> {:?}", error);
                                }
                            });
                        }
                        Err(error) => error!("receiving UDP message -> {:?}", error),
                    }
                }
            }
        }

        Ok(())
    }
}

struct UdpMessage {
    list: ListChannel,
    dissemination: DisseminationSender,
    failure_detector: PingTargetSender,
}

impl UdpMessage {
    async fn init(
        list: ListChannel,
        dissemination: DisseminationSender,
        failure_detector: PingTargetSender,
    ) -> UdpMessage {
        UdpMessage {
            list,
            dissemination,
            failure_detector,
        }
    }

    async fn process(
        &self,
        bytes: &[u8],
        origin: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (
            message,
            origin_node,
            suspect,
            peer_active_list,
            peer_suspected_list,
            peer_confirmed_list,
        ) = Message::from_list(bytes).await?;

        self.list.insert_alive(&origin_node).await?;

        for alive_member in &peer_active_list {
            self.list.remove_suspected(alive_member).await?;
            self.list.insert_alive(alive_member).await?;
        }

        for suspected_member in &peer_suspected_list {
            self.list.insert_suspected(suspected_member).await?;
            self.list.remove_alive(suspected_member).await?;
        }

        for confirmed_member in &peer_confirmed_list {
            self.list.remove_alive(confirmed_member).await?;
            self.list.remove_suspected(confirmed_member).await?;
            self.list.insert_confirmed(confirmed_member).await?;
        }

        match message {
            Message::Ack => {
                info!("received ack!");

                self.ack(origin, suspect).await?;
            }
            Message::Ping => {
                info!("received ping!");

                self.ping(origin, suspect).await?;
            }
            Message::PingReq => {
                info!("received ping request!");

                self.ping_req(suspect).await?;
            }
        }

        Ok(())
    }

    async fn ack(
        &self,
        origin: SocketAddr,
        suspect: Option<(SocketAddr, SocketAddr)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.failure_detector.receiver_count() > 0 {
            match suspect {
                Some((forward_address, suspect_address)) => {
                    let node = self.list.get_node().await?;
                    let local_alive_list = self.list.get_alive().await?;
                    let local_suspected_list = self.list.get_suspected().await?;
                    let local_confirmed_list = self.list.get_confirmed().await?;

                    let ack = Message::Ack
                        .build_list(
                            &node,
                            Some((&forward_address, &suspect_address)),
                            &local_alive_list,
                            &local_suspected_list,
                            &local_confirmed_list,
                        )
                        .await;

                    if node.membership_address().await == forward_address {
                        self.failure_detector
                            .send(PingTarget::Member(suspect_address))?;
                    } else {
                        self.dissemination
                            .send(Dissemination::Message(ack, forward_address))?;
                    }
                }
                None => {
                    self.failure_detector.send(PingTarget::Member(origin))?;
                }
            }
        }

        Ok(())
    }

    async fn ping(
        &self,
        origin: SocketAddr,
        _suspect: Option<(SocketAddr, SocketAddr)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let node = self.list.get_node().await?;
        let local_alive_list = self.list.get_alive().await?;
        let local_suspected_list = self.list.get_suspected().await?;
        let local_confirmed_list = self.list.get_confirmed().await?;

        let ack = Message::Ack
            .build_list(
                &node,
                None,
                &local_alive_list,
                &local_suspected_list,
                &local_confirmed_list,
            )
            .await;

        self.dissemination
            .send(Dissemination::Message(ack, origin))?;

        Ok(())
    }

    async fn ping_req(
        &self,
        suspect: Option<(SocketAddr, SocketAddr)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let node = self.list.get_node().await?;
        let local_alive_list = self.list.get_alive().await?;
        let local_suspected_list = self.list.get_suspected().await?;
        let local_confirmed_list = self.list.get_confirmed().await?;

        if let Some((forward_address, suspect_address)) = suspect {
            let ping = Message::Ping
                .build_list(
                    &node,
                    Some((&forward_address, &suspect_address)),
                    &local_alive_list,
                    &local_suspected_list,
                    &local_confirmed_list,
                )
                .await;

            warn!("sending ping -> {:?}", &ping);
            warn!("suspect address -> {:?}", &suspect_address);

            self.dissemination
                .send(Dissemination::Message(ping, suspect_address))?;
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
        let test_socket_address = SocketAddr::from_str("127.0.0.1:25000")?;

        let test_udp_socket = UdpSocket::bind(test_socket_address).await?;
        let test_receiving_udp_socket = Arc::new(test_udp_socket);
        let _test_sending_udp_socket = test_receiving_udp_socket.clone();

        let (test_list_sender, _test_list_receiver) = ListChannel::init().await;
        let test_dissemination = Dissemination::build().await;
        let test_failure_detector = PingTarget::build().await;
        let test_shutdown_signal = Shutdown::init();

        let test_receiver = Receiver::init(
            test_receiving_udp_socket,
            test_list_sender,
            test_dissemination,
            test_failure_detector,
            test_shutdown_signal,
        )
        .await;

        assert_eq!(
            &test_receiver.udp_socket.local_addr()?.to_string(),
            "127.0.0.1:25000",
        );
        assert_eq!(test_receiver.dissemination.receiver_count(), 0);
        assert_eq!(test_receiver.failure_detector.receiver_count(), 0);

        let test_udp_message = UdpMessage::init(
            test_receiver.list.to_owned(),
            test_receiver.dissemination.to_owned(),
            test_receiver.failure_detector.to_owned(),
        )
        .await;

        assert_eq!(test_udp_message.dissemination.receiver_count(), 0);
        assert_eq!(test_udp_message.failure_detector.receiver_count(), 0);

        Ok(())
    }
}
