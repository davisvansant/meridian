use crate::channel::{StateReceiver, StateRequest, StateResponse};
use crate::rpc::append_entries::{AppendEntriesArguments, AppendEntriesResults};
use crate::rpc::request_vote::{RequestVoteArguments, RequestVoteResults};

mod leader_volatile;
mod persistent;
mod volatile;

use crate::state::leader_volatile::LeaderVolatile;
use crate::state::persistent::Persistent;
use crate::state::volatile::Volatile;

pub struct State {
    persistent: Persistent,
    volatile: Volatile,
    leader_volatile: Option<LeaderVolatile>,
    receiver: StateReceiver,
}

impl State {
    pub async fn init(receiver: StateReceiver) -> Result<State, Box<dyn std::error::Error>> {
        let persistent = Persistent::init().await?;
        let volatile = Volatile::init().await?;

        Ok(State {
            persistent,
            volatile,
            leader_volatile: None,
            receiver,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                StateRequest::AppendEntries(arguments) => {
                    println!("received append entires request - {:?}", &arguments);

                    let results = AppendEntriesResults {
                        term: 0,
                        success: false,
                    };

                    if let Err(error) = response.send(StateResponse::AppendEntries(results)) {
                        println!("state > error sending append entries response {:?}", error);
                    }
                }
                StateRequest::Candidate(candidate_id) => {
                    self.persistent.increment_current_term().await?;
                    self.persistent.vote_for_self(candidate_id.as_str()).await?;

                    let arguments = self.build_request_vote_arguments(&candidate_id).await?;

                    if let Err(error) = response.send(StateResponse::Candidate(arguments)) {
                        println!("state > error sending request vote arguments {:?}", error);
                    }
                }
                StateRequest::Heartbeat(leader_id) => {
                    let heartbeat = self.heartbeat(leader_id).await?;

                    if let Err(error) = response.send(StateResponse::Heartbeat(heartbeat)) {
                        println!("state > error sending heartbeat arguments {:?}", error);
                    }
                }
                StateRequest::Leader => {
                    self.init_leader_volatile_state().await?;
                }
                StateRequest::RequestVote(arguments) => {
                    println!("received request vote {:?}", &arguments);

                    let results = self.request_vote(arguments).await?;

                    if let Err(error) = response.send(StateResponse::RequestVote(results)) {
                        println!("state > error sending request vote response {:?}", error);
                    }
                }
            }
        }

        Ok(())
    }

    async fn append_entries(
        &self,
        request: AppendEntriesArguments,
    ) -> Result<AppendEntriesResults, Box<dyn std::error::Error>> {
        let true_response = AppendEntriesResults {
            term: self.persistent.current_term,
            success: true,
        };
        let false_response = AppendEntriesResults {
            term: self.persistent.current_term,
            success: false,
        };

        match self.check_term(request.term).await {
            false => Ok(false_response),
            true => {
                match self
                    .check_candidate_log(self.persistent.current_term, request.prev_log_term)
                    .await
                {
                    false => Ok(false_response),
                    true => {
                        // do some delete stuff here
                        // do some appending stuff here
                        // do some setting of commit_index here
                        Ok(true_response)
                    }
                }
            }
        }
    }

    async fn heartbeat(
        &self,
        leader_id: String,
    ) -> Result<AppendEntriesArguments, Box<dyn std::error::Error>> {
        let term = self.persistent.current_term;
        let prev_log_index = 0;
        let prev_log_term = 0;
        let entries = Vec::with_capacity(0);
        let leader_commit = self.volatile.commit_index;

        let heartbeat = AppendEntriesArguments {
            term,
            leader_id,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit,
        };

        Ok(heartbeat)
    }

    async fn request_vote(
        &self,
        request: RequestVoteArguments,
    ) -> Result<RequestVoteResults, Box<dyn std::error::Error>> {
        let true_response = RequestVoteResults {
            term: self.persistent.current_term,
            vote_granted: true,
        };

        let false_response = RequestVoteResults {
            term: self.persistent.current_term,
            vote_granted: false,
        };

        match self.check_term(request.term).await {
            false => Ok(false_response),
            true => {
                match self.check_candidate_id(request.candidate_id.as_str()).await
                    && self
                        .check_candidate_log(self.persistent.current_term, request.last_log_term)
                        .await
                {
                    true => Ok(true_response),
                    false => Ok(false_response),
                }
            }
        }
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

    async fn build_request_vote_arguments(
        &self,
        candidate_id: &str,
    ) -> Result<RequestVoteArguments, Box<dyn std::error::Error>> {
        let term = self.persistent.current_term;
        let candidate_id = String::from(candidate_id);
        let last_log_index = self.persistent.log.len() as u32;
        let last_log_term = if let Some(log) = self.persistent.log.last() {
            log.term
        } else {
            0
        };

        let request_vote_arguments = RequestVoteArguments {
            term,
            candidate_id,
            last_log_index,
            last_log_term,
        };

        Ok(request_vote_arguments)
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
