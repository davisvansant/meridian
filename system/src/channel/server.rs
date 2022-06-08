use tokio::sync::broadcast;

pub type CandidateReceiver = broadcast::Receiver<CandidateTransition>;
pub type CandidateSender = broadcast::Sender<CandidateTransition>;

pub type LeaderReceiver = broadcast::Receiver<Leader>;
pub type LeaderSender = broadcast::Sender<Leader>;

#[derive(Clone, Debug)]
pub enum CandidateTransition {
    Follower,
    Leader,
}

#[derive(Clone, Debug)]
pub enum Leader {
    Heartbeat,
}

pub async fn build_candidate_transition() -> CandidateSender {
    let (candidate_sender, _candidate_receiver) = broadcast::channel::<CandidateTransition>(64);

    candidate_sender
}

pub async fn build_leader_heartbeat() -> LeaderSender {
    let (leader_sender, _leader_receiver) = broadcast::channel::<Leader>(64);

    leader_sender
}
