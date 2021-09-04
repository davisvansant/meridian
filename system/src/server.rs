use tokio::time::timeout;

use crate::server::candidate::Candidate;
use crate::server::follower::Follower;
use crate::server::leader::Leader;

mod candidate;
mod follower;
mod leader;

#[derive(Debug, PartialEq)]
pub enum ServerState {
    Follower,
    Candidate,
    Leader,
}

pub struct LogEntry {
    term: u32,
    command: String,
    committed: bool,
}

pub struct PersistentState {
    current_term: u32,
    voted_for: String,
    log: Vec<LogEntry>,
}

pub struct VolatileState {
    commit_index: u32,
    last_applied: u32,
}

pub struct Server {
    pub server_state: ServerState,
    persistent_state: PersistentState,
    volatile_state: VolatileState,
}

impl Server {
    pub async fn init() -> Result<Server, Box<dyn std::error::Error>> {
        let server_state = ServerState::Follower;
        let persistent_state = PersistentState {
            current_term: 0,
            voted_for: String::with_capacity(10),
            log: Vec::with_capacity(4096),
        };
        let volatile_state = VolatileState {
            commit_index: 0,
            last_applied: 0,
        };

        Ok(Server {
            server_state,
            persistent_state,
            volatile_state,
        })
    }

    pub async fn follower(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let follower = Follower::init().await?;

        if timeout(follower.election_timeout, follower.election_timeout())
            .await
            .is_ok()
        {
            self.server_state = ServerState::Candidate;
        }

        Ok(())
    }

    pub async fn candidate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let candidate = Candidate::init().await?;

        self.increment_current_term().await?;

        let start_election = candidate.start_election();

        if timeout(candidate.election_timeout, start_election)
            .await
            .is_ok()
        {
            self.server_state = ServerState::Leader;
        }

        Ok(())
    }

    pub async fn leader(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let leader = Leader::init().await?;
        let request = self
            .build_append_entires_request(&leader.volatile_state)
            .await?;

        leader.send_heartbeat(request).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn server_state_candidate() -> Result<(), Box<dyn std::error::Error>> {
        let test_server_state = ServerState::Candidate;
        assert_eq!(test_server_state, ServerState::Candidate);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn server_state_follower() -> Result<(), Box<dyn std::error::Error>> {
        let test_server_state = ServerState::Follower;
        assert_eq!(test_server_state, ServerState::Follower);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn server_state_leader() -> Result<(), Box<dyn std::error::Error>> {
        let test_server_state = ServerState::Leader;
        assert_eq!(test_server_state, ServerState::Leader);
        Ok(())
    }

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
    async fn persistent_state() -> Result<(), Box<dyn std::error::Error>> {
        let test_persistent_state = PersistentState {
            current_term: 0,
            voted_for: String::from("some_test_voted_for"),
            log: Vec::with_capacity(0),
        };
        assert_eq!(test_persistent_state.current_term, 0);
        assert_eq!(
            test_persistent_state.voted_for.as_str(),
            "some_test_voted_for",
        );
        assert_eq!(test_persistent_state.log.len(), 0);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn volatile_state() -> Result<(), Box<dyn std::error::Error>> {
        let test_volatile_state = VolatileState {
            commit_index: 0,
            last_applied: 0,
        };
        assert_eq!(test_volatile_state.commit_index, 0);
        assert_eq!(test_volatile_state.last_applied, 0);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_server = Server::init().await?;
        assert_eq!(test_server.server_state, ServerState::Follower);
        assert_eq!(test_server.persistent_state.current_term, 0);
        assert_eq!(test_server.persistent_state.voted_for.as_str(), "");
        assert_eq!(test_server.persistent_state.log.capacity(), 4096);
        Ok(())
    }
}
