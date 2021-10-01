use crate::runtime::tasks::JoinHandle;
use crate::runtime::tasks::Server;
use crate::runtime::tasks::SocketAddr;

use crate::grpc::cluster_server::{
    CommunicationsServer as ClusterServer, InternalClusterGrpcServer,
};

use crate::runtime::sync::launch::ChannelLaunch;
use crate::runtime::sync::state_receive_task::ChannelStateReceiveTask;
use crate::runtime::sync::state_send_grpc_task::ChannelStateSendGrpcTask;

pub async fn run_task(
    state_receive_grpc_task: ChannelStateSendGrpcTask,
    grpc_send_task: ChannelStateReceiveTask,
    socket_address: SocketAddr,
    channel_launch: ChannelLaunch,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    let server = InternalClusterGrpcServer::init(state_receive_grpc_task, grpc_send_task).await?;
    let grpc_service = ClusterServer::new(server);
    let transport_router = Server::builder()
        .add_service(grpc_service)
        .serve(socket_address);

    let handle = tokio::spawn(async move {
        println!("waiting on membership...");

        let mut receiver = channel_launch.subscribe();

        if let Ok(()) = receiver.recv().await {
            println!("launching!!!!");
        }

        println!("starting up server");
        if let Err(error) = transport_router.await {
            println!(
                "something went wrong with the internal grpc interface - {:?}",
                error,
            );
        }
    });

    Ok(handle)
}
