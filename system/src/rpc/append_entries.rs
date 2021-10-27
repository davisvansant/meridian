pub struct Arguments<'a> {
    term: u8,
    leader_id: &'a str,
    prev_log_index: u8,
    prev_log_term: u8,
    entries: Vec<u8>,
    leader_commit: u8,
}

pub struct Results {
    term: u8,
    success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn arguments() -> Result<(), Box<dyn std::error::Error>> {
        let test_arguments = Arguments {
            term: 0,
            leader_id: "some_leader_id",
            prev_log_index: 0,
            prev_log_term: 0,
            entries: Vec::with_capacity(0),
            leader_commit: 0,
        };
        assert_eq!(test_arguments.term, 0);
        assert_eq!(test_arguments.leader_id, "some_leader_id");
        assert_eq!(test_arguments.prev_log_index, 0);
        assert_eq!(test_arguments.prev_log_term, 0);
        assert_eq!(test_arguments.entries.len(), 0);
        assert_eq!(test_arguments.leader_commit, 0);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn results() -> Result<(), Box<dyn std::error::Error>> {
        let test_results = Results {
            term: 0,
            success: false,
        };
        assert_eq!(test_results.term, 0);
        assert!(!test_results.success);
        Ok(())
    }
}
