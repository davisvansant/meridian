use std::net::SocketAddr;
use tokio::sync::{broadcast, mpsc};

use crate::node::Node;

pub type MembershipFailureDetectorReceiver = mpsc::Receiver<MembershipFailureDetectorRequest>;
pub type MembershipFailureDetectorSender = mpsc::Sender<MembershipFailureDetectorRequest>;

pub type MembershipFailureDetectorPingTargetSender =
    broadcast::Sender<MembershipFailureDetectorPingTarget>;

#[derive(Clone, Debug)]
pub enum MembershipFailureDetectorRequest {
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
    let (sender, receiver) = mpsc::channel::<MembershipFailureDetectorRequest>(1);

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
    failure_detector
        .send(MembershipFailureDetectorRequest::Launch)
        .await?;

    Ok(())
}
