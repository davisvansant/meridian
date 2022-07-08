use tokio::time::{sleep, Duration};

use crate::channel::membership::MembershipChannel;
use crate::channel::server::LeaderHeartbeatSender;
use crate::channel::server_state::candidate::CandidateState;
use crate::channel::server_state::follower::FollowerState;
use crate::channel::server_state::leader::LeaderState;
use crate::channel::server_state::preflight::PreflightState;
use crate::channel::server_state::shutdown::Shutdown;
use crate::channel::server_state::{ServerState, ServerStateChannel};
use crate::channel::state::StateChannel;
use crate::{error, info};

use candidate::Candidate;
use follower::Follower;
use leader::Leader;
use preflight::Preflight;

mod candidate;
mod follower;
mod leader;
mod preflight;

pub struct Server {
    membership: MembershipChannel,
    state: StateChannel,
    leader_heartbeat: LeaderHeartbeatSender,
    shutdown: Shutdown,
}

impl Server {
    pub async fn init(
        membership: MembershipChannel,
        state: StateChannel,
        leader_heartbeat: LeaderHeartbeatSender,
        shutdown: Shutdown,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        Ok(Server {
            membership,
            state,
            leader_heartbeat,
            shutdown,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut server_shutdown = self.shutdown.subscribe();

        let (server_state_transition, mut server_enter_state) = ServerStateChannel::init().await;

        sleep(Duration::from_secs(5)).await;

        server_state_transition.preflight().await?;

        loop {
            tokio::select! {
                biased;
                _ = server_shutdown.recv() => {
                    info!("server shutting down!");

                    self.run_shutdown().await?;

                    break;
                }

                server_state = server_enter_state.recv() => {
                    match server_state {
                        Some(ServerState::Preflight) => {
                            info!("running preflight tasks");

                            let (task, enter_state) = PreflightState::init().await;

                            let mut preflight = Preflight::init(
                                enter_state,
                                server_state_transition.to_owned(),
                                self.shutdown.to_owned(),
                                self.membership.to_owned(),
                            )
                            .await?;

                            tokio::spawn(async move {
                                if let Err(error) = preflight.run().await {
                                    error!("preflight error -> {:?}", error);
                                }
                            });

                            sleep(Duration::from_secs(5)).await;

                            task.run().await?;
                        }
                        Some(ServerState::Follower) => {
                            info!("transitioning to follower");

                            let (task, enter_state) = FollowerState::init().await;

                            let mut follower = Follower::init(
                                enter_state,
                                self.leader_heartbeat.to_owned(),
                                self.shutdown.to_owned(),
                                server_state_transition.to_owned(),
                            )
                            .await?;

                            tokio::spawn(async move {
                                if let Err(error) = follower.run().await {
                                    error!("follower error -> {:?}", error);
                                }
                            });

                            task.run().await?;
                        }
                        Some(ServerState::Candidate) => {
                            info!("transitioning to candidate");

                            let (task, enter_state) = CandidateState::init().await;

                            let mut candidate = Candidate::init(
                                enter_state,
                                server_state_transition.to_owned(),
                                self.shutdown.to_owned(),
                                self.leader_heartbeat.to_owned(),
                                self.membership.to_owned(),
                                self.state.to_owned(),
                            )
                            .await?;

                            tokio::spawn(async move {
                                if let Err(error) = candidate.run().await {
                                    error!("candidate error -> {:?}", error);
                                }
                            });

                            task.run().await?;
                        }
                        Some(ServerState::Leader) => {
                            info!("transitioning to leader");

                            let (task, enter_state) = LeaderState::init().await;

                            let mut leader = Leader::init(
                                enter_state,
                                server_state_transition.to_owned(),
                                self.shutdown.to_owned(),
                                self.membership.to_owned(),
                                self.state.to_owned(),
                            )
                            .await?;

                            tokio::spawn(async move {
                                if let Err(error) = leader.run().await {
                                    error!("leader error -> {:?}", error);
                                }
                            });

                            task.run().await?;
                        }
                        Some(ServerState::Shutdown) => {
                            info!("shutting down!");

                            self.run_shutdown().await?;

                            break;
                        }
                        None => info!("no state transition found!"),
                    }
                }
            }
        }

        Ok(())
    }

    async fn run_shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.shutdown.run()?;
        self.state.shutdown().await?;
        self.membership.shutdown().await?;

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test(flavor = "multi_thread")]
//     async fn init() -> Result<(), Box<dyn std::error::Error>> {
//         let (test_client_sender, _test_client_receiver) = crate::channel::rpc_client::build().await;
//         let (test_membership_sender, _test_membership_receiver) =
//             crate::channel::membership::build().await;
//         let (test_state_sender, _test_state_receiver) = crate::channel::state::build().await;
//         let test_candidate_sender = crate::channel::server::build_candidate_transition().await;
//         let _test_candidate_receiver = test_candidate_sender.subscribe();
//         let test_leader_sender = crate::channel::server::build_leader_heartbeat().await;
//         let _test_leader_receiver = test_leader_sender.subscribe();
//         let test_shutdown_sender = crate::channel::shutdown::build().await;

//         let test_server = Server::init(
//             test_client_sender,
//             test_membership_sender,
//             test_state_sender,
//             test_candidate_sender,
//             test_leader_sender,
//             test_shutdown_sender,
//         )
//         .await?;

//         assert!(!test_server.client.is_closed());
//         assert!(!test_server.membership.is_closed());
//         assert!(!test_server.state.is_closed());
//         assert_eq!(test_server.candidate_sender.receiver_count(), 1);
//         assert_eq!(test_server.heartbeat.receiver_count(), 1);

//         Ok(())
//     }
// }
