use crate::runtime::sync::launch::ChannelLaunch;
use crate::runtime::sync::membership_receive_task::ChannelMembershipReceiveTask;
use crate::runtime::sync::membership_send_server_task::ChannelMembershipSendServerTask;
use crate::runtime::sync::state_receive_task::ChannelStateReceiveTask;
use crate::runtime::sync::state_send_server_task::ChannelStateSendServerTask;
use crate::runtime::tasks::JoinHandle;
use crate::server::Server;

pub async fn run_task(
    state_receive_server_task: ChannelStateSendServerTask,
    server_send_task: ChannelStateReceiveTask,
    server_send_membership_task: ChannelMembershipReceiveTask,
    server_receive_membership_task: ChannelMembershipSendServerTask,
    channel_launch: ChannelLaunch,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    let mut server = Server::init(
        state_receive_server_task,
        server_send_task,
        server_send_membership_task,
        server_receive_membership_task,
    )
    .await?;

    let server_handle = tokio::spawn(async move {
        // sleep(Duration::from_secs(10)).await;
        println!("waiting on membership...");

        let mut receiver = channel_launch.subscribe();

        if let Ok(()) = receiver.recv().await {
            println!("launching!!!!");
        }

        if let Err(error) = server.run().await {
            println!("error with running {:?}", error);
        };
    });

    Ok(server_handle)
}