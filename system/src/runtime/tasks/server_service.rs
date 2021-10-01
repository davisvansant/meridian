use crate::runtime::sync::membership_receive_task::ChannelMembershipReceiveAction;
use crate::runtime::sync::membership_send_server_task::ChannelMembershipSendServerAction;
use crate::runtime::sync::state_receive_action::ChannelStateReceiveAction;
use crate::runtime::sync::state_send_server_action::ChannelStateSendServerAction;
use crate::runtime::tasks::JoinHandle;
use crate::server::Server;

pub async fn run_task(
    state_receive_server_actions: ChannelStateSendServerAction,
    server_send_actions: ChannelStateReceiveAction,
    server_send_membership_action: ChannelMembershipReceiveAction,
    server_receive_membership_action: ChannelMembershipSendServerAction,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    let mut server = Server::init(
        state_receive_server_actions,
        server_send_actions,
        server_send_membership_action,
        server_receive_membership_action,
    )
    .await?;

    let server_handle = tokio::spawn(async move {
        // sleep(Duration::from_secs(10)).await;

        if let Err(error) = server.run().await {
            println!("error with running {:?}", error);
        };
    });

    Ok(server_handle)
}
