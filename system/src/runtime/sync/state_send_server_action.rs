use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;
use crate::AppendEntriesRequest;
use crate::RequestVoteRequest;

pub type ChannelStateSendServerAction = Sender<StateSendServerAction>;

#[derive(Clone, Debug)]
pub enum StateSendServerAction {
    AppendEntriesRequest(AppendEntriesRequest),
    RequestVoteRequest(RequestVoteRequest),
    Follower,
}

pub async fn build_channel() -> (ChannelStateSendServerAction, ChannelStateSendServerAction) {
    let (state_receive_server_actions, _) = channel(64);
    let state_send_server_actions = state_receive_server_actions.clone();

    (state_receive_server_actions, state_send_server_actions)
}
