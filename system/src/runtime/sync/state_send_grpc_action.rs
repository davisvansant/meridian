use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;
use crate::AppendEntriesResponse;
use crate::RequestVoteResponse;

pub type ChannelStateSendGrpcAction = Sender<StateSendGrpcAction>;

#[derive(Clone, Debug)]
pub enum StateSendGrpcAction {
    AppendEntriesResponse(AppendEntriesResponse),
    RequestVoteResponse(RequestVoteResponse),
}

pub async fn build_channel() -> (ChannelStateSendGrpcAction, ChannelStateSendGrpcAction) {
    let (state_receive_grpc_actions, _) = channel(64);
    let state_send_grpc_actions = state_receive_grpc_actions.clone();

    (state_receive_grpc_actions, state_send_grpc_actions)
}
