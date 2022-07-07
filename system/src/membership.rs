use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

use crate::channel::membership::failure_detector::{FailureDetectorProtocol, PingTarget};
use crate::channel::membership::list::ListRequest;
use crate::channel::membership::sender::Dissemination;
use crate::channel::membership::{MembershipReceiver, MembershipRequest, MembershipResponse};
use crate::channel::transition::ShutdownSender;
use crate::node::Node;
use crate::{error, info};

use failure_detector::FailureDectector;
use list::List;
use message::Message;
use receiver::Receiver;
use sender::Sender;
use static_join::StaticJoin;

// mod communications;
mod failure_detector;
mod list;
mod message;
mod receiver;
mod sender;
mod static_join;

#[derive(Clone, Debug, PartialEq)]
pub enum ClusterSize {
    One,
    Three,
    Five,
}

impl ClusterSize {
    pub async fn from_str(size: &str) -> ClusterSize {
        match size {
            "1" => ClusterSize::One,
            "3" => ClusterSize::Three,
            "5" => ClusterSize::Five,
            _ => panic!("Expected a cluster size of 1, 3, or 5"),
        }
    }

    pub async fn majority(&self) -> usize {
        match self {
            ClusterSize::One => 0,
            ClusterSize::Three => 2,
            ClusterSize::Five => 3,
        }
    }
}

pub struct Membership {
    cluster_size: ClusterSize,
    receiver: MembershipReceiver,
    shutdown: ShutdownSender,
}

impl Membership {
    pub async fn init(
        cluster_size: ClusterSize,
        receiver: MembershipReceiver,
        shutdown: ShutdownSender,
    ) -> Result<Membership, Box<dyn std::error::Error>> {
        info!("initialized!");

        Ok(Membership {
            cluster_size,
            receiver,
            shutdown,
        })
    }

    pub async fn run(
        &mut self,
        server: Node,
        launch_nodes: Vec<SocketAddr>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (list_sender, list_receiver) = ListRequest::build().await;
        let failure_detector_ping_target_sender = PingTarget::build().await;
        let send_udp_message = Dissemination::build().await;
        let (failure_detector_sender, failure_detector_receiver) =
            FailureDetectorProtocol::build().await;
        let mut list = List::init(server, launch_nodes, list_receiver).await?;

        tokio::spawn(async move {
            if let Err(error) = list.run().await {
                error!("membership list -> {:?}", error);
            }
        });

        let port = server.membership_address().await;
        let socket = UdpSocket::bind(port).await?;
        let receiving_udp_socket = Arc::new(socket);
        let sending_udp_socket = receiving_udp_socket.clone();

        let mut receiver = Receiver::init(
            receiving_udp_socket,
            list_sender.to_owned(),
            send_udp_message.to_owned(),
            failure_detector_ping_target_sender.to_owned(),
            self.shutdown.to_owned(),
        )
        .await;

        tokio::spawn(async move {
            if let Err(error) = receiver.run().await {
                error!("member message receiver -> {:?}", error);
            }
        });

        let mut sender = Sender::init(
            sending_udp_socket,
            send_udp_message.to_owned(),
            self.shutdown.to_owned(),
        )
        .await;

        tokio::spawn(async move {
            if let Err(error) = sender.run().await {
                error!("membership message sender -> {:?}", error);
            }
        });

        let mut failure_detector = FailureDectector::init(
            list_sender.to_owned(),
            send_udp_message.to_owned(),
            failure_detector_ping_target_sender,
            failure_detector_receiver,
            self.shutdown.to_owned(),
        )
        .await;

        tokio::spawn(async move {
            if let Err(error) = failure_detector.run().await {
                error!("membership failure detector -> {:?}", error);
            }
        });

        let mut static_join =
            StaticJoin::init(send_udp_message.to_owned(), list_sender.to_owned()).await;

        drop(self.shutdown.to_owned());

        info!("running...");

        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                MembershipRequest::FailureDectector => {
                    failure_detector_sender
                        .send(FailureDetectorProtocol::Run)
                        .await?;
                }
                MembershipRequest::Members => {
                    info!("received members request!");

                    let members = ListRequest::get_alive(&list_sender).await?;

                    response.send(MembershipResponse::Members(members))?;
                }
                MembershipRequest::Node => {
                    let node = server;

                    response.send(MembershipResponse::Node(node))?;
                }
                MembershipRequest::StaticJoin => {
                    static_join.run().await?;

                    let alive = ListRequest::get_alive(&list_sender).await?;
                    let expected = self.cluster_size.majority().await;

                    response.send(MembershipResponse::Status((alive.len(), expected)))?
                }
                // MembershipRequest::Status => {
                //     let members = get_alive(&list_sender).await?;
                //     let connected_nodes = match members.len() {
                //         0 => 0,
                //         1 => 1,
                //         2 => 2,
                //         3 => 3,
                //         _ => panic!("unexpected number of peers!"),
                //     };

                //     if let Err(error) = response.send(MembershipResponse::Status(connected_nodes)) {
                //         println!("error sending membership response -> {:?}", error);
                //     }
                // }
                MembershipRequest::Shutdown => {
                    info!("shutting down...");

                    ListRequest::shutdown(&list_sender).await?;

                    self.receiver.close();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::membership::MembershipChannel;

    #[tokio::test(flavor = "multi_thread")]
    async fn cluster_size_one() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(ClusterSize::from_str("1").await, ClusterSize::One);
        assert_eq!(ClusterSize::One.majority().await, 0);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cluster_size_three() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(ClusterSize::from_str("3").await, ClusterSize::Three);
        assert_eq!(ClusterSize::Three.majority().await, 2);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cluster_size_five() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(ClusterSize::from_str("5").await, ClusterSize::Five);
        assert_eq!(ClusterSize::Five.majority().await, 3);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init_one() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = MembershipChannel::init().await;
        let test_shutdown_signal = crate::channel::transition::Shutdown::build().await;
        let test_membership =
            Membership::init(ClusterSize::One, test_receiver, test_shutdown_signal).await?;

        assert_eq!(test_membership.cluster_size, ClusterSize::One);
        assert_eq!(test_membership.shutdown.receiver_count(), 0);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init_three() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = MembershipChannel::init().await;
        let test_shutdown_signal = crate::channel::transition::Shutdown::build().await;
        let test_membership =
            Membership::init(ClusterSize::Three, test_receiver, test_shutdown_signal).await?;

        assert_eq!(test_membership.cluster_size, ClusterSize::Three);
        assert_eq!(test_membership.shutdown.receiver_count(), 0);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init_five() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = MembershipChannel::init().await;
        let test_shutdown_signal = crate::channel::transition::Shutdown::build().await;
        let test_membership =
            Membership::init(ClusterSize::Five, test_receiver, test_shutdown_signal).await?;

        assert_eq!(test_membership.cluster_size, ClusterSize::Five);
        assert_eq!(test_membership.shutdown.receiver_count(), 0);

        Ok(())
    }
}
