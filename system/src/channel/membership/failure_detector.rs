use std::net::SocketAddr;
use tokio::sync::{broadcast, mpsc};

pub type PingTargetSender = broadcast::Sender<PingTarget>;

pub type EnterState = mpsc::Receiver<FailureDetectorComponent>;
pub type FailureDetectorTask = mpsc::Sender<FailureDetectorComponent>;

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

#[derive(Clone, Debug)]
pub enum FailureDetectorComponent {
    Run,
}

pub struct FailureDetector {
    task: FailureDetectorTask,
}

impl FailureDetector {
    pub async fn init() -> (FailureDetector, EnterState) {
        let (task, enter_state) = mpsc::channel::<FailureDetectorComponent>(64);

        (FailureDetector { task }, enter_state)
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.task.send(FailureDetectorComponent::Run).await?;

        Ok(())
    }
}
