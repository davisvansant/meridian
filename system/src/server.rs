use tokio::time::{sleep, Duration};

use crate::channel::membership::MembershipSender;
// use crate::channel::rpc_client::RpcClientSender;
// use crate::channel::shutdown::ShutdownSender;
use crate::channel::state::StateSender;
use crate::channel::transition::ShutdownSender;
// use crate::server::candidate::Candidate;
// use crate::server::follower::Follower;
// use crate::server::leader::Leader;
use crate::{error, info, warn};

use candidate::Candidate;
use follower::Follower;
use leader::Leader;
use preflight::Preflight;

mod candidate;
mod follower;
mod leader;
mod preflight;

pub struct Server {
    membership: MembershipSender,
    state: StateSender,
    leader_heartbeat: crate::channel::server::LeaderSender,
    shutdown: ShutdownSender,
    transition: crate::channel::transition::ServerStateReceiver,
}

impl Server {
    pub async fn init(
        membership: MembershipSender,
        state: StateSender,
        leader_heartbeat: crate::channel::server::LeaderSender,
        shutdown: ShutdownSender,
        transition: crate::channel::transition::ServerStateReceiver,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        Ok(Server {
            membership,
            state,
            leader_heartbeat,
            shutdown,
            transition,
        })
    }

    pub async fn run(
        &mut self,
        send_transition: &crate::channel::transition::ServerStateSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut server_shutdown = self.shutdown.subscribe();

        sleep(Duration::from_secs(5)).await;

        send_transition
            .send(crate::channel::transition::ServerState::Preflight)
            .await?;

        loop {
            tokio::select! {
                biased;
                _ = server_shutdown.recv() => {
                    info!("server shutting down!");

                    self.run_shutdown().await?;

                    break;
                }

                server_state = self.transition.recv() => {
                    match server_state {
                        Some(crate::channel::transition::ServerState::Preflight) => {
                            info!("running preflight tasks");

                            let (preflight_sender, preflight_receiver) =
                                crate::channel::transition::Preflight::build().await;

                            let preflight_server_state_transition_sender = send_transition.to_owned();
                            let preflight_shutdown = self.shutdown.to_owned();
                            let preflight_membership_sender = self.membership.to_owned();

                            let mut preflight = Preflight::init(
                                preflight_receiver,
                                preflight_server_state_transition_sender,
                                preflight_shutdown,
                                preflight_membership_sender,
                            )
                            .await?;

                            tokio::spawn(async move {
                                if let Err(error) = preflight.run().await {
                                    error!("preflight error -> {:?}", error);
                                }
                            });

                            sleep(Duration::from_secs(5)).await;

                            preflight_sender.send(crate::channel::transition::Preflight::Run).await?;
                        }
                        Some(crate::channel::transition::ServerState::Follower) => {
                            info!("transitioning to follower");

                            let follower_shutdown = self.shutdown.clone();

                            let (follower_sender, follower_receiver) =
                                crate::channel::transition::Follower::build().await;

                            let follower_heartbeat = self.leader_heartbeat.clone();
                            let mut follower = Follower::init(
                                follower_receiver,
                                follower_heartbeat,
                                follower_shutdown,
                                send_transition.to_owned(),
                            )
                            .await?;

                            tokio::spawn(async move {
                                if let Err(error) = follower.run().await {
                                    error!("follower error -> {:?}", error);
                                }
                            });

                            follower_sender.send(crate::channel::transition::Follower::Run).await?;
                        }
                        Some(crate::channel::transition::ServerState::Candidate) => {
                            info!("transitioning to candidate");

                            let (candidate_sender, candidate_receiver) =
                                crate::channel::transition::Candidate::build().await;

                            let candidate_transition_sender = send_transition.to_owned();
                            let candidate_shutdown = self.shutdown.clone();
                            let candidate_heartbeat = self.leader_heartbeat.clone();

                            let mut candidate = Candidate::init(
                                candidate_receiver,
                                candidate_transition_sender,
                                candidate_shutdown,
                                candidate_heartbeat,
                                self.membership.to_owned(),
                                self.state.to_owned(),
                            )
                            .await?;

                            tokio::spawn(async move {
                                if let Err(error) = candidate.run().await {
                                    error!("candidate error -> {:?}", error);
                                }
                            });

                            candidate_sender.send(crate::channel::transition::Candidate::Run).await?;
                        }
                        Some(crate::channel::transition::ServerState::Leader) => {
                            info!("transitioning to leader");

                            let (leader_sender, leader_receiver) = crate::channel::transition::Leader::build().await;

                            let leader_transition_sender = send_transition.to_owned();
                            let leader_shutdown = self.shutdown.to_owned();
                            let leader_membership = self.membership.to_owned();
                            let leader_state = self.state.to_owned();
                            let mut leader =
                                Leader::init(leader_receiver, leader_transition_sender, leader_shutdown, leader_membership, leader_state).await?;

                            tokio::spawn(async move {
                                if let Err(error) = leader.run().await {
                                    error!("leader error -> {:?}", error);
                                }
                            });

                            leader_sender.send(crate::channel::transition::Leader::Run).await?;
                        }
                        Some(crate::channel::transition::ServerState::Shutdown) => {
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
        crate::channel::transition::Shutdown::send(&self.shutdown).await?;
        crate::channel::state::shutdown(&self.state).await?;
        crate::channel::membership::shutdown(&self.membership).await?;

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
