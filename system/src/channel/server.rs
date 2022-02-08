use tokio::sync::{broadcast, mpsc, watch};

// pub type ServerShutdown = mpsc::Receiver<bool>;
// pub type ServerShutdown = broadcast::Receiver<bool>;
// pub type ServerShutdown = watch::Receiver<bool>;
// pub type SendServerShutdown = mpsc::Sender<bool>;
pub type ServerShutdown = watch::Receiver<u8>;
pub type SendServerShutdown = mpsc::Sender<u8>;

// pub type ServerReceiver = broadcast::Receiver<ServerState>;
// pub type ServerSender = broadcast::Sender<ServerState>;
pub type ServerReceiver = mpsc::Receiver<ServerState>;
pub type ServerSender = mpsc::Sender<ServerState>;
// pub type ServerSender = broadcast::Sender<ServerState>;

// pub type CandidateReceiver = mpsc::Receiver<CandidateTransition>;
// pub type CandidateSender = mpsc::Sender<CandidateTransition>;
pub type CandidateReceiver = broadcast::Receiver<CandidateTransition>;
pub type CandidateSender = broadcast::Sender<CandidateTransition>;

// pub type LeaderReceiver = mpsc::Receiver<Leader>;
// pub type LeaderSender = mpsc::Sender<Leader>;
pub type LeaderReceiver = broadcast::Receiver<Leader>;
pub type LeaderSender = broadcast::Sender<Leader>;

#[derive(Clone, Debug, PartialEq)]
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
