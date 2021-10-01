use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;
use crate::JoinClusterResponse;

pub type ChannelMembershipSendGrpcTask = Sender<MembershipSendGrpcTask>;

#[derive(Clone, Debug)]
pub enum MembershipSendGrpcTask {
    JoinClusterResponse(JoinClusterResponse),
}

pub async fn build_channel() -> (ChannelMembershipSendGrpcTask, ChannelMembershipSendGrpcTask) {
    let (membership_send_membership_grpc_task, _) = channel(64);
    let membership_grpc_receive_membership_task = membership_send_membership_grpc_task.clone();

    (
        membership_send_membership_grpc_task,
        membership_grpc_receive_membership_task,
    )
}
