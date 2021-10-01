use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;
use crate::AppendEntriesRequest;
use crate::RequestVoteRequest;

pub type ChannelStateReceiveAction = Sender<StateReceiveAction>;

#[derive(Clone, Debug)]
pub enum StateReceiveAction {
    AppendEntriesRequest(AppendEntriesRequest),
    RequestVoteRequest(RequestVoteRequest),
    Candidate(String),
    Leader(String),
}

pub async fn build_channel() -> (
    ChannelStateReceiveAction,
    ChannelStateReceiveAction,
    ChannelStateReceiveAction,
) {
    let (state_send_handle, _) = channel(64);
    let grpc_send_actions = state_send_handle.clone();
    let server_send_actions = state_send_handle.clone();

    (state_send_handle, grpc_send_actions, server_send_actions)
}
