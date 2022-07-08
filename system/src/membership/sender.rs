use std::sync::Arc;
use tokio::net::UdpSocket;

use crate::channel::membership::sender::{Dissemination, DisseminationSender};
use crate::channel::server_state::shutdown::Shutdown;
use crate::info;

pub struct Sender {
    udp_socket: Arc<UdpSocket>,
    component: DisseminationSender,
    shutdown: Shutdown,
}

impl Sender {
    pub async fn init(
        udp_socket: Arc<UdpSocket>,
        component: DisseminationSender,
        shutdown: Shutdown,
    ) -> Sender {
        info!("initialized!");

        Sender {
            udp_socket,
            component,
            shutdown,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut shutdown = self.shutdown.subscribe();
        let mut component = self.component.subscribe();

        loop {
            tokio::select! {
                biased;
                _ = shutdown.recv() => {
                    info!("shutting down...");

                    break
                }
                Ok(Dissemination::Message(bytes, target)) = component.recv() => {
                    info!("sending UDP message -> {:?}", &bytes);
                    info!("target -> {:?}", &target);

                    self.udp_socket.send_to(&bytes, target).await?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_socket_address = SocketAddr::from_str("127.0.0.1:25000")?;

        let test_udp_socket = UdpSocket::bind(test_socket_address).await?;
        let test_receiving_udp_socket = Arc::new(test_udp_socket);
        let test_sending_udp_socket = test_receiving_udp_socket.clone();

        let test_disseminiation = Dissemination::build().await;
        let test_shutdown_signal = Shutdown::init();

        let test_sender = Sender::init(
            test_sending_udp_socket,
            test_disseminiation,
            test_shutdown_signal,
        )
        .await;

        assert_eq!(
            &test_sender.udp_socket.local_addr()?.to_string(),
            "127.0.0.1:25000",
        );
        assert_eq!(test_sender.component.receiver_count(), 0);

        Ok(())
    }
}
