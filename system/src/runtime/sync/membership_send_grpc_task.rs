use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;
use crate::JoinClusterResponse;

pub type ChannelMembershipSendGrpcAction = Sender<MembershipSendGrpcAction>;

#[derive(Clone, Debug)]
pub enum MembershipSendGrpcAction {
    JoinClusterResponse(JoinClusterResponse),
}

pub async fn build_channel() -> (
    ChannelMembershipSendGrpcAction,
    ChannelMembershipSendGrpcAction,
) {
    let (membership_send_membership_grpc_action, _) = channel(64);
    let membership_grpc_receive_membership_action = membership_send_membership_grpc_action.clone();

    (
        membership_send_membership_grpc_action,
        membership_grpc_receive_membership_action,
    )
}
