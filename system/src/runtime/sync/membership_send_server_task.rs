use crate::node::Node;
use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;

pub type ChannelMembershipSendServerTask = Sender<MembershipSendServerTask>;

#[derive(Clone, Debug)]
pub enum MembershipSendServerTask {
    NodeResponse(Node),
    MembersResponse(Vec<Node>),
}

pub async fn build_channel() -> (
    ChannelMembershipSendServerTask,
    ChannelMembershipSendServerTask,
) {
    let (membership_send_server_task, _) = channel(64);
    let server_receive_membership_task = membership_send_server_task.clone();

    (membership_send_server_task, server_receive_membership_task)
}
