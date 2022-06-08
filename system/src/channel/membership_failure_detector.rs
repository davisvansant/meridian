use std::net::SocketAddr;
use tokio::sync::{broadcast, watch};

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
    Member(SocketAddr),
}

pub async fn build() -> (
    MembershipFailureDetectorSender,
    MembershipFailureDetectorReceiver,
) {
    let (sender, receiver) = watch::channel::<MembershipFailureDetectorRequest>(
        MembershipFailureDetectorRequest::Shutdown,
    );

    (sender, receiver)
}

pub async fn build_ping_target() -> MembershipFailureDetectorPingTargetSender {
    let (sender, _receiver) = broadcast::channel::<MembershipFailureDetectorPingTarget>(1);

    sender
}

pub async fn launch(
    failure_detector: &MembershipFailureDetectorSender,
) -> Result<(), Box<dyn std::error::Error>> {
    failure_detector.send(MembershipFailureDetectorRequest::Launch)?;

    Ok(())
}
