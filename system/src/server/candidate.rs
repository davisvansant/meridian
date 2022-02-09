use tokio::time::{timeout_at, Duration, Instant};

use crate::channel::start_election;
use crate::channel::{CandidateReceiver, CandidateTransition, ClientSender};

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
    ) -> Result<CandidateTransition, Box<dyn std::error::Error>> {
        let client_owner = client.to_owned();

        tokio::spawn(async move {
            if let Err(error) = start_election(&client_owner).await {
                println!("election error -> {:?}", error);
            }
        });

        loop {
            match timeout_at(Instant::now() + self.election_timeout, transition.recv()).await {
                Ok(state) => match state {
                    Ok(CandidateTransition::Follower) => {
                        println!("received heartbeat...stepping down");

                        return Ok(CandidateTransition::Follower);
                    }
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
}
