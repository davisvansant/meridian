use std::net::SocketAddr;
use tokio::sync::{broadcast, mpsc};

pub type MembershipFailureDetectorPingTargetSender =
    broadcast::Sender<MembershipFailureDetectorPingTarget>;

pub type FailureDetectorReceiver = mpsc::Receiver<FailureDetector>;
pub type FailureDetectorSender = mpsc::Sender<FailureDetector>;

#[derive(Clone, Debug)]
pub enum MembershipFailureDetectorPingTarget {
    Member(SocketAddr),
}

pub async fn build_ping_target() -> MembershipFailureDetectorPingTargetSender {
    let (sender, _receiver) = broadcast::channel::<MembershipFailureDetectorPingTarget>(64);

    sender
}

#[derive(Debug)]
pub enum FailureDetector {
    Run,
}

impl FailureDetector {
    pub async fn build() -> (FailureDetectorSender, FailureDetectorReceiver) {
        let (failure_detector_sender, failure_detector_receiver) =
            mpsc::channel::<FailureDetector>(64);

        (failure_detector_sender, failure_detector_receiver)
    }
}
