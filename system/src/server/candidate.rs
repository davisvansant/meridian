use tokio::time::{timeout_at, Duration, Instant};

use crate::channel::rpc_client;
use crate::channel::rpc_client::RpcClientSender;
use crate::channel::server::{CandidateReceiver, CandidateTransition};
use crate::{error, info};

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
        client: &RpcClientSender,
        transition: &mut CandidateReceiver,
    ) -> Result<CandidateTransition, Box<dyn std::error::Error>> {
        let client_owner = client.to_owned();

        tokio::spawn(async move {
            if let Err(error) = rpc_client::start_election(&client_owner).await {
                error!("election error -> {:?}", error);
            }
        });

        loop {
            match timeout_at(Instant::now() + self.election_timeout, transition.recv()).await {
                Ok(state) => match state {
                    Ok(CandidateTransition::Follower) => {
                        info!("received heartbeat...stepping down");

                        return Ok(CandidateTransition::Follower);
                    }
                    Ok(CandidateTransition::Leader) => {
                        info!("transitioning server to leader...");

                        return Ok(CandidateTransition::Leader);
                    }
                    Err(error) => {
                        error!("error receiving candidate transition... -> {:?}", error);
                        continue;
                    }
                },
                Err(error) => error!(
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
