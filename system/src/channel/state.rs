use tokio::sync::{mpsc, oneshot};

use crate::rpc::append_entries::{AppendEntriesArguments, AppendEntriesResults};
use crate::rpc::request_vote::{RequestVoteArguments, RequestVoteResults};

pub type StateReceiver = mpsc::Receiver<(StateRequest, oneshot::Sender<StateResponse>)>;
pub type StateSender = mpsc::Sender<(StateRequest, oneshot::Sender<StateResponse>)>;

#[derive(Clone, Debug)]
pub enum StateRequest {
    AppendEntries(AppendEntriesArguments),
    RequestVote(RequestVoteArguments),
    Candidate(String),
    Leader,
    Heartbeat(String),
}

#[derive(Clone, Debug)]
pub enum StateResponse {
    AppendEntries(AppendEntriesResults),
    RequestVote(RequestVoteResults),
    Follower,
    Candidate(RequestVoteArguments),
    Heartbeat(AppendEntriesArguments),
}

pub async fn candidate(
    state: &StateSender,
    candidate_id: String,
) -> Result<RequestVoteArguments, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    state
        .send((StateRequest::Candidate(candidate_id), request))
        .await?;

    match response.await {
        Ok(StateResponse::Candidate(request_vote_arguments)) => Ok(request_vote_arguments),
        Err(error) => Err(Box::new(error)),
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

    match response.await {
        Ok(StateResponse::RequestVote(results)) => Ok(results),
        Err(error) => Err(Box::new(error)),
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

    match response.await {
        Ok(StateResponse::AppendEntries(results)) => Ok(results),
        Err(error) => Err(Box::new(error)),
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

    match response.await {
        Ok(StateResponse::Heartbeat(heartbeat)) => Ok(heartbeat),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn leader(state: &StateSender) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    state.send((StateRequest::Leader, request)).await?;

    Ok(())
}
