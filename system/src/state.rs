use crate::channel::state::{StateReceiver, StateRequest, StateResponse};
use crate::rpc::append_entries::{AppendEntriesArguments, AppendEntriesResults};
use crate::rpc::request_vote::{RequestVoteArguments, RequestVoteResults};
use crate::{error, info, warn};

use leader_volatile::LeaderVolatile;
use persistent::Persistent;
use volatile::Volatile;

mod leader_volatile;
mod persistent;
mod volatile;

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

        info!("initialized!");

        Ok(State {
            persistent,
            volatile,
            leader_volatile: None,
            receiver,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("running!");

        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                StateRequest::AppendEntries(arguments) => {
                    info!("received append entires request - {:?}", &arguments);

                    let results = self.append_entries(arguments).await?;

                    if let Err(error) = response.send(StateResponse::AppendEntries(results)) {
                        error!("append entries response -> {:?}", error);
                    }
                }
                StateRequest::AppendEntriesResults(results) => {
                    info!("received append entries results -> {:?}", &results);

                    match self.check_term(results.term).await {
                        true => {
                            if let Err(error) = response.send(StateResponse::Follower(false)) {
                                error!("check term true response -> {:?}", error);
                            }
                        }
                        false => {
                            if let Err(error) = response.send(StateResponse::Follower(true)) {
                                error!("check term false response -> {:?}", error);
                            }
                        }
                    }
                }
                StateRequest::Candidate(candidate_id) => {
                    self.persistent.increment_current_term().await?;
                    self.persistent.vote_for_self(candidate_id.as_str()).await?;

                    let arguments = self.build_request_vote_arguments(&candidate_id).await?;

                    if let Err(error) = response.send(StateResponse::Candidate(arguments)) {
                        error!("sending request vote arguments -> {:?}", error);
                    }
                }
                StateRequest::Heartbeat(leader_id) => {
                    let heartbeat = self.heartbeat(leader_id).await?;

                    if let Err(error) = response.send(StateResponse::Heartbeat(heartbeat)) {
                        println!("sending heartbeat arguments -> {:?}", error);
                    }
                }
                StateRequest::Leader => {
                    self.init_leader_volatile_state().await?;
                }
                StateRequest::RequestVote(arguments) => {
                    info!("received request vote {:?}", &arguments);

                    let results = self.request_vote(arguments).await?;

                    error!("testing request vote results -> {:?}", &results);

                    if let Err(error) = response.send(StateResponse::RequestVote(results)) {
                        error!("sending request vote response -> {:?}", error);
                    }
                }
                StateRequest::RequestVoteResults(results) => {
                    info!("received request vote results -> {:?}", &results);

                    match self.check_term(results.term).await {
                        true => {
                            if let Err(error) = response.send(StateResponse::Follower(true)) {
                                error!("check term true response -> {:?}", error);
                            }
                        }
                        false => {
                            if let Err(error) = response.send(StateResponse::Follower(false)) {
                                error!("check term false response -> {:?}", error);
                            }
                        }
                    }
                }
                StateRequest::Shutdown => {
                    info!("shutting down...");

                    self.receiver.close();
                }
            }
        }

        Ok(())
    }

    async fn append_entries(
        &mut self,
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
        &mut self,
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

    async fn check_term(&mut self, term: u32) -> bool {
        match term.cmp(&self.persistent.current_term) {
            std::cmp::Ordering::Less => {
                info!(
                    "current term higher than incoming -> current {:?} | incoming {:?}",
                    self.persistent.current_term, term,
                );

                false
            }
            std::cmp::Ordering::Equal => {
                info!(
                    "terms are equal! current {:?} | {:?}",
                    self.persistent.current_term, term,
                );

                true
            }
            std::cmp::Ordering::Greater => {
                warn!(
                    "adjusting term! current {:?} | {:?}",
                    self.persistent.current_term, term,
                );

                self.persistent.adjust_term(term).await;

                true
            }
        }
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
            info!("initializing leader volatile state ...");
            let leader_volatile = LeaderVolatile::init().await?;
            self.leader_volatile = Some(leader_volatile);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_sender, test_receiver) = crate::channel::state::build().await;
        let test_state = State::init(test_receiver).await?;

        assert_eq!(test_state.persistent.current_term, 0);
        assert_eq!(test_state.persistent.voted_for, None);
        assert_eq!(test_state.persistent.log.len(), 0);
        assert_eq!(test_state.persistent.log.capacity(), 4096);
        assert!(!test_sender.is_closed());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init_leader_volatile_state() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = crate::channel::state::build().await;
        let mut test_state = State::init(test_receiver).await?;

        assert!(test_state.leader_volatile.is_none());

        test_state.init_leader_volatile_state().await?;

        assert!(test_state.leader_volatile.is_some());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn append_entries_true() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = crate::channel::state::build().await;
        let mut test_state = State::init(test_receiver).await?;

        test_state.persistent.current_term = 1;

        let test_append_entries_arguments = AppendEntriesArguments {
            term: 1,
            leader_id: String::from("some_leader_id"),
            prev_log_index: 0,
            prev_log_term: 0,
            entries: Vec::with_capacity(0),
            leader_commit: 0,
        };

        let test_append_entries_results = test_state
            .append_entries(test_append_entries_arguments)
            .await?;

        assert_eq!(test_append_entries_results.term, 1);
        assert!(test_append_entries_results.success);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn append_entries_false() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = crate::channel::state::build().await;
        let mut test_state = State::init(test_receiver).await?;

        test_state.persistent.current_term = 2;

        let test_append_entries_arguments = AppendEntriesArguments {
            term: 1,
            leader_id: String::from("some_leader_id"),
            prev_log_index: 0,
            prev_log_term: 0,
            entries: Vec::with_capacity(0),
            leader_commit: 0,
        };

        let test_append_entries_results = test_state
            .append_entries(test_append_entries_arguments)
            .await?;

        assert_eq!(test_append_entries_results.term, 2);
        assert!(!test_append_entries_results.success);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn heartbeat() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = crate::channel::state::build().await;
        let test_state = State::init(test_receiver).await?;

        let test_leader_id = String::from("some_leader_id");

        let test_append_entries_arguments = test_state.heartbeat(test_leader_id).await?;

        assert_eq!(test_append_entries_arguments.term, 0);
        assert_eq!(
            test_append_entries_arguments.leader_id.as_str(),
            "some_leader_id",
        );
        assert_eq!(test_append_entries_arguments.prev_log_index, 0);
        assert_eq!(test_append_entries_arguments.prev_log_term, 0);
        assert!(test_append_entries_arguments.entries.is_empty());
        assert_eq!(test_append_entries_arguments.leader_commit, 0);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn request_vote_true() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = crate::channel::state::build().await;
        let mut test_state = State::init(test_receiver).await?;

        test_state.persistent.current_term = 1;

        let test_request_vote_arguments = RequestVoteArguments {
            term: 1,
            candidate_id: String::from("some_candidate_id"),
            last_log_index: 0,
            last_log_term: 0,
        };

        let test_request_vote_results =
            test_state.request_vote(test_request_vote_arguments).await?;

        assert_eq!(test_request_vote_results.term, 1);
        assert!(test_request_vote_results.vote_granted);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn request_vote_false() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = crate::channel::state::build().await;
        let mut test_state = State::init(test_receiver).await?;

        test_state.persistent.current_term = 2;

        let test_request_vote_arguments = RequestVoteArguments {
            term: 1,
            candidate_id: String::from("some_candidate_id"),
            last_log_index: 0,
            last_log_term: 0,
        };

        let test_request_vote_results =
            test_state.request_vote(test_request_vote_arguments).await?;

        assert_eq!(test_request_vote_results.term, 2);
        assert!(!test_request_vote_results.vote_granted);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_term_true() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = crate::channel::state::build().await;
        let mut test_state = State::init(test_receiver).await?;

        assert!(test_state.check_term(0).await);

        test_state.persistent.current_term = 1;

        assert!(test_state.check_term(2).await);
        assert_eq!(test_state.persistent.current_term, 2);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_term_false() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = crate::channel::state::build().await;
        let mut test_state = State::init(test_receiver).await?;

        test_state.persistent.current_term = 1;

        assert!(!test_state.check_term(0).await);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_candidate_id_true() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = crate::channel::state::build().await;
        let test_state = State::init(test_receiver).await?;

        assert!(test_state.check_candidate_id("some_candidate_id").await);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn check_candidate_id_false() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = crate::channel::state::build().await;
        let mut test_state = State::init(test_receiver).await?;

        test_state.persistent.voted_for = Some(String::from("some_test_uuid"));

        assert!(!test_state.check_candidate_id("some_candidate_id").await);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn build_request_vote_arguments() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_sender, test_receiver) = crate::channel::state::build().await;
        let test_state = State::init(test_receiver).await?;

        let test_candidate_id = String::from("some_candidate_id");

        let test_request_vote_arguments = test_state
            .build_request_vote_arguments(&test_candidate_id)
            .await?;

        assert_eq!(test_request_vote_arguments.term, 0);
        assert_eq!(
            test_request_vote_arguments.candidate_id.as_str(),
            "some_candidate_id",
        );
        assert_eq!(test_request_vote_arguments.last_log_index, 0);
        assert_eq!(test_request_vote_arguments.last_log_term, 0);

        Ok(())
    }
}
