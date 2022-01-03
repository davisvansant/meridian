use tokio::time::Duration;

use crate::server::ServerState;

use tokio::time::timeout_at;

use tokio::time::Instant;

use tokio::sync::broadcast::Receiver;
// use tokio::sync::mpsc::Receiver;

pub struct Follower {
    pub election_timeout: Duration,
}

impl Follower {
    pub async fn init() -> Result<Follower, Box<dyn std::error::Error>> {
        let election_timeout = Duration::from_millis(150);

        Ok(Follower { election_timeout })
    }

    // pub async fn election_timeout(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     println!("Waiting for leader...");
    //     Ok(())
    // }
    pub async fn run(
        &mut self,
        heartbeat: &mut Receiver<ServerState>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while let Ok(result) =
            timeout_at(Instant::now() + self.election_timeout, heartbeat.recv()).await
        {
            if let Ok(ServerState::Follower) = result {
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

    // #[tokio::test(flavor = "multi_thread")]
    // async fn election_timeout() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_follower = Follower::init().await?;
    //     assert!(test_follower.election_timeout().await.is_ok());
    //     Ok(())
    // }
}
