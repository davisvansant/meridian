use tokio::time::Duration;

use crate::internal_cluster_grpc_client::InternalClusterGrpcClient;
use crate::meridian_cluster_v010::RequestVoteRequest;

pub struct Candidate {
    pub election_timeout: Duration,
}

impl Candidate {
    pub async fn init() -> Result<Candidate, Box<dyn std::error::Error>> {
        let election_timeout = Duration::from_millis(150);
        Ok(Candidate { election_timeout })
    }

    pub async fn start_election(
        &self,
        request: RequestVoteRequest,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // println!("starting election...");
        let members: Vec<&str> = Vec::with_capacity(5);

        if members.is_empty() {
            Ok(true)
        } else {
            let mut transport = InternalClusterGrpcClient::init("some_test_candidate_id").await?;
            let result = transport.request_vote(request).await?;

            match result.into_inner().vote_granted.as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Ok(false),
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
