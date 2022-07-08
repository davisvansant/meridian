use tokio::sync::mpsc::{channel, Receiver, Sender};

pub type EnterState = Receiver<ServerState>;
pub type Transition = Sender<ServerState>;

pub mod candidate;
pub mod follower;
pub mod leader;
pub mod preflight;
pub mod shutdown;

#[derive(Debug, PartialEq)]
pub enum ServerState {
    Preflight,
    Follower,
    Candidate,
    Leader,
    Shutdown,
}

#[derive(Clone, Debug)]
pub struct ServerStateChannel {
    transition: Transition,
}

impl ServerStateChannel {
    pub async fn init() -> (ServerStateChannel, EnterState) {
        let (transition, enter_state) = channel::<ServerState>(64);

        (ServerStateChannel { transition }, enter_state)
    }

    pub async fn preflight(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.transition.send(ServerState::Preflight).await?;

        Ok(())
    }

    pub async fn follower(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.transition.send(ServerState::Follower).await?;

        Ok(())
    }

    pub async fn candidate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.transition.send(ServerState::Candidate).await?;

        Ok(())
    }

    pub async fn leader(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.transition.send(ServerState::Leader).await?;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.transition.send(ServerState::Shutdown).await?;

        Ok(())
    }
}
