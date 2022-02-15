use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{sleep, Duration};

pub struct FailureDectector {
    protocol_period: Duration,
}

impl FailureDectector {
    pub async fn init() -> FailureDectector {
        let protocol_period = Duration::from_secs(10);

        FailureDectector { protocol_period }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut signal = signal(SignalKind::interrupt())?;

        loop {
            tokio::select! {
                biased;
                _ = signal.recv() => {
                    println!("shutting down failure dectector...");

                    break
                }
                result = self.probe() => {
                    match result {
                        Ok(()) => println!("probe complete!"),
                        Err(error) => println!("probe failed with error -> {:?}", error),
                    }
                }
            }
        }

        Ok(())
    }

    async fn probe(&self) -> Result<(), Box<dyn std::error::Error>> {
        sleep(self.protocol_period).await;

        Ok(())
    }
}
