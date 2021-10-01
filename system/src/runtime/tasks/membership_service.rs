use crate::membership::ClusterSize;
use crate::membership::Membership;
use crate::node::Node;
use crate::runtime::sync::launch::ChannelLaunch;
use crate::runtime::sync::membership_receive_task::ChannelMembershipReceiveTask;
use crate::runtime::sync::membership_send_grpc_task::ChannelMembershipSendGrpcTask;
use crate::runtime::sync::membership_send_server_task::ChannelMembershipSendServerTask;
use crate::runtime::tasks::JoinHandle;

pub async fn run_task(
    cluster_size: ClusterSize,
    server: Node,
    peers: Vec<String>,
    membership_send_grpc_task: ChannelMembershipReceiveTask,
    membership_receive_grpc_task: ChannelMembershipSendGrpcTask,
    membership_send_server_task: ChannelMembershipSendServerTask,
    launch_action: ChannelLaunch,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    // let server = self.to_owned();
    let mut membership = Membership::init(
        // ClusterSize::One,
        cluster_size,
        server,
        membership_send_grpc_task,
        membership_receive_grpc_task,
        membership_send_server_task,
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
