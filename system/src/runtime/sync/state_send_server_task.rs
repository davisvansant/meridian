use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;
use crate::AppendEntriesRequest;
use crate::RequestVoteRequest;

pub type ChannelStateSendServerTask = Sender<StateSendServerTask>;

#[derive(Clone, Debug)]
pub enum StateSendServerTask {
    AppendEntriesRequest(AppendEntriesRequest),
    RequestVoteRequest(RequestVoteRequest),
    Follower,
}

pub async fn build_channel() -> (ChannelStateSendServerTask, ChannelStateSendServerTask) {
    let (state_receive_server_task, _) = channel(64);
    let state_send_server_task = state_receive_server_task.clone();

    (state_receive_server_task, state_send_server_task)
}
