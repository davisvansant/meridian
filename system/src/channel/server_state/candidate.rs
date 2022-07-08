use tokio::sync::mpsc::{channel, Receiver, Sender};

pub type EnterState = Receiver<CandidateTask>;
pub type Transition = Sender<CandidateTask>;

#[derive(Debug)]
pub enum CandidateTask {
    Run,
}

pub struct CandidateState {
    transition: Transition,
}

impl CandidateState {
    pub async fn init() -> (CandidateState, EnterState) {
        let (transition, enter_state) = channel::<CandidateTask>(64);

        (CandidateState { transition }, enter_state)
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.transition.send(CandidateTask::Run).await?;

        Ok(())
    }
}
