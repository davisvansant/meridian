use tokio::sync::{broadcast, mpsc};

pub type PreflightReceiver = mpsc::Receiver<Preflight>;
pub type PreflightSender = mpsc::Sender<Preflight>;

pub type FollowerReceiver = mpsc::Receiver<Follower>;
pub type FollowerSender = mpsc::Sender<Follower>;

pub type CandidateReceiver = mpsc::Receiver<Candidate>;
pub type CandidateSender = mpsc::Sender<Candidate>;

pub type LeaderReceiver = mpsc::Receiver<Leader>;
pub type LeaderSender = mpsc::Sender<Leader>;

pub type ShutdownReceiver = broadcast::Receiver<Shutdown>;
pub type ShutdownSender = broadcast::Sender<Shutdown>;

pub type ServerStateReceiver = mpsc::Receiver<ServerState>;
pub type ServerStateSender = mpsc::Sender<ServerState>;

#[derive(Debug)]
pub enum Preflight {
    Run,
}

impl Preflight {
    pub async fn build() -> (PreflightSender, PreflightReceiver) {
        let (preflight_sender, preflight_receiver) = mpsc::channel::<Preflight>(64);

        (preflight_sender, preflight_receiver)
    }
}

#[derive(Debug)]
pub enum Follower {
    Run,
}

impl Follower {
    pub async fn build() -> (FollowerSender, FollowerReceiver) {
        let (follower_sender, follower_receiver) = mpsc::channel::<Follower>(64);

        (follower_sender, follower_receiver)
    }
}

#[derive(Debug)]
pub enum Candidate {
    Run,
}

impl Candidate {
    pub async fn build() -> (CandidateSender, CandidateReceiver) {
        let (candidate_sender, candidate_receiver) = mpsc::channel::<Candidate>(64);

        (candidate_sender, candidate_receiver)
    }
}

#[derive(Debug)]
pub enum Leader {
    Run,
}

impl Leader {
    pub async fn build() -> (LeaderSender, LeaderReceiver) {
        let (leader_sender, leader_receiver) = mpsc::channel::<Leader>(64);

        (leader_sender, leader_receiver)
    }
}

#[derive(Clone, Debug)]
pub enum Shutdown {
    Run,
}

impl Shutdown {
    pub async fn build() -> ShutdownSender {
        let (shutdown_sender, _shutdown_receiver) = broadcast::channel::<Shutdown>(64);

        shutdown_sender
    }

    pub async fn send(shutdown: &ShutdownSender) -> Result<(), Box<dyn std::error::Error>> {
        shutdown.send(Shutdown::Run)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum ServerState {
    Candidate,
    Follower,
    Leader,
    Preflight,
    Shutdown,
}

impl ServerState {
    pub async fn build() -> (ServerStateSender, ServerStateReceiver) {
        let (server_state_sender, server_state_recever) = mpsc::channel::<ServerState>(64);

        (server_state_sender, server_state_recever)
    }
}
