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

pub struct Server {
    pub server_state: ServerState,
    persistent_state: PersistentState,
}

impl Server {
    pub async fn init() -> Result<Server, Box<dyn std::error::Error>> {
        let server_state = ServerState::Follower;
        let persistent_state = PersistentState {
            current_term: 0,
            voted_for: String::with_capacity(10),
            log: Vec::with_capacity(4096),
        };

        Ok(Server {
            server_state,
            persistent_state,
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
