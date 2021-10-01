use crate::node::Node;
use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;

pub type ChannelMembershipSendServerAction = Sender<MembershipSendServerAction>;

#[derive(Clone, Debug)]
pub enum MembershipSendServerAction {
    NodeResponse(Node),
    MembersResponse(Vec<Node>),
}

pub async fn build_channel() -> (
    ChannelMembershipSendServerAction,
    ChannelMembershipSendServerAction,
) {
    let (membership_send_server_action, _) = channel(64);
    let server_receive_membership_action = membership_send_server_action.clone();

    (
        membership_send_server_action,
        server_receive_membership_action,
    )
}
