use tokio::sync::{mpsc, oneshot};

pub type ServerReceiver = mpsc::Receiver<ServerState>;
pub type ServerSender = mpsc::Sender<ServerState>;

pub type CandidateReceiver = mpsc::Receiver<CandidateTransition>;
pub type CandidateSender = mpsc::Sender<CandidateTransition>;

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