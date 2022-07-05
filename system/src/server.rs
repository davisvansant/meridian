use tokio::time::{sleep, Duration};

use crate::channel::membership;
use crate::channel::server::LeaderHeartbeatSender;
use crate::channel::state::{StateRequest, StateSender};
use crate::channel::transition::{
    CandidateState, FollowerState, LeaderState, PreflightState, Shutdown, ShutdownSender,
    Transition, TransitionReceiver, TransitionSender,
};
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
    leader_heartbeat: LeaderHeartbeatSender,
    shutdown: ShutdownSender,
    transition: TransitionReceiver,
}

impl Server {
    pub async fn init(
        membership: membership::MembershipSender,
        state: StateSender,
        leader_heartbeat: LeaderHeartbeatSender,
        shutdown: ShutdownSender,
        transition: TransitionReceiver,
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
        send_transition: &TransitionSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut server_shutdown = self.shutdown.subscribe();

        sleep(Duration::from_secs(5)).await;

        send_transition.send(Transition::PreflightState).await?;

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
                        Some(Transition::PreflightState) => {
                            info!("running preflight tasks");

                            let (transition, enter_state) = PreflightState::build().await;

                            let mut preflight = Preflight::init(
                                enter_state,
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

                            transition.send(PreflightState::Run).await?;
                        }
                        Some(Transition::FollowerState) => {
                            info!("transitioning to follower");

                            let (transition, enter_state) = FollowerState::build().await;

                            let mut follower = Follower::init(
                                enter_state,
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

                            transition.send(FollowerState::Run).await?;
                        }
                        Some(Transition::CandidateState) => {
                            info!("transitioning to candidate");

                            let (transition, enter_state) = CandidateState::build().await;

                            let mut candidate = Candidate::init(
                                enter_state,
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

                            transition.send(CandidateState::Run).await?;
                        }
                        Some(Transition::LeaderState) => {
                            info!("transitioning to leader");

                            let (transition, enter_state) = LeaderState::build().await;

                            let mut leader = Leader::init(
                                enter_state,
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

                            transition.send(LeaderState::Run).await?;
                        }
                        Some(Transition::Shutdown) => {
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
        Shutdown::send(&self.shutdown).await?;
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
