use crate::node::Node;
use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;

pub type ChannelMembershipSendPreflightTask = Sender<MembershipSendPreflightTask>;

#[derive(Clone, Debug)]
pub enum MembershipSendPreflightTask {
    NodeResponse(Node),
    MembersResponse(Vec<Node>),
}

pub async fn build_channel() -> (
    ChannelMembershipSendPreflightTask,
    ChannelMembershipSendPreflightTask,
) {
    let (membership_send_preflight_task, _) = channel(64);
    let preflight_receive_membership_task = membership_send_preflight_task.clone();

    (
        membership_send_preflight_task,
        preflight_receive_membership_task,
    )
}
