pub struct Arguments<'a> {
    term: u8,
    candidate_id: &'a str,
    last_log_index: u8,
    last_log_term: u8,
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
        let test_arguments = Arguments {
            term: 0,
            candidate_id: "some_candidate_id",
            last_log_index: 0,
            last_log_term: 0,
        };
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
