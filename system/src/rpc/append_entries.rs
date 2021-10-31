pub struct Arguments {
    term: u8,
    leader_id: String,
    prev_log_index: u8,
    prev_log_term: u8,
    entries: Vec<u8>,
    leader_commit: u8,
}

impl Arguments {
    pub async fn build() -> Result<Arguments, Box<dyn std::error::Error>> {
        let term = 0;
        let leader_id = String::from("some_leader_id");
        let prev_log_index = 0;
        let prev_log_term = 0;
        let entries = Vec::with_capacity(10);
        let leader_commit = 0;

        Ok(Arguments {
            term,
            leader_id,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit,
        })
    }
}

pub struct Results {
    term: u8,
    success: bool,
}

impl Results {
    pub async fn build() -> Result<Results, Box<dyn std::error::Error>> {
        let term = 0;
        let success = false;

        Ok(Results { term, success })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn arguments() -> Result<(), Box<dyn std::error::Error>> {
        let test_arguments = Arguments::build().await?;

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
        let test_results = Results::build().await?;

        assert_eq!(test_results.term, 0);
        assert!(!test_results.success);

        Ok(())
    }
}
