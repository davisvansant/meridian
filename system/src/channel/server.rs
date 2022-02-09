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
