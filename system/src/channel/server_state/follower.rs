use tokio::sync::mpsc::{channel, Receiver, Sender};

pub type EnterState = Receiver<FollowerTask>;
pub type Transition = Sender<FollowerTask>;

#[derive(Debug)]
pub enum FollowerTask {
    Run,
}

pub struct FollowerState {
    transition: Transition,
}

impl FollowerState {
    pub async fn init() -> (FollowerState, EnterState) {
        let (transition, enter_state) = channel::<FollowerTask>(64);

        (FollowerState { transition }, enter_state)
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.transition.send(FollowerTask::Run).await?;

        Ok(())
    }
}
