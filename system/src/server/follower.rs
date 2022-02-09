use tokio::time::{timeout_at, Duration, Instant};

use crate::channel::{Leader, LeaderReceiver};

pub struct Follower {
    pub election_timeout: Duration,
}

impl Follower {
    pub async fn init() -> Result<Follower, Box<dyn std::error::Error>> {
        let election_timeout = Duration::from_millis(150);

        Ok(Follower { election_timeout })
    }

    pub async fn run(
        &mut self,
        heartbeat: &mut LeaderReceiver,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while let Ok(result) =
            timeout_at(Instant::now() + self.election_timeout, heartbeat.recv()).await
        {
            if let Ok(Leader::Heartbeat) = result {
                println!("receiving heartbeat...");
            }
        }

        println!("timeout ending...starting election");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_follower = Follower::init().await?;
        assert_eq!(test_follower.election_timeout.as_millis(), 150);
        Ok(())
    }
}
