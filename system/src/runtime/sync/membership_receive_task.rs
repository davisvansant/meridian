use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;
use crate::JoinClusterRequest;

pub type ChannelMembershipReceiveAction = Sender<MembershipReceiveAction>;

#[derive(Clone, Debug)]
pub enum MembershipReceiveAction {
    JoinClusterRequest(JoinClusterRequest),
    Node,
    Members,
}

pub async fn build_channel() -> (
    ChannelMembershipReceiveAction,
    ChannelMembershipReceiveAction,
    ChannelMembershipReceiveAction,
) {
    let (membership_receive_action, _) = channel(64);
    let membership_grpc_send_membership_action = membership_receive_action.clone();
    let server_send_membership_action = membership_receive_action.clone();

    (
        membership_receive_action,
        membership_grpc_send_membership_action,
        server_send_membership_action,
    )
}
