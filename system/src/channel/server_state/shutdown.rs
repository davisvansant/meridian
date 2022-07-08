use tokio::sync::broadcast::{channel, Receiver, Sender};

pub type ShutdownReceiver = Receiver<ShutdownTask>;
pub type Transition = Sender<ShutdownTask>;

#[derive(Clone, Debug)]
pub enum ShutdownTask {
    Run,
}

#[derive(Clone, Debug)]
pub struct Shutdown {
    transition: Transition,
}

impl Shutdown {
    pub fn init() -> Shutdown {
        let (transition, _run) = channel::<ShutdownTask>(64);

        Shutdown { transition }
    }

    pub fn subscribe(&self) -> ShutdownReceiver {
        self.transition.subscribe()
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.transition.send(ShutdownTask::Run)?;

        Ok(())
    }
}
