// use tokio::time::{sleep, timeout, timeout_at, Duration, Instant};
use tokio::sync::{broadcast, mpsc};

use crate::server::candidate::Candidate;
use crate::server::follower::Follower;
use crate::server::leader::Leader;

pub mod candidate;
pub mod follower;
pub mod leader;
mod preflight;

use crate::channel::CandidateTransition;
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
}

impl Server {
    pub async fn init(
        client: ClientSender,
        membership: MembershipSender,
        // receiver: ServerReceiver,
        state: StateSender,
        tx: ServerSender,
        rx: ServerReceiver,
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
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // sleep(Duration::from_secs(15)).await;

        let (sender, mut receiver) = broadcast::channel::<ServerState>(64);

        let (candidate_sender, mut candidate_receiver) = mpsc::channel::<CandidateTransition>(64);

        let launch = sender.clone();
        // let mut follower_receiver = sender.subscribe();
        // let mut candidate_receiver = sender.subscribe();
        // let mut follower_receiver = tx.clone();
        // let mut candidate_receiver = tx.clone();
        let mut follower_receiver = self.tx.to_owned().subscribe();
        let mut rx = self.tx.to_owned().subscribe();
        // let mut candidate_receiver = tx.subscribe();

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
                            println!("error sending transition...");
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
                    follower.run(&mut follower_receiver).await?;

                    if let Err(error) = self.tx.send(ServerState::Candidate) {
                        println!("error sending server state {:?}", error);
                    }
                }
                ServerState::Candidate => {
                    println!("server > candidate!");

                    let mut candidate = Candidate::init().await?;
                    candidate
                        .run(&self.client, &mut candidate_receiver, &self.tx)
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

    // #[tokio::test(flavor = "multi_thread")]
    // async fn server_state_candidate() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_server_state = ServerState::Candidate;
    //     assert_eq!(test_server_state, ServerState::Candidate);
    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn server_state_follower() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_server_state = ServerState::Follower;
    //     assert_eq!(test_server_state, ServerState::Follower);
    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn server_state_leader() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_server_state = ServerState::Leader;
    //     assert_eq!(test_server_state, ServerState::Leader);
    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn init() -> Result<(), Box<dyn std::error::Error>> {
    //     // let (test_sender, _) = test_channel().await;
    //     // let (test_subscriber, _) = test_channel().await;
    //     // let test_subscriber_clone = test_subscriber.clone();
    //     // let test_server = Server::init(test_subscriber, test_subscriber_clone).await?;
    //     let test_server = test_server().await?;
    //     // assert_eq!(test_server.server_state, ServerState::Follower);
    //     assert_eq!(test_server.persistent_state.current_term, 0);
    //     assert_eq!(test_server.persistent_state.voted_for, None);
    //     assert_eq!(test_server.persistent_state.log.len(), 0);
    //     assert_eq!(test_server.persistent_state.log.capacity(), 4096);
    //     assert_eq!(test_server.volatile_state.commit_index, 0);
    //     assert_eq!(test_server.volatile_state.last_applied, 0);
    //     Ok(())
    // }
}
