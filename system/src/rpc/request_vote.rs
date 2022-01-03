#[derive(Clone, Debug)]
pub struct RequestVoteArguments {
    pub term: u32,
    pub candidate_id: String,
    pub last_log_index: u32,
    pub last_log_term: u32,
}

impl RequestVoteArguments {
    pub async fn build() -> Result<RequestVoteArguments, Box<dyn std::error::Error>> {
        let term = 0;
        let candidate_id = String::from("some_candidate_id");
        let last_log_index = 0;
        let last_log_term = 0;

        Ok(RequestVoteArguments {
            term,
            candidate_id,
            last_log_index,
            last_log_term,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RequestVoteResults {
    pub term: u32,
    pub vote_granted: bool,
}

impl RequestVoteResults {
    pub async fn build() -> Result<RequestVoteResults, Box<dyn std::error::Error>> {
        let term = 0;
        let vote_granted = false;

        Ok(RequestVoteResults { term, vote_granted })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn request_vote_arguments() -> Result<(), Box<dyn std::error::Error>> {
        let test_request_vote_arguments = RequestVoteArguments::build().await?;

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
        let test_request_vote_results = RequestVoteResults::build().await?;

        assert_eq!(test_request_vote_results.term, 0);
        assert!(!test_request_vote_results.vote_granted);

        Ok(())
    }
}
