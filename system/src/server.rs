use tokio::time::{sleep, Duration};

use crate::channel::membership::failure_detector;
use crate::channel::membership::MembershipSender;
use crate::channel::rpc_client::RpcClientSender;
use crate::channel::server::LeaderSender;
use crate::channel::server::{CandidateSender, CandidateTransition};
use crate::channel::shutdown::ShutdownSender;
use crate::channel::state::StateSender;
use crate::server::candidate::Candidate;
use crate::server::follower::Follower;
use crate::server::leader::Leader;
use crate::{error, info, warn};

mod candidate;
mod follower;
mod leader;
mod preflight;

#[derive(PartialEq)]
pub enum ServerState {
    Candidate,
    Follower,
    Leader,
    Preflight,
    Shutdown,
}

pub struct Server {
    pub server_state: ServerState,
    client: RpcClientSender,
    membership: MembershipSender,
    state: StateSender,
    candidate_sender: CandidateSender,
    heartbeat: LeaderSender,
    shutdown: ShutdownSender,
}

impl Server {
    pub async fn init(
        client: RpcClientSender,
        membership: MembershipSender,
        state: StateSender,
        candidate_sender: CandidateSender,
        heartbeat: LeaderSender,
        shutdown: ShutdownSender,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        let server_state = ServerState::Shutdown;

        Ok(Server {
            server_state,
            client,
            membership,
            state,
            candidate_sender,
            heartbeat,
            shutdown,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        sleep(Duration::from_secs(5)).await;

        let mut server_shutdown = self.shutdown.subscribe();

        self.server_state = ServerState::Preflight;

        loop {
            tokio::select! {
                biased;
                _ = server_shutdown.recv() => {
                    info!("shutting down system server...");

                    self.server_state = ServerState::Shutdown;

                    break
                }
                server_state = self.server_state() => {
                    if self.server_state == ServerState::Shutdown {
                        info!("server > shutdown...");

                        break;
                    } else {
                        info!("output of server state -> {:?}", server_state);
                    }
                }
            }
        }

        sleep(Duration::from_secs(5)).await;

        self.run_shutdown().await?;

        Ok(())
    }

    async fn server_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.server_state {
            ServerState::Preflight => {
                info!("running preflight tasks...");

                let mut errors = 0;

                while errors <= 2 {
                    match preflight::run(&self.membership).await {
                        Ok(()) => {
                            info!("launching...");

                            break;
                        }
                        Err(error) => {
                            error!("preflight error -> {:?}", error);
                            error!("attempting preflight again -> {:?}", &error);

                            errors += 1;
                        }
                    }
                    sleep(Duration::from_secs(10)).await;
                }

                if errors >= 2 {
                    warn!("preflight tasks failed...shutting down...");

                    self.server_state = ServerState::Shutdown;
                } else {
                    self.server_state = ServerState::Follower;

                    sleep(Duration::from_secs(10)).await;

                    failure_detector(&self.membership).await?;
                }

                Ok(())
            }
            ServerState::Follower => {
                info!("server > follower!");

                let mut heartbeat = self.heartbeat.subscribe();
                let mut follower = Follower::init().await?;

                follower.run(&mut heartbeat).await?;

                self.server_state = ServerState::Candidate;

                Ok(())
            }
            ServerState::Candidate => {
                info!("server > candidate!");

                let mut candidate_receiver = self.candidate_sender.subscribe();
                let mut candidate = Candidate::init().await?;

                match candidate.run(&self.client, &mut candidate_receiver).await {
                    Ok(CandidateTransition::Follower) => {
                        self.server_state = ServerState::Follower;
                    }
                    Ok(CandidateTransition::Leader) => {
                        self.server_state = ServerState::Leader;
                    }
                    Err(error) => {
                        error!("candidate error -> {:?}", error);

                        self.server_state = ServerState::Candidate;
                    }
                }

                Ok(())
            }
            ServerState::Leader => {
                info!("server > leader!");

                let shutdown = self.shutdown.subscribe();
                let mut leader = Leader::init(shutdown).await?;

                leader.run(&self.client, &self.state).await?;

                self.server_state = ServerState::Candidate;

                Ok(())
            }
            ServerState::Shutdown => {
                info!("server > shutdown...");

                self.server_state = ServerState::Shutdown;

                Ok(())
            }
        }
    }

    async fn run_shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        crate::channel::shutdown::shutdown(&self.shutdown).await?;
        crate::channel::state::shutdown(&self.state).await?;
        crate::channel::rpc_client::shutdown(&self.client).await?;
        crate::channel::membership::shutdown(&self.membership).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_client_sender, _test_client_receiver) = crate::channel::rpc_client::build().await;
        let (test_membership_sender, _test_membership_receiver) =
            crate::channel::membership::build().await;
        let (test_state_sender, _test_state_receiver) = crate::channel::state::build().await;
        let test_candidate_sender = crate::channel::server::build_candidate_transition().await;
        let _test_candidate_receiver = test_candidate_sender.subscribe();
        let test_leader_sender = crate::channel::server::build_leader_heartbeat().await;
        let _test_leader_receiver = test_leader_sender.subscribe();
        let test_shutdown_sender = crate::channel::shutdown::build().await;

        let test_server = Server::init(
            test_client_sender,
            test_membership_sender,
            test_state_sender,
            test_candidate_sender,
            test_leader_sender,
            test_shutdown_sender,
        )
        .await?;

        assert!(!test_server.client.is_closed());
        assert!(!test_server.membership.is_closed());
        assert!(!test_server.state.is_closed());
        assert_eq!(test_server.candidate_sender.receiver_count(), 1);
        assert_eq!(test_server.heartbeat.receiver_count(), 1);

        Ok(())
    }
}
