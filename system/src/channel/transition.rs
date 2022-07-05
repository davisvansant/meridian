use tokio::sync::{broadcast, mpsc};

pub type PreflightStateReceiver = mpsc::Receiver<PreflightState>;
pub type PreflightStateSender = mpsc::Sender<PreflightState>;

pub type FollowerStateReceiver = mpsc::Receiver<FollowerState>;
pub type FollowerStateSender = mpsc::Sender<FollowerState>;

pub type CandidateStateReceiver = mpsc::Receiver<CandidateState>;
pub type CandidateStateSender = mpsc::Sender<CandidateState>;

pub type LeaderStateReceiver = mpsc::Receiver<LeaderState>;
pub type LeaderStateSender = mpsc::Sender<LeaderState>;

pub type ShutdownReceiver = broadcast::Receiver<Shutdown>;
pub type ShutdownSender = broadcast::Sender<Shutdown>;

pub type TransitionReceiver = mpsc::Receiver<Transition>;
pub type TransitionSender = mpsc::Sender<Transition>;

#[derive(Debug)]
pub enum PreflightState {
    Run,
}

impl PreflightState {
    pub async fn build() -> (PreflightStateSender, PreflightStateReceiver) {
        let (preflight_state_sender, preflight_state_receiver) =
            mpsc::channel::<PreflightState>(64);

        (preflight_state_sender, preflight_state_receiver)
    }
}

#[derive(Debug)]
pub enum FollowerState {
    Run,
}

impl FollowerState {
    pub async fn build() -> (FollowerStateSender, FollowerStateReceiver) {
        let (follower_state_sender, follower_state_receiver) = mpsc::channel::<FollowerState>(64);

        (follower_state_sender, follower_state_receiver)
    }
}

#[derive(Debug)]
pub enum CandidateState {
    Run,
}

impl CandidateState {
    pub async fn build() -> (CandidateStateSender, CandidateStateReceiver) {
        let (candidate_state_sender, candidate_state_receiver) =
            mpsc::channel::<CandidateState>(64);

        (candidate_state_sender, candidate_state_receiver)
    }
}

#[derive(Debug)]
pub enum LeaderState {
    Run,
}

impl LeaderState {
    pub async fn build() -> (LeaderStateSender, LeaderStateReceiver) {
        let (leader_state_sender, leader_state_receiver) = mpsc::channel::<LeaderState>(64);

        (leader_state_sender, leader_state_receiver)
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
pub enum Transition {
    PreflightState,
    FollowerState,
    CandidateState,
    LeaderState,
    Shutdown,
}

impl Transition {
    pub async fn build() -> (TransitionSender, TransitionReceiver) {
        let (transition_sender, transition_recever) = mpsc::channel::<Transition>(64);

        (transition_sender, transition_recever)
    }
}
