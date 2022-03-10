use std::net::SocketAddr;
use tokio::sync::watch;
use tokio::sync::{broadcast, mpsc};

use crate::node::Node;

// pub type MembershipFailureDetectorReceiver = mpsc::Receiver<MembershipFailureDetectorRequest>;
// pub type MembershipFailureDetectorSender = mpsc::Sender<MembershipFailureDetectorRequest>;
pub type MembershipFailureDetectorReceiver = watch::Receiver<MembershipFailureDetectorRequest>;
pub type MembershipFailureDetectorSender = watch::Sender<MembershipFailureDetectorRequest>;

pub type MembershipFailureDetectorPingTargetSender =
    broadcast::Sender<MembershipFailureDetectorPingTarget>;

#[derive(Clone, Debug)]
pub enum MembershipFailureDetectorRequest {
    Shutdown,
    Launch,
}

#[derive(Clone, Debug)]
pub enum MembershipFailureDetectorPingTarget {
    // Member(Node),
    Member(SocketAddr),
}

pub async fn build_failure_detector_channel() -> (
    MembershipFailureDetectorSender,
    MembershipFailureDetectorReceiver,
) {
    let (sender, receiver) = watch::channel::<MembershipFailureDetectorRequest>(
        MembershipFailureDetectorRequest::Shutdown,
    );

    (sender, receiver)
}

pub async fn build_failure_detector_ping_target_channel(
) -> MembershipFailureDetectorPingTargetSender {
    let (sender, _receiver) = broadcast::channel::<MembershipFailureDetectorPingTarget>(1);

    sender
}

pub async fn launch_failure_detector(
    failure_detector: &MembershipFailureDetectorSender,
) -> Result<(), Box<dyn std::error::Error>> {
    failure_detector.send(MembershipFailureDetectorRequest::Launch)?;

    Ok(())
}
