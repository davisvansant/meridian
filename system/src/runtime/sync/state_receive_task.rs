use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;
use crate::AppendEntriesRequest;
use crate::RequestVoteRequest;

pub type ChannelStateReceiveTask = Sender<StateReceiveTask>;

#[derive(Clone, Debug)]
pub enum StateReceiveTask {
    AppendEntriesRequest(AppendEntriesRequest),
    RequestVoteRequest(RequestVoteRequest),
    Candidate(String),
    Leader(String),
}

pub async fn build_channel() -> (
    ChannelStateReceiveTask,
    ChannelStateReceiveTask,
    ChannelStateReceiveTask,
) {
    let (state_send_task, _) = channel(64);
    let grpc_send_task = state_send_task.clone();
    let server_send_task = state_send_task.clone();

    (state_send_task, grpc_send_task, server_send_task)
}
