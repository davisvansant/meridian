use crate::grpc::client_server::ExternalClientGrpcServer;
use crate::runtime::sync::launch::ChannelLaunch;
use crate::runtime::tasks::JoinHandle;
use crate::runtime::tasks::Server;
use crate::runtime::tasks::SocketAddr;

pub use crate::meridian_client_v010::communications_server::{
    Communications, CommunicationsServer,
};

pub async fn run_task(
    socket_address: SocketAddr,
    channel_launch: ChannelLaunch,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    let client_grpc_server = ExternalClientGrpcServer::init().await?;
    let service = CommunicationsServer::new(client_grpc_server);
    let client_grpc_server = Server::builder().add_service(service).serve(socket_address);

    let handle = tokio::spawn(async move {
        println!("waiting on membership...");

        let mut receiver = channel_launch.subscribe();

        if let Ok(()) = receiver.recv().await {
            println!("launching!!!!");
        }

        println!("starting up client server");
        if let Err(error) = client_grpc_server.await {
            println!(
                "something went wrong with the client grpc interface - {:?}",
                error,
            );
        }
    });

    Ok(handle)
}
