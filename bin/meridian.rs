use clap::{App, Arg, SubCommand};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use tokio::sync::broadcast::channel;
use tokio::time::{sleep, Duration};

use system::external_client_grpc_server::{
    CommunicationsServer as ClientServer, ExternalClientGrpcServer,
};
use system::internal_cluster_grpc_server::{
    CommunicationsServer as ClusterServer, InternalClusterGrpcServer,
};

use system::server::Server;
use system::state::State;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let leaders = Arg::with_name("leaders")
        .help("how many leaders")
        .long("leaders")
        .takes_value(true)
        .possible_value("1")
        .possible_value("3")
        .possible_value("5")
        .index(1);
    let cluster_port = Arg::with_name("cluster_port")
        .help("set the cluster port (membership gRPC)")
        .long("cluster_port")
        .takes_value(true)
        .default_value("10000")
        .index(2);
    let client_port = Arg::with_name("client_port")
        .help("set the client port (client communications gRPC)")
        .long("client_port")
        .takes_value(true)
        .default_value("20000")
        .index(3);
    let run = SubCommand::with_name("run")
        .about("run meridian")
        .arg(leaders)
        .arg(cluster_port)
        .arg(client_port);
    let meridian = App::new("meridian")
        .author("some_author_goes_here")
        .version(env!("CARGO_PKG_VERSION"))
        .about("meridian")
        .subcommand(run)
        .get_matches();

    match meridian.subcommand() {
        ("run", Some(run)) => {
            if let Some(leaders) = run.value_of("leaders") {
                match leaders {
                    "1" => println!("one"),
                    "3" => println!("three"),
                    "5" => println!("five"),
                    _ => println!("not supported!"),
                }
            }

            let cluster_port_value = run.value_of("cluster_port").unwrap();
            let cluster_port = u16::from_str(cluster_port_value)?;
            let cluster_ip_address = IpAddr::from_str("0.0.0.0")?;
            let cluster_address = SocketAddr::new(cluster_ip_address, cluster_port);

            println!("launching cluster on {:?}", &cluster_address);

            let client_port_value = run.value_of("client_port").unwrap();
            let client_port = u16::from_str(client_port_value)?;
            let client_ip_address = IpAddr::from_str("0.0.0.0")?;
            let client_address = SocketAddr::new(client_ip_address, client_port);

            println!("launching client on {:?}", &client_address);

            let (state_send_handle, _) = channel(64);
            let grpc_send_actions = state_send_handle.clone();
            let server_send_actions = state_send_handle.clone();

            let (state_receive_server_actions, _) = channel(64);
            let (state_receive_grpc_actions, _) = channel(64);
            let state_send_server_actions = state_receive_server_actions.clone();
            let state_send_grpc_actions = state_receive_grpc_actions.clone();

            let internal_cluster_grpc_server =
                InternalClusterGrpcServer::init(state_receive_grpc_actions, grpc_send_actions)
                    .await?;

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

            let mut server =
                Server::init(state_receive_server_actions, server_send_actions).await?;

            let server_handle = tokio::spawn(async move {
                sleep(Duration::from_secs(10)).await;

                if let Err(error) = server.run().await {
                    println!("error with running {:?}", error);
                };
            });

            let mut state = State::init(
                state_send_handle,
                state_send_server_actions,
                state_send_grpc_actions,
            )
            .await?;

            let state_handle = tokio::spawn(async move {
                sleep(Duration::from_secs(5)).await;

                if let Err(error) = state.run().await {
                    println!("state error! {:?}", error);
                }
            });

            tokio::try_join!(cluster_handle, client_handle, server_handle, state_handle)?;
        }
        _ => println!("{:?}", meridian.usage()),
    }

    Ok(())
}
