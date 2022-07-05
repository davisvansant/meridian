use tokio::sync::{broadcast, mpsc};

pub type ElectionResultReceiver = mpsc::Receiver<ElectionResult>;
pub type ElectionResultSender = mpsc::Sender<ElectionResult>;

pub type LeaderHeartbeatSender = broadcast::Sender<Leader>;

#[derive(Clone, Debug)]
pub enum ElectionResult {
    Follower,
    Leader,
}

impl ElectionResult {
    pub async fn build() -> (ElectionResultSender, ElectionResultReceiver) {
        let (election_result_sender, election_result_receiver) =
            mpsc::channel::<ElectionResult>(64);

        (election_result_sender, election_result_receiver)
    }
}

#[derive(Clone, Debug)]
pub enum Leader {
    Heartbeat,
}

impl Leader {
    pub async fn build() -> LeaderHeartbeatSender {
        let (leader_sender, _leader_receiver) = broadcast::channel::<Leader>(64);

        leader_sender
    }
}
