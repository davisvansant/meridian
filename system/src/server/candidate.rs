use tokio::time::{timeout_at, Duration, Instant};

use crate::channel::start_election;
use crate::channel::{CandidateReceiver, CandidateTransition, ClientSender};
use crate::server::ServerSender;

pub struct Candidate {
    pub election_timeout: Duration,
}

impl Candidate {
    pub async fn init() -> Result<Candidate, Box<dyn std::error::Error>> {
        let election_timeout = Duration::from_millis(150);
        Ok(Candidate { election_timeout })
    }

    pub async fn run(
        &mut self,
        client: &ClientSender,
        transition: &mut CandidateReceiver,
        tx: &ServerSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        start_election(client).await?;

        loop {
            match timeout_at(Instant::now() + self.election_timeout, transition.recv()).await {
                Ok(state) => {
                    match state {
                        Some(CandidateTransition::Follower) => {
                            println!("received heartbeat...stepping down");

                            // if let Err(error) = tx.send(ServerState::Follower) {
                            //     println!("error sending server state {:?}", error);
                            // }

                            break;
                        }
                        Some(CandidateTransition::Leader) => {
                            // if let Err(error) = tx.send(ServerState::Leader) {
                            //     println!("error sending server state {:?}", error);
                            // }
                            println!("transitioning server to leader...");
                            break;
                        }
                        None => break,
                    }
                }
                Err(error) => println!(
                    "candidate election timeout lapsed...trying again...{:?}",
                    error,
                ),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_candidate = Candidate::init().await?;
        assert_eq!(test_candidate.election_timeout.as_millis(), 150);
        Ok(())
    }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn start_election() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_candidate = Candidate::init().await?;
    //     // let test_server = Server::init().await?;
    //     let test_server = test_server().await?;
    //     let test_request = test_server.build_request_vote_request().await?;
    //     assert!(test_candidate.start_election(test_request).await.is_ok());
    //     Ok(())
    // }
}
