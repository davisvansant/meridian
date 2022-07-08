use tokio::sync::mpsc::{channel, Receiver, Sender};

pub type EnterState = Receiver<LeaderTask>;
pub type Transition = Sender<LeaderTask>;

#[derive(Debug)]
pub enum LeaderTask {
    Run,
}

pub struct LeaderState {
    transition: Transition,
}

impl LeaderState {
    pub async fn init() -> (LeaderState, EnterState) {
        let (transition, enter_state) = channel::<LeaderTask>(64);

        (LeaderState { transition }, enter_state)
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.transition.send(LeaderTask::Run).await?;

        Ok(())
    }
}
