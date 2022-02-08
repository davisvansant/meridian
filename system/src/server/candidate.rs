use tokio::time::{timeout_at, Duration, Instant};

use crate::channel::start_election;
use crate::channel::{CandidateReceiver, CandidateTransition, ClientSender};
use crate::server::{ServerSender, ServerState};

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
        // ) -> Result<(), Box<dyn std::error::Error>> {
        // ) -> Result<ServerState, Box<dyn std::error::Error>> {
    ) -> Result<CandidateTransition, Box<dyn std::error::Error>> {
        // start_election(client).await?;
        let client_owner = client.to_owned();
        tokio::spawn(async move {
            if let Err(error) = start_election(&client_owner).await {
                println!("election error -> {:?}", error);
            }
        });

        loop {
            match timeout_at(Instant::now() + self.election_timeout, transition.recv()).await {
                Ok(state) => match state {
                    // Some(CandidateTransition::Follower) => {
                    Ok(CandidateTransition::Follower) => {
                        println!("received heartbeat...stepping down");

                        return Ok(CandidateTransition::Follower);
                    }
                    // Some(CandidateTransition::Leader) => {
                    Ok(CandidateTransition::Leader) => {
                        println!("transitioning server to leader...");

                        return Ok(CandidateTransition::Leader);
                    }
                    Err(error) => {
                        println!("error receiving candidate transition... -> {:?}", error);
                        continue;
                    }
                },
                Err(error) => println!(
                    "candidate election timeout lapsed...trying again...{:?}",
                    error,
                ),
            }
        }

        // Ok(())
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
