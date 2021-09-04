use crate::meridian_cluster_v010::AppendEntriesRequest;
use crate::server::Server;

use self::volatile_state::VolatileState;

mod volatile_state;

pub struct Leader {
    pub(super) volatile_state: VolatileState,
}

impl Leader {
    pub async fn init() -> Result<Leader, Box<dyn std::error::Error>> {
        let volatile_state = VolatileState::init().await?;

        Ok(Leader { volatile_state })
    }

    pub async fn send_heartbeat(
        &self,
        _request: AppendEntriesRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("sending heartbeat...");
        Ok(())
    }
}

impl Server {
    pub(crate) async fn build_append_entires_request(
        &self,
        leader_volatile_state: &VolatileState,
    ) -> Result<AppendEntriesRequest, Box<dyn std::error::Error>> {
        let term = self.persistent_state.current_term;
        let leader_id = String::from("some_leader_id");
        let prev_log_index = leader_volatile_state.next_index;
        let prev_log_term = leader_volatile_state.match_index;
        let entries = Vec::with_capacity(0);
        let leader_commit = self.volatile_state.commit_index;

        let request = AppendEntriesRequest {
            term,
            leader_id,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit,
        };

        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_leader = Leader::init().await?;
        assert_eq!(test_leader.volatile_state.next_index, 0);
        assert_eq!(test_leader.volatile_state.match_index, 0);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn send_heartbeat() -> Result<(), Box<dyn std::error::Error>> {
        let test_server = Server::init().await?;
        let test_leader_volatile_state = VolatileState::init().await?;
        let test_request = test_server
            .build_append_entires_request(&test_leader_volatile_state)
            .await?;
        let test_leader = Leader::init().await?;
        assert!(test_leader.send_heartbeat(test_request).await.is_ok());
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn build_append_entires_request() -> Result<(), Box<dyn std::error::Error>> {
        let test_server = Server::init().await?;
        let test_leader_volatile_state = VolatileState::init().await?;
        let test_request = test_server
            .build_append_entires_request(&test_leader_volatile_state)
            .await?;
        assert_eq!(test_request.term, 0);
        assert_eq!(test_request.leader_id.as_str(), "some_leader_id");
        assert_eq!(test_request.prev_log_index, 0);
        assert_eq!(test_request.prev_log_term, 0);
        assert_eq!(test_request.entries.len(), 0);
        assert_eq!(test_request.leader_commit, 0);
        Ok(())
    }
}
