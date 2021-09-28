use crate::state::leader_volatile::LeaderVolatile;
use crate::state::persistent::Persistent;
use crate::state::volatile::Volatile;

mod leader_volatile;
mod persistent;
mod volatile;

use crate::meridian_cluster_v010::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    RequestVoteRequest, RequestVoteResponse,
};

use crate::channels::ChannelStateReceiveAction;
use crate::channels::ChannelStateSendGrpcAction;
use crate::channels::ChannelStateSendServerAction;

use crate::channels::StateReceiveAction;
use crate::channels::StateSendGrpcAction;
use crate::channels::StateSendServerAction;

pub struct State {
    persistent: Persistent,
    volatile: Volatile,
    leader_volatile: Option<LeaderVolatile>,
    receive_actions: ChannelStateReceiveAction,
    send_server_actions: ChannelStateSendServerAction,
    send_grpc_actions: ChannelStateSendGrpcAction,
}

impl State {
    pub async fn init(
        receive_actions: ChannelStateReceiveAction,
        send_server_actions: ChannelStateSendServerAction,
        send_grpc_actions: ChannelStateSendGrpcAction,
    ) -> Result<State, Box<dyn std::error::Error>> {
        let persistent = Persistent::init().await?;
        let volatile = Volatile::init().await?;

        Ok(State {
            persistent,
            volatile,
            leader_volatile: None,
            receive_actions,
            send_server_actions,
            send_grpc_actions,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut receiver = self.receive_actions.subscribe();

        while let Ok(action) = receiver.recv().await {
            println!("received action! {:?}", &action);
            match action {
                StateReceiveAction::AppendEntriesRequest(request) => {
                    println!("received append entires request - {:?}", &request);
                    if request.entries.is_empty() {
                        self.send_server_actions
                            .send(StateSendServerAction::Follower)?;
                    }
                    self.append_entries_receiver(request).await?;
                }
                StateReceiveAction::RequestVoteRequest(request) => {
                    println!("do things with request votes!");
                    self.request_vote_receiver(request).await?;
                }
                StateReceiveAction::Candidate(candidate_id) => {
                    self.persistent.increment_current_term().await?;
                    self.persistent.vote_for_self(candidate_id.as_str()).await?;

                    let request_vote_request = self
                        .build_request_vote_request(candidate_id.as_str())
                        .await?;

                    self.send_server_actions
                        .send(StateSendServerAction::RequestVoteRequest(
                            request_vote_request,
                        ))?;
                }
                StateReceiveAction::Leader(leader_id) => {
                    self.init_leader_volatile_state().await?;
                    let append_entries_request =
                        self.build_append_entries_request(leader_id).await?;

                    self.send_server_actions
                        .send(StateSendServerAction::AppendEntriesRequest(
                            append_entries_request,
                        ))?;
                }
            }
        }

        Ok(())
    }

    async fn append_entries_receiver(
        &self,
        request: AppendEntriesRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let true_response = AppendEntriesResponse {
            term: self.persistent.current_term,
            success: String::from("true"),
        };
        let false_response = AppendEntriesResponse {
            term: self.persistent.current_term,
            success: String::from("false"),
        };

        match self.check_term(request.term).await {
            false => {
                self.send_grpc_actions
                    .send(StateSendGrpcAction::AppendEntriesResponse(false_response))?;
            }
            true => {
                match self
                    .check_candidate_log(self.persistent.current_term, request.prev_log_term)
                    .await
                {
                    false => {
                        self.send_grpc_actions
                            .send(StateSendGrpcAction::AppendEntriesResponse(false_response))?;
                    }
                    true => {
                        // do some delete stuff here
                        // do some appending stuff here
                        // do some setting of commit_index here
                        self.send_grpc_actions
                            .send(StateSendGrpcAction::AppendEntriesResponse(true_response))?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn request_vote_receiver(
        &self,
        request: RequestVoteRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let true_response = RequestVoteResponse {
            term: self.persistent.current_term,
            vote_granted: String::from("true"),
        };

        let false_response = RequestVoteResponse {
            term: self.persistent.current_term,
            vote_granted: String::from("false"),
        };

        match self.check_term(request.term).await {
            false => {
                self.send_grpc_actions
                    .send(StateSendGrpcAction::RequestVoteResponse(false_response))?;
            }
            true => {
                match self.check_candidate_id(request.candidate_id.as_str()).await
                    && self
                        .check_candidate_log(self.persistent.current_term, request.last_log_term)
                        .await
                {
                    true => {
                        self.send_grpc_actions
                            .send(StateSendGrpcAction::RequestVoteResponse(true_response))?;
                    }
                    false => {
                        self.send_grpc_actions
                            .send(StateSendGrpcAction::RequestVoteResponse(false_response))?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn check_term(&self, term: u32) -> bool {
        let current_term = 0;
        term > current_term
    }

    async fn check_candidate_id(&self, candidate_id: &str) -> bool {
        self.persistent.voted_for == None
            || self.persistent.voted_for == Some(candidate_id.to_string())
    }

    async fn check_candidate_log(&self, log: u32, candidate_log: u32) -> bool {
        log >= candidate_log
    }

    async fn build_append_entries_request(
        &self,
        leader_id: String,
    ) -> Result<AppendEntriesRequest, Box<dyn std::error::Error>> {
        let term = self.persistent.current_term;
        // let leader_id = String::from("some_leader_id");
        // let prev_log_index = self.persistent.next_index;
        // let prev_log_term = self.persistent.match_index;
        let prev_log_index = 0;
        let prev_log_term = 0;
        let entries = Vec::with_capacity(0);
        let leader_commit = self.volatile.commit_index;

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

    async fn build_request_vote_request(
        &self,
        candidate_id: &str,
    ) -> Result<RequestVoteRequest, Box<dyn std::error::Error>> {
        let term = self.persistent.current_term;
        let candidate_id = String::from(candidate_id);
        let last_log_index = self.persistent.log.len() as u32;
        let last_log_term = if let Some(log) = self.persistent.log.last() {
            log.term
        } else {
            0
        };

        let request_vote_request = RequestVoteRequest {
            term,
            candidate_id,
            last_log_index,
            last_log_term,
        };

        Ok(request_vote_request)
    }

    async fn init_leader_volatile_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.leader_volatile.is_none() {
            println!("initialing leader volatile state ...");
            let leader_volatile = LeaderVolatile::init().await?;
            self.leader_volatile = Some(leader_volatile);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[tokio::test(flavor = "multi_thread")]
    // async fn log_entry() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_log_entry = LogEntry {
    //         term: 0,
    //         command: String::from("test_log_entry"),
    //         committed: true,
    //     };
    //     assert_eq!(test_log_entry.term, 0);
    //     assert_eq!(test_log_entry.command.as_str(), "test_log_entry");
    //     assert!(test_log_entry.committed);
    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn init() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_state = State::init().await?;
    //     assert_eq!(test_state.persistent.current_term, 0);
    //     assert_eq!(test_state.persistent.voted_for, None);
    //     assert_eq!(test_state.persistent.log.len(), 0);
    //     assert_eq!(test_state.persistent.log.capacity(), 4096);
    //     Ok(())
    // }
}
