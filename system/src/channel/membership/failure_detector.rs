use std::net::SocketAddr;
use tokio::sync::{broadcast, mpsc};

pub type PingTargetSender = broadcast::Sender<PingTarget>;

pub type FailureDetectorProtocolReceiver = mpsc::Receiver<FailureDetectorProtocol>;
pub type FailureDetectorProtocolSender = mpsc::Sender<FailureDetectorProtocol>;

#[derive(Clone, Debug)]
pub enum PingTarget {
    Member(SocketAddr),
}

impl PingTarget {
    pub async fn build() -> PingTargetSender {
        let (sender, _receiver) = broadcast::channel::<PingTarget>(64);

        sender
    }
}

#[derive(Debug)]
pub enum FailureDetectorProtocol {
    Run,
}

impl FailureDetectorProtocol {
    pub async fn build() -> (
        FailureDetectorProtocolSender,
        FailureDetectorProtocolReceiver,
    ) {
        let (sender, receiver) = mpsc::channel::<FailureDetectorProtocol>(64);

        (sender, receiver)
    }
}
