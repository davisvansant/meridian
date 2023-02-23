use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestVoteArguments {
    pub term: u32,
    pub candidate_id: String,
    pub last_log_index: u32,
    pub last_log_term: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestVoteResults {
    pub term: u32,
    pub vote_granted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn request_vote_arguments() -> Result<(), Box<dyn std::error::Error>> {
        let test_request_vote_arguments = RequestVoteArguments {
            term: 0,
            candidate_id: String::from("some_candidate_id"),
            last_log_index: 0,
            last_log_term: 0,
        };

        assert_eq!(test_request_vote_arguments.term, 0);
        assert_eq!(
            test_request_vote_arguments.candidate_id,
            "some_candidate_id",
        );
        assert_eq!(test_request_vote_arguments.last_log_index, 0);
        assert_eq!(test_request_vote_arguments.last_log_term, 0);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn request_vote_results() -> Result<(), Box<dyn std::error::Error>> {
        let test_request_vote_results = RequestVoteResults {
            term: 0,
            vote_granted: false,
        };

        assert_eq!(test_request_vote_results.term, 0);
        assert!(!test_request_vote_results.vote_granted);

        Ok(())
    }
}
