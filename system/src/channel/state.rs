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

#[derive(Clone, Debug)]
pub enum StateResponse {
    AppendEntries(AppendEntriesResults),
    RequestVote(RequestVoteResults),
    Candidate(RequestVoteArguments),
    Heartbeat(AppendEntriesArguments),
    Follower(bool),
}

pub async fn build() -> (StateSender, StateReceiver) {
    let (state_sender, state_receiver) =
        mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

    (state_sender, state_receiver)
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
        _ => panic!("unexpected response!"),
    }
}

pub async fn request_vote(
    state: &StateSender,
    arguments: RequestVoteArguments,
) -> Result<RequestVoteResults, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    state
        .send((StateRequest::RequestVote(arguments), request))
        .await?;

    match response.await? {
        StateResponse::RequestVote(results) => Ok(results),
        _ => panic!("unexpected response!"),
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
        _ => panic!("unexpected response!"),
    }
}

pub async fn append_entries(
    state: &StateSender,
    arguments: AppendEntriesArguments,
) -> Result<AppendEntriesResults, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    state
        .send((StateRequest::AppendEntries(arguments), request))
        .await?;

    match response.await? {
        StateResponse::AppendEntries(results) => Ok(results),
        _ => panic!("unexpected response!"),
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
        _ => panic!("unexpected response!"),
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
        _ => panic!("unexpected response!"),
    }
}

pub async fn leader(state: &StateSender) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    state.send((StateRequest::Leader, request)).await?;

    Ok(())
}

pub async fn shutdown(state: &StateSender) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    state.send((StateRequest::Shutdown, request)).await?;

    Ok(())
}
