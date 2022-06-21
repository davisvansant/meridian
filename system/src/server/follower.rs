use rand::{thread_rng, Rng};
use tokio::time::{timeout_at, Duration, Instant};

use crate::channel::{server, transition};
use crate::{error, info, warn};

pub struct Follower {
    election_timeout: Duration,
    enter_state: transition::FollowerReceiver,
    leader_heartbeat: server::LeaderSender,
    shutdown: transition::ShutdownSender,
    exit_state: transition::ServerStateSender,
}

impl Follower {
    pub async fn init(
        enter_state: transition::FollowerReceiver,
        leader_heartbeat: server::LeaderSender,
        shutdown: transition::ShutdownSender,
        exit_state: transition::ServerStateSender,
    ) -> Result<Follower, Box<dyn std::error::Error>> {
        let mut rng = thread_rng();

        let election_timeout =
            rng.gen_range(Duration::from_millis(15000)..Duration::from_millis(30000));

        info!("election timeoute value -> {:?}", &election_timeout);

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
                            Ok(Ok(server::Leader::Heartbeat)) => {
                                    info!("receiving heartbeat...");
                            }
                            Ok(Err(error)) => {
                                error!("there was an error! -> {:?}", error);

                                break;
                            }
                            Err(error) => {
                                warn!("timeout ending... starting election! {:?}", error);

                                self.exit_state.send(transition::ServerState::Candidate).await?;

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
