use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{sleep, Duration};

use crate::channel::shutdown;
use crate::channel::LeaderSender;
// use crate::channel::ShutdownReceiver;
use crate::channel::ShutdownSender;
use crate::channel::{CandidateSender, CandidateTransition};
use crate::channel::{MembershipSender, RpcClientSender, StateSender};
use crate::server::candidate::Candidate;
use crate::server::follower::Follower;
use crate::server::leader::Leader;

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

        let mut stream = signal(SignalKind::interrupt())?;

        // let mut server_shutdown = self.shutdown.to_owned();
        // let mut leader_shutdown = self.shutdown.to_owned();
        let mut server_shutdown = self.shutdown.subscribe();

        // drop(self.shutdown.to_owned());

        self.server_state = ServerState::Preflight;

        loop {
            tokio::select! {
                biased;
                // _ = stream.recv() => {
                _ = server_shutdown.recv() => {
                    println!("shutting down system server...");

                    self.server_state = ServerState::Shutdown;

                    break
                }
                server_state = self.server_state() => {
                    if self.server_state == ServerState::Shutdown {
                        println!("server > shutdown...");

                        break;
                    } else {
                        println!("output of server state -> {:?}", server_state);
                    }
                }
            }
        }

        self.run_shutdown().await?;

        Ok(())
    }

    async fn server_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.server_state {
            ServerState::Preflight => {
                println!("running preflight tasks...");

                let mut errors = 0;

                while errors <= 3 {
                    match preflight::run(&self.membership).await {
                        Ok(()) => {
                            println!("launching...");

                            break;
                        }
                        Err(error) => {
                            println!("preflight error -> {:?}", error);
                            println!("attempting preflight again -> {:?}", &error);

                            errors += 1;
                        }
                    }
                }

                if errors >= 3 {
                    self.server_state = ServerState::Shutdown;
                } else {
                    self.server_state = ServerState::Follower;

                    sleep(Duration::from_secs(5)).await;
                }

                Ok(())
            }
            ServerState::Follower => {
                println!("server > follower!");

                let mut heartbeat = self.heartbeat.subscribe();
                let mut follower = Follower::init().await?;

                follower.run(&mut heartbeat).await?;

                self.server_state = ServerState::Candidate;

                Ok(())
            }
            ServerState::Candidate => {
                println!("server > candidate!");

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
                        println!("candidate error -> {:?}", error);

                        self.server_state = ServerState::Candidate;
                    }
                }

                Ok(())
            }
            ServerState::Leader => {
                println!("server > leader!");

                // let mut leader_shutdown = self.shutdown.to_owned();
                let mut shutdown = self.shutdown.subscribe();

                let mut leader = Leader::init(shutdown).await?;
                leader.run(&self.client, &self.state).await?;

                self.server_state = ServerState::Candidate;

                Ok(())
            }
            ServerState::Shutdown => {
                println!("server > shutdown...");

                // shutdown(&self.shutdown).await?;

                // crate::channel::shutdown_state(&self.state).await?;
                // crate::channel::shutdown_rpc_client(&self.client).await?;
                // crate::channel::shutdown_membership(&self.membership).await?;

                self.server_state = ServerState::Shutdown;

                Ok(())
            }
        }
    }

    async fn run_shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        shutdown(&self.shutdown).await?;

        crate::channel::shutdown_state(&self.state).await?;
        crate::channel::shutdown_rpc_client(&self.client).await?;
        crate::channel::shutdown_membership(&self.membership).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::build_shutdown_channel;
    use crate::channel::CandidateTransition;
    use crate::channel::Leader;
    use crate::channel::RpcClientRequest;
    use crate::channel::{MembershipRequest, MembershipResponse};
    use crate::channel::{StateRequest, StateResponse};
    use tokio::sync::{broadcast, mpsc, oneshot};

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_client_sender, _test_client_receiver) = mpsc::channel::<RpcClientRequest>(64);
        let (test_membership_sender, _test_membership_receiver) =
            mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);
        let (test_state_sender, _test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);
        let (test_candidate_sender, _test_candidate_receiver) =
            broadcast::channel::<CandidateTransition>(64);
        let (test_leader_sender, _test_leader_receiver) = broadcast::channel::<Leader>(64);
        let test_shutdown_sender = build_shutdown_channel().await;

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
