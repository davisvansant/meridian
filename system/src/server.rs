// use tokio::sync::{broadcast, mpsc};
use tokio::sync::broadcast;
// use tokio::time::{sleep, timeout, timeout_at, Duration, Instant};
use tokio::time::{sleep, Duration};

use crate::server::candidate::Candidate;
use crate::server::follower::Follower;
use crate::server::leader::Leader;

pub mod candidate;
pub mod follower;
pub mod leader;
mod preflight;

use crate::channel::LeaderReceiver;
use crate::channel::{CandidateReceiver, CandidateSender, CandidateTransition};
use crate::channel::{ClientSender, MembershipSender, StateSender};
use crate::channel::{ServerReceiver, ServerSender, ServerState};

pub struct Server {
    pub server_state: ServerState,
    client: ClientSender,
    membership: MembershipSender,
    // receiver: ServerReceiver,
    state: StateSender,
    tx: ServerSender,
    rx: ServerReceiver,
    candidate_sender: CandidateSender,
    candidate_receiver: CandidateReceiver,
    heartbeat: LeaderReceiver,
}

impl Server {
    pub async fn init(
        client: ClientSender,
        membership: MembershipSender,
        // receiver: ServerReceiver,
        state: StateSender,
        tx: ServerSender,
        rx: ServerReceiver,
        candidate_sender: CandidateSender,
        candidate_receiver: CandidateReceiver,
        heartbeat: LeaderReceiver,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        let server_state = ServerState::Follower;

        Ok(Server {
            server_state,
            client,
            membership,
            // receiver,
            state,
            tx,
            rx,
            candidate_sender,
            candidate_receiver,
            heartbeat,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // sleep(Duration::from_secs(15)).await;

        let (sender, mut receiver) = broadcast::channel::<ServerState>(64);

        // let (candidate_sender, mut candidate_receiver) = mpsc::channel::<CandidateTransition>(64);

        let launch = sender.clone();
        // let mut follower_receiver = sender.subscribe();
        // let mut candidate_receiver = sender.subscribe();
        // let mut follower_receiver = tx.clone();
        // let mut candidate_receiver = tx.clone();
        let follower_receiver = self.tx.to_owned().subscribe();
        let mut rx = self.tx.to_owned().subscribe();
        // let mut candidate_receiver = tx.subscribe();
        let candidate_sender = self.candidate_sender.to_owned();

        tokio::spawn(async move {
            while let Ok(transition) = rx.recv().await {
                match transition {
                    ServerState::Preflight => {
                        println!("some preflight stuff");

                        if let Err(error) = sender.send(ServerState::Preflight) {
                            println!("preflight error {:?}", error);
                        }
                    }
                    ServerState::Follower => {
                        println!("follwer");

                        if let Err(error) = sender.send(ServerState::Follower) {
                            println!("preflight error {:?}", error);
                        }
                    }
                    ServerState::Candidate => {
                        println!("candidate");

                        if let Err(error) = sender.send(ServerState::Candidate) {
                            println!("preflight error {:?}", error);
                        }
                    }
                    ServerState::Leader => {
                        println!("leader");

                        if let Err(error) = candidate_sender.send(CandidateTransition::Leader).await
                        {
                            println!("error sending transition... {:?}", error);
                        }

                        if let Err(error) = sender.send(ServerState::Leader) {
                            println!("leader error {:?}", error);
                        }
                    }
                    ServerState::Shutdown => {
                        println!("shutdown");

                        if let Err(error) = sender.send(ServerState::Shutdown) {
                            println!("leader error {:?}", error);
                        }

                        break;
                    }
                }
            }
        });

        sleep(Duration::from_secs(5)).await;

        self.tx.send(ServerState::Preflight)?;

        while let Ok(transition) = receiver.recv().await {
            match transition {
                ServerState::Preflight => {
                    println!("running preflight tasks...");

                    preflight::run(&self.client, &self.membership).await?;

                    if let Err(error) = self.tx.send(ServerState::Follower) {
                        println!("error sending server state {:?}", error);
                    }
                }
                ServerState::Follower => {
                    println!("server > follower!");

                    let mut follower = Follower::init().await?;
                    // follower.run(&mut follower_receiver).await?;
                    follower.run(&mut self.heartbeat).await?;

                    if let Err(error) = self.tx.send(ServerState::Candidate) {
                        println!("error sending server state {:?}", error);
                    }
                }
                ServerState::Candidate => {
                    println!("server > candidate!");

                    let mut candidate = Candidate::init().await?;
                    candidate
                        .run(&self.client, &mut self.candidate_receiver, &self.tx)
                        .await?;
                }
                ServerState::Leader => {
                    println!("server > leader!");

                    let mut leader = Leader::init().await?;
                    leader.run(&self.client, &self.state).await?;
                }
                ServerState::Shutdown => {
                    println!("server > shutdown...");
                    // self.server_state = ServerState::Shutdown;

                    // drop(self.client.to_owned());
                    // drop(self.membership.to_owned());
                    // drop(self.state.to_owned());

                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::CandidateTransition;
    use crate::channel::Leader;
    use crate::channel::ServerState;
    use crate::channel::{ClientRequest, ClientResponse};
    use crate::channel::{MembershipRequest, MembershipResponse};
    use crate::channel::{StateRequest, StateResponse};
    use tokio::sync::{broadcast, mpsc, oneshot};

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_client_sender, _test_client_receiver) =
            mpsc::channel::<(ClientRequest, oneshot::Sender<ClientResponse>)>(64);
        let (test_membership_sender, _test_membership_receiver) =
            mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);
        let (test_state_sender, _test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);
        let (test_transition_sender, test_transition_receiver) =
            broadcast::channel::<ServerState>(64);
        let (test_candidate_sender, test_candidate_receiver) =
            mpsc::channel::<CandidateTransition>(64);
        let (test_leader_sender, test_leader_receiver) = mpsc::channel::<Leader>(64);

        let test_server = Server::init(
            test_client_sender,
            test_membership_sender,
            test_state_sender,
            test_transition_sender,
            test_transition_receiver,
            test_candidate_sender,
            test_candidate_receiver,
            test_leader_receiver,
        )
        .await?;

        assert!(!test_server.client.is_closed());
        assert!(!test_server.membership.is_closed());
        assert!(!test_server.state.is_closed());
        assert_eq!(test_server.tx.receiver_count(), 1);
        assert!(!test_server.candidate_sender.is_closed());
        assert!(!test_leader_sender.is_closed());

        Ok(())
    }
}
