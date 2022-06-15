use tokio::time::{timeout_at, Duration, Instant};

use crate::{error, info, warn};

pub struct Follower {
    pub election_timeout: Duration,
    enter_state: crate::channel::transition::FollowerReceiver,
    leader_heartbeat: crate::channel::server::LeaderSender,
    // shutdown: crate::channel::shutdown::ShutdownSender,
    shutdown: crate::channel::transition::ShutdownSender,
    exit_state: crate::channel::transition::ServerStateSender,
}

impl Follower {
    pub async fn init(
        enter_state: crate::channel::transition::FollowerReceiver,
        leader_heartbeat: crate::channel::server::LeaderSender,
        // shutdown: crate::channel::shutdown::ShutdownSender,
        shutdown: crate::channel::transition::ShutdownSender,
        exit_state: crate::channel::transition::ServerStateSender,
    ) -> Result<Follower, Box<dyn std::error::Error>> {
        let election_timeout = Duration::from_millis(30000);

        Ok(Follower {
            election_timeout,
            enter_state,
            leader_heartbeat,
            shutdown,
            exit_state,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut shutdown = self.shutdown.subscribe();
        let mut leader_heartbeat = self.leader_heartbeat.subscribe();

        loop {
            tokio::select! {
                biased;
                _ = shutdown.recv() => {
                    info!("shutting down!");

                    break;
                }

                Some(run) = self.enter_state.recv() => {
                    info!("follower -> {:?} | starting election timout....", run);

                    loop {
                        match timeout_at(Instant::now() + self.election_timeout, leader_heartbeat.recv()).await {
                            Ok(Ok(crate::channel::server::Leader::Heartbeat)) => {
                                    info!("receiving heartbeat...");
                            }
                            Ok(Err(error)) => {
                                error!("there was an error! -> {:?}", error);

                                break;
                            }
                            Err(error) => {
                                warn!("timeout ending... starting election! {:?}", error);

                                self.exit_state.send(crate::channel::transition::ServerState::Candidate).await?;

                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test(flavor = "multi_thread")]
//     async fn init() -> Result<(), Box<dyn std::error::Error>> {
//         let test_follower = Follower::init().await?;
//         assert_eq!(test_follower.election_timeout.as_millis(), 30000);
//         Ok(())
//     }
// }
