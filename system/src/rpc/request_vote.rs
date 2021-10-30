pub struct Arguments {
    pub term: u8,
    pub candidate_id: String,
    pub last_log_index: u8,
    pub last_log_term: u8,
}

impl Arguments {
    pub async fn build() -> Result<Arguments, Box<dyn std::error::Error>> {
        let term = 0;
        let candidate_id = String::from("some_candidate_id");
        let last_log_index = 0;
        let last_log_term = 0;

        Ok(Arguments {
            term,
            candidate_id,
            last_log_index,
            last_log_term,
        })
    }
}

pub struct Results {
    term: u8,
    vote_granted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn arguments() -> Result<(), Box<dyn std::error::Error>> {
        let test_arguments = Arguments::build().await?;
        assert_eq!(test_arguments.term, 0);
        assert_eq!(test_arguments.candidate_id, "some_candidate_id");
        assert_eq!(test_arguments.last_log_index, 0);
        assert_eq!(test_arguments.last_log_term, 0);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn results() -> Result<(), Box<dyn std::error::Error>> {
        let test_results = Results {
            term: 0,
            vote_granted: false,
        };
        assert_eq!(test_results.term, 0);
        assert!(!test_results.vote_granted);
        Ok(())
    }
}
