use crate::membership::ClusterSize;
use crate::membership::Membership;
use crate::node::Node;
use crate::runtime::sync::launch::ChannelLaunch;
use crate::runtime::sync::membership_receive_task::ChannelMembershipReceiveAction;
use crate::runtime::sync::membership_send_grpc_task::ChannelMembershipSendGrpcAction;
use crate::runtime::sync::membership_send_server_task::ChannelMembershipSendServerAction;
use crate::runtime::tasks::JoinHandle;

pub async fn run_task(
    cluster_size: ClusterSize,
    server: Node,
    peers: Vec<String>,
    membership_send_grpc_actions: ChannelMembershipReceiveAction,
    membership_receive_grpc_actions: ChannelMembershipSendGrpcAction,
    membership_send_server_action: ChannelMembershipSendServerAction,
    launch_action: ChannelLaunch,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    // let server = self.to_owned();
    let mut membership = Membership::init(
        // ClusterSize::One,
        cluster_size,
        server,
        membership_send_grpc_actions,
        membership_receive_grpc_actions,
        membership_send_server_action,
    )
    .await?;

    let membership_run_handle = tokio::spawn(async move {
        // sleep(Duration::from_secs(10)).await;

        if let Err(error) = membership.run(peers, launch_action).await {
            println!("error with running {:?}", error);
        };
    });

    Ok(membership_run_handle)
}
