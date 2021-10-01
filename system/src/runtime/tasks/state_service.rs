use crate::runtime::sync::state_receive_action::ChannelStateReceiveAction;
use crate::runtime::sync::state_send_grpc_action::ChannelStateSendGrpcAction;
use crate::runtime::sync::state_send_server_action::ChannelStateSendServerAction;
use crate::runtime::tasks::JoinHandle;
use crate::state::State;

pub async fn run_task(
    state_send_handle: ChannelStateReceiveAction,
    state_send_server_actions: ChannelStateSendServerAction,
    state_send_grpc_actions: ChannelStateSendGrpcAction,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    let mut state = State::init(
        state_send_handle,
        state_send_server_actions,
        state_send_grpc_actions,
    )
    .await?;

    let state_handle = tokio::spawn(async move {
        // sleep(Duration::from_secs(5)).await;

        if let Err(error) = state.run().await {
            println!("state error! {:?}", error);
        }
    });

    Ok(state_handle)
}
