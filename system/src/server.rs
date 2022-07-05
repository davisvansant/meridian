use tokio::time::{sleep, Duration};

use crate::channel::state::{StateRequest, StateSender};
use crate::channel::{membership, server, transition};
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
    membership: membership::MembershipSender,
    state: StateSender,
    leader_heartbeat: server::LeaderSender,
    shutdown: transition::ShutdownSender,
    transition: transition::ServerStateReceiver,
}

impl Server {
    pub async fn init(
        membership: membership::MembershipSender,
        state: StateSender,
        leader_heartbeat: server::LeaderSender,
        shutdown: transition::ShutdownSender,
        transition: transition::ServerStateReceiver,
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
        send_transition: &transition::ServerStateSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut server_shutdown = self.shutdown.subscribe();

        sleep(Duration::from_secs(5)).await;

        send_transition
            .send(transition::ServerState::Preflight)
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
                        Some(transition::ServerState::Preflight) => {
                            info!("running preflight tasks");

                            let (transition, receive) = transition::Preflight::build().await;

                            let mut preflight = Preflight::init(
                                receive,
                                send_transition.to_owned(),
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

                            transition.send(transition::Preflight::Run).await?;
                        }
                        Some(transition::ServerState::Follower) => {
                            info!("transitioning to follower");

                            let (transition, receive) = transition::Follower::build().await;

                            let mut follower = Follower::init(
                                receive,
                                self.leader_heartbeat.to_owned(),
                                self.shutdown.to_owned(),
                                send_transition.to_owned(),
                            )
                            .await?;

                            tokio::spawn(async move {
                                if let Err(error) = follower.run().await {
                                    error!("follower error -> {:?}", error);
                                }
                            });

                            transition.send(transition::Follower::Run).await?;
                        }
                        Some(transition::ServerState::Candidate) => {
                            info!("transitioning to candidate");

                            let (transition, receive) = transition::Candidate::build().await;

                            let mut candidate = Candidate::init(
                                receive,
                                send_transition.to_owned(),
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

                            transition.send(transition::Candidate::Run).await?;
                        }
                        Some(transition::ServerState::Leader) => {
                            info!("transitioning to leader");

                            let (transition, receive) = transition::Leader::build().await;

                            let mut leader = Leader::init(
                                receive,
                                send_transition.to_owned(),
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

                            transition.send(transition::Leader::Run).await?;
                        }
                        Some(transition::ServerState::Shutdown) => {
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
        transition::Shutdown::send(&self.shutdown).await?;
        StateRequest::shutdown(&self.state).await?;
        membership::shutdown(&self.membership).await?;

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
