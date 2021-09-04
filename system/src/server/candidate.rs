use tokio::time::Duration;

use crate::internal_cluster_grpc_client::InternalClusterGrpcClient;
use crate::meridian_cluster_v010::RequestVoteRequest;
use crate::server::Server;

pub struct Candidate {
    pub election_timeout: Duration,
}

impl Candidate {
    pub async fn init() -> Result<Candidate, Box<dyn std::error::Error>> {
        let election_timeout = Duration::from_millis(150);
        Ok(Candidate { election_timeout })
    }

    pub async fn start_election(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("starting election...");
        Ok(())
    }
}

impl Server {
    pub(super) async fn increment_current_term(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.persistent_state.current_term += 1;

        Ok(())
    }

    pub(super) async fn build_request_vote_request(
        &self,
    ) -> Result<RequestVoteRequest, Box<dyn std::error::Error>> {
        let term = self.persistent_state.current_term;
        let candidate_id = String::from("some_candidate_id");
        let last_log_index = self.persistent_state.log.len() as u32;
        let last_log_term = if let Some(log) = self.persistent_state.log.last() {
            log.term
        } else {
            0
        };

        let request_vote_request = RequestVoteRequest {
            term,
            candidate_id,
            last_log_index,
            last_log_term,
        };

        Ok(request_vote_request)
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

    #[tokio::test(flavor = "multi_thread")]
    async fn start_election() -> Result<(), Box<dyn std::error::Error>> {
        let test_candidate = Candidate::init().await?;
        assert!(test_candidate.start_election().await.is_ok());
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn increment_current_term() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_server = Server::init().await?;
        assert_eq!(test_server.persistent_state.current_term, 0);
        test_server.increment_current_term().await?;
        assert_eq!(test_server.persistent_state.current_term, 1);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn build_request_vote_request() -> Result<(), Box<dyn std::error::Error>> {
        let test_server = Server::init().await?;
        let test_request = test_server.build_request_vote_request().await?;
        assert_eq!(test_request.term, 0);
        assert_eq!(test_request.candidate_id.as_str(), "some_candidate_id");
        assert_eq!(test_request.last_log_index, 0);
        assert_eq!(test_request.last_log_term, 0);
        Ok(())
    }
}
