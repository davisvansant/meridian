use tokio::time::timeout;

use crate::server::candidate::Candidate;
use crate::server::follower::Follower;
use crate::server::leader::Leader;
use crate::server::persistent_state::PersistentState;
use crate::server::volatile_state::VolatileState;

mod candidate;
mod follower;
mod leader;
mod persistent_state;
mod volatile_state;

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

pub struct Server {
    pub server_state: ServerState,
    persistent_state: PersistentState,
    volatile_state: VolatileState,
}

impl Server {
    pub async fn init() -> Result<Server, Box<dyn std::error::Error>> {
        let server_state = ServerState::Follower;
        let persistent_state = PersistentState::init().await?;
        let volatile_state = VolatileState::init().await?;

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

        let request = self.build_request_vote_request().await?;
        let start_election = candidate.start_election(request);

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
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_server = Server::init().await?;
        assert_eq!(test_server.server_state, ServerState::Follower);
        assert_eq!(test_server.persistent_state.current_term, 0);
        assert_eq!(test_server.persistent_state.voted_for, None);
        assert_eq!(test_server.persistent_state.log.len(), 0);
        assert_eq!(test_server.persistent_state.log.capacity(), 4096);
        assert_eq!(test_server.volatile_state.commit_index, 0);
        assert_eq!(test_server.volatile_state.last_applied, 0);
        Ok(())
    }
}
