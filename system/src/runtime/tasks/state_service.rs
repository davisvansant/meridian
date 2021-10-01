use crate::runtime::sync::state_receive_task::ChannelStateReceiveTask;
use crate::runtime::sync::state_send_grpc_task::ChannelStateSendGrpcTask;
use crate::runtime::sync::state_send_server_task::ChannelStateSendServerTask;
use crate::runtime::tasks::JoinHandle;
use crate::state::State;

pub async fn run_task(
    state_send_handle: ChannelStateReceiveTask,
    state_send_server_task: ChannelStateSendServerTask,
    state_send_grpc_task: ChannelStateSendGrpcTask,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    let mut state = State::init(
        state_send_handle,
        state_send_server_task,
        state_send_grpc_task,
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
