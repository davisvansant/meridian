use crate::{info, warn};

pub struct LogEntry {
    pub term: u32,
    pub command: String,
    pub committed: bool,
}

pub struct Persistent {
    pub current_term: u32,
    pub voted_for: Option<String>,
    pub log: Vec<LogEntry>,
}

impl Persistent {
    pub async fn init() -> Result<Persistent, Box<dyn std::error::Error>> {
        let current_term = 0;
        let voted_for = None;
        let log = Vec::with_capacity(4096);

        Ok(Persistent {
            current_term,
            voted_for,
            log,
        })
    }

    pub async fn adjust_term(&mut self, term: u32) {
        self.current_term = term;
        self.voted_for = None;
    }

    pub async fn check_candidate_log(&self, candidate_log: u32) -> bool {
        self.current_term >= candidate_log
    }

    pub async fn check_candidate_id(&self, candidate_id: &str) -> bool {
        self.voted_for == None || self.voted_for == Some(candidate_id.to_string())
    }

    pub async fn check_term(&mut self, term: u32) -> bool {
        match term.cmp(&self.current_term) {
            std::cmp::Ordering::Less => {
                info!(
                    "current term higher than incoming -> current {:?} | incoming {:?}",
                    self.current_term, term,
                );

                false
            }
            std::cmp::Ordering::Equal => {
                info!(
                    "terms are equal! current {:?} | {:?}",
                    self.current_term, term,
                );

                true
            }
            std::cmp::Ordering::Greater => {
                warn!(
                    "adjusting term! current {:?} | {:?}",
                    self.current_term, term,
                );

                self.adjust_term(term).await;

                true
            }
        }
    }

    pub async fn increment_current_term(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.current_term += 1;

        Ok(())
    }

    pub async fn vote_for_self(&mut self, node_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.voted_for = Some(node_id.to_string());

        Ok(())
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
        let test_persistent_state = Persistent::init().await?;
        assert_eq!(test_persistent_state.current_term, 0);
        assert_eq!(test_persistent_state.voted_for, None);
        assert_eq!(test_persistent_state.log.len(), 0);
        assert_eq!(test_persistent_state.log.capacity(), 4096);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn adjust_term() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_persistent_state = Persistent::init().await?;
        test_persistent_state.voted_for = Some(String::from("test_candidate_id"));
        assert_eq!(test_persistent_state.current_term, 0);
        assert_eq!(
            test_persistent_state.voted_for.as_ref().unwrap().as_str(),
            "test_candidate_id",
        );
        test_persistent_state.adjust_term(1).await;
        assert_eq!(test_persistent_state.current_term, 1);
        assert!(test_persistent_state.voted_for.is_none());
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_candidate_log_true() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_persistent_state = Persistent::init().await?;

        assert_eq!(test_persistent_state.current_term, 0);
        assert!(test_persistent_state.check_candidate_log(0).await);

        test_persistent_state.current_term = 2;

        assert!(test_persistent_state.check_candidate_log(1).await);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_candidate_log_false() -> Result<(), Box<dyn std::error::Error>> {
        let test_persistent_state = Persistent::init().await?;

        assert_eq!(test_persistent_state.current_term, 0);
        assert!(!test_persistent_state.check_candidate_log(1).await);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_candidate_id_true() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_persistent_state = Persistent::init().await?;

        assert!(test_persistent_state.voted_for.is_none());
        assert!(
            test_persistent_state
                .check_candidate_id("some_candidate_id")
                .await
        );

        test_persistent_state.voted_for = Some(String::from("test_vote_for_self"));

        assert!(
            test_persistent_state
                .check_candidate_id("test_vote_for_self")
                .await
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_candidate_id_false() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_persistent_state = Persistent::init().await?;

        test_persistent_state.voted_for = Some(String::from("some_test_uuid"));

        assert!(
            !test_persistent_state
                .check_candidate_id("some_candidate_id")
                .await
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_term_true() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_persistent_state = Persistent::init().await?;

        assert!(test_persistent_state.check_term(0).await);

        test_persistent_state.current_term = 1;

        assert!(test_persistent_state.check_term(2).await);
        assert_eq!(test_persistent_state.current_term, 2);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_term_false() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_persistent_state = Persistent::init().await?;

        test_persistent_state.current_term = 1;

        assert!(!test_persistent_state.check_term(0).await);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn increment_current_term() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_persistent_state = Persistent::init().await?;
        assert_eq!(test_persistent_state.current_term, 0);
        test_persistent_state.increment_current_term().await?;
        assert_eq!(test_persistent_state.current_term, 1);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn vote_for_self() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_persistent_state = Persistent::init().await?;
        assert!(test_persistent_state.voted_for.is_none());
        test_persistent_state
            .vote_for_self("test_candidate_id")
            .await?;
        assert!(test_persistent_state.voted_for.is_some());
        assert_eq!(
            test_persistent_state.voted_for.unwrap().as_str(),
            "test_candidate_id",
        );
        Ok(())
    }
}
