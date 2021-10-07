use crate::runtime::sync::channel;
use crate::runtime::sync::Sender;
use crate::MembershipNode;

pub type ChannelMembershipReceiveTask = Sender<MembershipReceiveTask>;

#[derive(Clone, Debug)]
pub enum MembershipReceiveTask {
    JoinCluster((u8, MembershipNode)),
    Node(u8),
    Members(u8),
    Status,
}

pub async fn build_channel() -> (
    ChannelMembershipReceiveTask,
    ChannelMembershipReceiveTask,
    ChannelMembershipReceiveTask,
    ChannelMembershipReceiveTask,
) {
    let (membership_receive_task, _) = channel(64);
    let membership_grpc_send_membership_task = membership_receive_task.clone();
    let preflight_send_membership_task = membership_receive_task.clone();
    let server_send_membership_task = membership_receive_task.clone();

    (
        membership_receive_task,
        membership_grpc_send_membership_task,
        preflight_send_membership_task,
        server_send_membership_task,
    )
}
