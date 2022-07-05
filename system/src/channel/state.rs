use std::fmt;
use tokio::sync::{mpsc, oneshot};

use crate::rpc::append_entries::{AppendEntriesArguments, AppendEntriesResults};
use crate::rpc::request_vote::{RequestVoteArguments, RequestVoteResults};

pub type StateReceiver = mpsc::Receiver<(StateRequest, oneshot::Sender<StateResponse>)>;
pub type StateSender = mpsc::Sender<(StateRequest, oneshot::Sender<StateResponse>)>;

#[derive(Clone, Debug)]
pub enum StateRequest {
    AppendEntries(AppendEntriesArguments),
    AppendEntriesResults(AppendEntriesResults),
    RequestVote(RequestVoteArguments),
    RequestVoteResults(RequestVoteResults),
    Candidate(String),
    Leader,
    Heartbeat(String),
    Shutdown,
}

impl StateRequest {
    pub async fn build() -> (StateSender, StateReceiver) {
        let (sender, receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        (sender, receiver)
    }

    pub async fn append_entries_arguments(
        state: &StateSender,
        arguments: AppendEntriesArguments,
    ) -> Result<AppendEntriesResults, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        state
            .send((StateRequest::AppendEntries(arguments), request))
            .await?;

        match response.await? {
            StateResponse::AppendEntries(results) => Ok(results),
            _ => Err(Box::from("Unexpected invoke append entries response!")),
        }
    }

    pub async fn append_entries_results(
        state: &StateSender,
        results: AppendEntriesResults,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        state
            .send((StateRequest::AppendEntriesResults(results), request))
            .await?;

        match response.await? {
            StateResponse::Follower(transition) => Ok(transition),
            _ => Err(Box::from("unexpected receive append entries response!")),
        }
    }

    pub async fn request_vote_arguments(
        state: &StateSender,
        arguments: RequestVoteArguments,
    ) -> Result<RequestVoteResults, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        state
            .send((StateRequest::RequestVote(arguments), request))
            .await?;

        match response.await? {
            StateResponse::RequestVote(results) => Ok(results),
            _ => Err(Box::from("unexpected invoke Request Vote response!")),
        }
    }

    pub async fn request_vote_results(
        state: &StateSender,
        results: RequestVoteResults,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        state
            .send((StateRequest::RequestVoteResults(results), request))
            .await?;

        match response.await? {
            StateResponse::Follower(transition) => Ok(transition),
            _ => Err(Box::from("unexpected receive request vote response!")),
        }
    }

    pub async fn candidate(
        state: &StateSender,
        candidate_id: String,
    ) -> Result<RequestVoteArguments, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        state
            .send((StateRequest::Candidate(candidate_id), request))
            .await?;

        match response.await? {
            StateResponse::Candidate(request_vote_arguments) => Ok(request_vote_arguments),
            _ => Err(Box::from("Unexpected Candidate Response!")),
        }
    }

    pub async fn heartbeat(
        state: &StateSender,
        leader_id: String,
    ) -> Result<AppendEntriesArguments, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        state
            .send((StateRequest::Heartbeat(leader_id), request))
            .await?;

        match response.await? {
            StateResponse::Heartbeat(heartbeat) => Ok(heartbeat),
            _ => Err(Box::from("unexpected heartbeat response!")),
        }
    }

    pub async fn init_leader(state: &StateSender) -> Result<(), Box<dyn std::error::Error>> {
        let (request, _response) = oneshot::channel();

        state.send((StateRequest::Leader, request)).await?;

        Ok(())
    }

    pub async fn shutdown(state: &StateSender) -> Result<(), Box<dyn std::error::Error>> {
        let (_request, _response) = oneshot::channel();

        state.send((StateRequest::Shutdown, _request)).await?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum StateResponse {
    AppendEntries(AppendEntriesResults),
    RequestVote(RequestVoteResults),
    Candidate(RequestVoteArguments),
    Heartbeat(AppendEntriesArguments),
    Follower(bool),
}

impl fmt::Display for StateResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_response = "State Response |";

        match self {
            StateResponse::AppendEntries(results) => {
                write!(f, "{} append entries -> {:?}", state_response, results)
            }
            StateResponse::RequestVote(results) => {
                write!(f, "{} request vote -> {:?}", state_response, results)
            }
            StateResponse::Candidate(arguments) => {
                write!(f, "{}, candidate -> {:?}", state_response, arguments)
            }
            StateResponse::Heartbeat(arguments) => {
                write!(f, "{} heartbeat -> {:?}", state_response, arguments)
            }
            StateResponse::Follower(result) => {
                write!(f, "{} follower -> {}", state_response, result)
            }
        }
    }
}

impl std::error::Error for StateResponse {}
