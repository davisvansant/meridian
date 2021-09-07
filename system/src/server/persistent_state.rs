pub struct LogEntry {
    pub term: u32,
    pub command: String,
    pub committed: bool,
}

pub struct PersistentState {
    pub current_term: u32,
    pub voted_for: Option<String>,
    pub log: Vec<LogEntry>,
}

impl PersistentState {
    pub async fn init() -> Result<PersistentState, Box<dyn std::error::Error>> {
        let current_term = 0;
        let voted_for = None;
        let log = Vec::with_capacity(4096);

        Ok(PersistentState {
            current_term,
            voted_for,
            log,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn log_entry() -> Result<(), Box<dyn std::error::Error>> {
        let test_log_entry = LogEntry {
            term: 0,
            command: String::from("test_log_entry"),
            committed: true,
        };
        assert_eq!(test_log_entry.term, 0);
        assert_eq!(test_log_entry.command.as_str(), "test_log_entry");
        assert!(test_log_entry.committed);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_persistent_state = PersistentState::init().await?;
        assert_eq!(test_persistent_state.current_term, 0);
        assert_eq!(test_persistent_state.voted_for, None);
        assert_eq!(test_persistent_state.log.len(), 0);
        assert_eq!(test_persistent_state.log.capacity(), 4096);
        Ok(())
    }
}
