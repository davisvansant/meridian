#[derive(Clone, Debug)]
pub struct AppendEntriesArguments {
    pub term: u32,
    pub leader_id: String,
    pub prev_log_index: u32,
    pub prev_log_term: u32,
    pub entries: Vec<u32>,
    pub leader_commit: u32,
}

impl AppendEntriesArguments {
    pub async fn build() -> Result<AppendEntriesArguments, Box<dyn std::error::Error>> {
        let term = 0;
        let leader_id = String::from("some_leader_id");
        let prev_log_index = 0;
        let prev_log_term = 0;
        let entries = Vec::with_capacity(10);
        let leader_commit = 0;

        Ok(AppendEntriesArguments {
            term,
            leader_id,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit,
        })
    }
}

#[derive(Clone, Debug)]
pub struct AppendEntriesResults {
    pub term: u32,
    pub success: bool,
}

impl AppendEntriesResults {
    pub async fn build() -> Result<AppendEntriesResults, Box<dyn std::error::Error>> {
        let term = 0;
        let success = false;

        Ok(AppendEntriesResults { term, success })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn append_entries_arguments() -> Result<(), Box<dyn std::error::Error>> {
        let test_append_entries_arguments = AppendEntriesArguments::build().await?;

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
        let test_append_entries_results = AppendEntriesResults::build().await?;

        assert_eq!(test_append_entries_results.term, 0);
        assert!(!test_append_entries_results.success);

        Ok(())
    }
}
