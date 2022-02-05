use tokio::sync::{broadcast, mpsc};

pub type ServerShutdown = mpsc::Receiver<()>;

// pub type ServerReceiver = broadcast::Receiver<ServerState>;
// pub type ServerSender = broadcast::Sender<ServerState>;
pub type ServerReceiver = mpsc::Receiver<ServerState>;
pub type ServerSender = mpsc::Sender<ServerState>;

pub type CandidateReceiver = mpsc::Receiver<CandidateTransition>;
pub type CandidateSender = mpsc::Sender<CandidateTransition>;

pub type LeaderReceiver = mpsc::Receiver<Leader>;
pub type LeaderSender = mpsc::Sender<Leader>;

#[derive(Clone, Debug)]
pub enum ServerState {
    Candidate,
    Follower,
    Leader,
    Preflight,
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum CandidateTransition {
    Follower,
    Leader,
}

#[derive(Clone, Debug)]
pub enum Leader {
    Heartbeat,
}
