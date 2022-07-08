use tokio::sync::mpsc::{channel, Receiver, Sender};

pub type EnterState = Receiver<PreflightTask>;
pub type Transition = Sender<PreflightTask>;

#[derive(Debug)]
pub enum PreflightTask {
    Run,
}

pub struct PreflightState {
    transition: Transition,
}

impl PreflightState {
    pub async fn init() -> (PreflightState, EnterState) {
        let (transition, enter_state) = channel::<PreflightTask>(64);

        (PreflightState { transition }, enter_state)
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.transition.send(PreflightTask::Run).await?;

        Ok(())
    }
}
