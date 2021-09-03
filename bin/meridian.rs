use tokio::time::{sleep, Duration};

use system::external_client_grpc_server::{
    CommunicationsServer as ClientServer, ExternalClientGrpcServer,
};
use system::internal_cluster_grpc_server::{
    CommunicationsServer as ClusterServer, InternalClusterGrpcServer,
};
use system::server::{Server, ServerState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cluster_address = "0.0.0.0:10000".parse().unwrap();
    let internal_cluster_grpc_server = InternalClusterGrpcServer::init().await?;
    let cluster_service = ClusterServer::new(internal_cluster_grpc_server);
    let cluster_grpc_server = tonic::transport::Server::builder()
        .add_service(cluster_service)
        .serve(cluster_address);

    let cluster_handle = tokio::spawn(async move {
        println!("starting up server");
        if let Err(error) = cluster_grpc_server.await {
            println!(
                "something went wrong with the internal grpc interface - {:?}",
                error,
            );
        }
    });

    let client_address = "0.0.0.0:20000".parse().unwrap();
    let external_client_grpc_server = ExternalClientGrpcServer::init().await?;
    let client_service = ClientServer::new(external_client_grpc_server);
    let client_grpc_server = tonic::transport::Server::builder()
        .add_service(client_service)
        .serve(client_address);

    let client_handle = tokio::spawn(async move {
        println!("starting up client server");
        if let Err(error) = client_grpc_server.await {
            println!(
                "something went wrong with the client grpc interface - {:?}",
                error,
            );
        }
    });

    let mut state_machine = Server::init().await?;

    let machine_handle = tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;

        while state_machine.server_state == ServerState::Follower {
            println!("doing follwer stuff!");

            if let Err(error) = state_machine.follower().await {
                println!("Something went wrong with the follower - {:?}", error);
            }

            println!("transitioning to candidate...");
        }

        while state_machine.server_state == ServerState::Candidate {
            println!("doing candidate stuff!");

            if let Err(error) = state_machine.candidate().await {
                println!("something went wrong with the candidate {:?}", error);
            }

            println!("transitioning to leader...");
        }

        while state_machine.server_state == ServerState::Leader {
            println!("Leader!");

            if let Err(error) = state_machine.leader().await {
                println!("the leader had an error - {:?}", error);
            }
        }
    });

    tokio::try_join!(cluster_handle, client_handle, machine_handle)?;

    Ok(())
}
