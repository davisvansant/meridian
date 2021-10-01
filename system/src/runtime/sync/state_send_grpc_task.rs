use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;
use crate::AppendEntriesResponse;
use crate::RequestVoteResponse;

pub type ChannelStateSendGrpcTask = Sender<StateSendGrpcTask>;

#[derive(Clone, Debug)]
pub enum StateSendGrpcTask {
    AppendEntriesResponse(AppendEntriesResponse),
    RequestVoteResponse(RequestVoteResponse),
}

pub async fn build_channel() -> (ChannelStateSendGrpcTask, ChannelStateSendGrpcTask) {
    let (state_receive_grpc_task, _) = channel(64);
    let state_send_grpc_task = state_receive_grpc_task.clone();

    (state_receive_grpc_task, state_send_grpc_task)
}
