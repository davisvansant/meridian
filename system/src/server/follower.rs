use rand::{thread_rng, Rng};
use tokio::time::{timeout_at, Duration, Instant};

use crate::channel::server::{Leader, LeaderHeartbeatSender};
use crate::channel::server_state::follower::EnterState;
use crate::channel::server_state::shutdown::Shutdown;
use crate::channel::server_state::ServerStateChannel;
use crate::{error, info, warn};

pub struct Follower {
    election_timeout: Duration,
    enter_state: EnterState,
    leader_heartbeat: LeaderHeartbeatSender,
    shutdown: Shutdown,
    exit_state: ServerStateChannel,
}

impl Follower {
    pub async fn init(
        enter_state: EnterState,
        leader_heartbeat: LeaderHeartbeatSender,
        shutdown: Shutdown,
        exit_state: ServerStateChannel,
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
                            Ok(Ok(Leader::Heartbeat)) => {
                                    info!("receiving heartbeat...");
                            }
                            Ok(Err(error)) => {
                                error!("there was an error! -> {:?}", error);

                                break;
                            }
                            Err(error) => {
                                warn!("timeout ending... starting election! {:?}", error);

                                self.exit_state.candidate().await?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::server_state::follower::FollowerState;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_transition, test_receive) = FollowerState::init().await;
        let test_leader_heartbeat_sender = Leader::build().await;
        let test_shutdown_signal = Shutdown::init();
        let (test_server_transition_state_sender, _test_server_transition_state_receiver) =
            ServerStateChannel::init().await;

        let test_follower = Follower::init(
            test_receive,
            test_leader_heartbeat_sender,
            test_shutdown_signal,
            test_server_transition_state_sender,
        )
        .await?;

        assert!(test_follower.election_timeout.as_millis() >= 15000);
        assert!(test_follower.election_timeout.as_millis() <= 30000);
        assert_eq!(test_follower.leader_heartbeat.receiver_count(), 0);

        Ok(())
    }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn run() -> Result<(), Box<dyn std::error::Error>> {
    //     let (test_transition, test_receive) = transition::Follower::build().await;
    //     let test_leader_heartbeat_sender = server::Leader::build().await;
    //     let test_shutdown_signal = transition::Shutdown::build().await;
    //     let (test_server_transition_state_sender, mut test_server_transition_state_receiver) =
    //         transition::ServerState::build().await;

    //     let mut test_follower = Follower::init(
    //         test_receive,
    //         test_leader_heartbeat_sender.to_owned(),
    //         test_shutdown_signal,
    //         test_server_transition_state_sender,
    //     )
    //     .await?;

    //     tokio::spawn(async move {
    //         test_follower.run().await.unwrap();
    //     });

    //     test_transition.send(transition::Follower::Run).await?;

    //     let mut test_heartbeats = 0;

    //     while test_heartbeats <= 1 {
    //         tokio::time::sleep(Duration::from_millis(10000)).await;

    //         test_leader_heartbeat_sender.send(server::Leader::Heartbeat)?;

    //         test_heartbeats += 1;
    //     }

    //     match test_server_transition_state_receiver.recv().await {
    //         Some(test_server_transition) => {
    //             assert_eq!(test_server_transition, transition::ServerState::Candidate);
    //         }
    //         None => panic!("expected transition state"),
    //     }

    //     Ok(())
    // }
}
