#[derive(Clone, Debug)]
pub struct AppendEntriesArguments {
    pub term: u32,
    pub leader_id: String,
    pub prev_log_index: u32,
    pub prev_log_term: u32,
    pub entries: Vec<u32>,
    pub leader_commit: u32,
}

#[derive(Clone, Debug)]
pub struct AppendEntriesResults {
    pub term: u32,
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn append_entries_arguments() -> Result<(), Box<dyn std::error::Error>> {
        let test_append_entries_arguments = AppendEntriesArguments {
            term: 0,
            leader_id: String::from("some_leader_id"),
            prev_log_index: 0,
            prev_log_term: 0,
            entries: Vec::with_capacity(0),
            leader_commit: 0,
        };

        assert_eq!(test_append_entries_arguments.term, 0);
        assert_eq!(test_append_entries_arguments.leader_id, "some_leader_id");
        assert_eq!(test_append_entries_arguments.prev_log_index, 0);
        assert_eq!(test_append_entries_arguments.prev_log_term, 0);
        assert_eq!(test_append_entries_arguments.entries.len(), 0);
        assert_eq!(test_append_entries_arguments.leader_commit, 0);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn append_entries_results() -> Result<(), Box<dyn std::error::Error>> {
        let test_append_entries_results = AppendEntriesResults {
            term: 0,
            success: false,
        };

        assert_eq!(test_append_entries_results.term, 0);
        assert!(!test_append_entries_results.success);

        Ok(())
    }
}
