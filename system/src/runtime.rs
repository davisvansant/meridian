use std::net::SocketAddr;

use tokio::signal::ctrl_c;

use crate::channel;
use crate::membership::{ClusterSize, Membership};
use crate::node::Node;
use crate::rpc::{Client, Server};
use crate::server::Server as SystemServer;
use crate::state::State;
use crate::{error, info};

pub async fn launch(
    cluster_size: &str,
    peers: Vec<SocketAddr>,
    node: Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let cluster_size = ClusterSize::from_str(cluster_size).await;

    info!("cluster size -> {:?}", &cluster_size);
    info!("node id -> {:?}", &node.id);

    // let another_shutdown_signal = channel::shutdown::build().await;
    let another_shutdown_signal = crate::channel::transition::Shutdown::build().await;
    let shutdown_membership_tasks = another_shutdown_signal.to_owned();
    let shutdown_rpc_server_task = another_shutdown_signal.subscribe();
    let shutdown_system_server_task = another_shutdown_signal.to_owned();
    let mut system_shutdown = another_shutdown_signal.subscribe();

    // -------------------------------------------------------------------------------------------
    // |         init internal rpc client channel
    // -------------------------------------------------------------------------------------------

    // let (rpc_client_sender, rpc_client_receiver) = channel::rpc_client::build().await;
    // let shutdown_client = rpc_client_sender.clone();

    // -------------------------------------------------------------------------------------------
    // |        init membership channel
    // -------------------------------------------------------------------------------------------

    let (membership_sender, membership_receiver) = channel::membership::build().await;

    let server_membership_sender = membership_sender.to_owned();
    // let shutdown_membership = membership_sender.to_owned();

    // -------------------------------------------------------------------------------------------
    // |        init state channel
    // -------------------------------------------------------------------------------------------

    let (state_sender, state_receiver) = channel::state::build().await;

    let rpc_communications_server_state_sender = state_sender.clone();
    // let client_state_sender = state_sender.clone();
    // let shutdown_state = state_sender.to_owned();

    // -------------------------------------------------------------------------------------------
    // |        init server leader heartbeat channel
    // -------------------------------------------------------------------------------------------

    let leader_heartbeat_sender = channel::server::Leader::build().await;
    let system_leader_sender = leader_heartbeat_sender.to_owned();
    let rpc_server_heartbeat_sender = leader_heartbeat_sender.to_owned();

    drop(leader_heartbeat_sender);

    // -------------------------------------------------------------------------------------------
    // |        init membership
    // -------------------------------------------------------------------------------------------

    let mut membership = Membership::init(
        cluster_size,
        // node,
        membership_receiver,
        shutdown_membership_tasks,
    )
    .await?;

    let membership_handle = tokio::spawn(async move {
        if let Err(error) = membership.run(node, peers).await {
            error!("membership -> {:?}", error);
        }
    });

    // -------------------------------------------------------------------------------------------
    // |        init system server
    // -------------------------------------------------------------------------------------------

    let (server_transition_state_sender, server_transition_state_receiver) =
        crate::channel::transition::ServerState::build().await;

    let mut system_server = SystemServer::init(
        // rpc_client_sender,
        server_membership_sender,
        state_sender,
        system_leader_sender,
        shutdown_system_server_task,
        server_transition_state_receiver,
    )
    .await?;

    let system_server_handle = tokio::spawn(async move {
        if let Err(error) = system_server.run(&server_transition_state_sender).await {
            error!("server -> {:?}", error);
        }
    });

    // -------------------------------------------------------------------------------------------
    // |        init state
    // -------------------------------------------------------------------------------------------

    let mut state = State::init(state_receiver).await?;

    let state_handle = tokio::spawn(async move {
        if let Err(error) = state.run().await {
            error!("state -> {:?}", error);
        }
    });

    // -------------------------------------------------------------------------------------------
    // |        init internal rpc server channel
    // -------------------------------------------------------------------------------------------

    let node_socket_address = node.build_address(node.cluster_port).await;

    let mut rpc_communications_server = Server::init(
        rpc_communications_server_state_sender,
        rpc_server_heartbeat_sender,
        node_socket_address,
        shutdown_rpc_server_task,
    )
    .await?;

    let rpc_communications_server_handle = tokio::spawn(async move {
        if let Err(error) = rpc_communications_server.run().await {
            error!("rpc communications server -> {:?}", error);
        }
    });

    // -------------------------------------------------------------------------------------------
    // |        init shutdown signal
    // -------------------------------------------------------------------------------------------

    let shutdown_signal = tokio::spawn(async move {
        loop {
            tokio::select! {
                // biased;
                ctrl_c = ctrl_c() => {
                    info!("received shutdown signal {:?}", ctrl_c);
                    info!("preparing to shut down...");

                // if let Err(error) = crate::channel::shutdown::shutdown(&another_shutdown_signal).await {
                //     error!("error sending shutdown signal! -> {:?}", error);
                // }
                if let Err(error) = crate::channel::transition::Shutdown::send(&another_shutdown_signal).await {
                    error!("error sending shutdown signal! -> {:?}", error);
                }

                // if let Ok(()) = crate::channel::state::shutdown(&shutdown_state).await {
                //     info!("system state shutdown...");
                // }

                // if let Ok(()) = crate::channel::membership::shutdown(&shutdown_membership).await {
                //     info!("initiating membership shutdown...");
                // }

                    break
                }
                _ = system_shutdown.recv() => {
                    info!("shutting down...");

                    break
                }
            }
        }
    });

    // -------------------------------------------------------------------------------------------
    // |        launch!!!
    // -------------------------------------------------------------------------------------------

    tokio::try_join!(
        state_handle,
        membership_handle,
        system_server_handle,
        rpc_communications_server_handle,
        shutdown_signal,
    )?;

    Ok(())
}
