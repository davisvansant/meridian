use std::net::SocketAddr;

use tokio::signal::ctrl_c;

use crate::channel::membership::MembershipChannel;
use crate::channel::server::Leader;
use crate::channel::server_state::shutdown::Shutdown;
use crate::channel::state::StateChannel;
use crate::membership::{ClusterSize, Membership};
use crate::node::Node;
use crate::rpc;
use crate::server;
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

    // -------------------------------------------------------------------------------------------
    // |        init shutdown channel
    // -------------------------------------------------------------------------------------------

    let shutdown = Shutdown::init();
    let shutdown_membership = shutdown.to_owned();
    let shutdown_rpc_server = shutdown.to_owned();
    let shutdown_system_server = shutdown.to_owned();
    let mut system_shutdown = shutdown.subscribe();

    // -------------------------------------------------------------------------------------------
    // |        init membership channel
    // -------------------------------------------------------------------------------------------

    let (membership_channel, membership_receiver) = MembershipChannel::init().await;

    // -------------------------------------------------------------------------------------------
    // |        init state channel
    // -------------------------------------------------------------------------------------------

    let (state_channel, state_receiver) = StateChannel::init().await;
    let rpc_communications_server_state_sender = state_channel.clone();

    // -------------------------------------------------------------------------------------------
    // |        init server leader heartbeat channel
    // -------------------------------------------------------------------------------------------

    let leader_heartbeat_sender = Leader::build().await;
    let system_leader_sender = leader_heartbeat_sender.to_owned();
    let rpc_server_heartbeat_sender = leader_heartbeat_sender.to_owned();

    drop(leader_heartbeat_sender);

    // -------------------------------------------------------------------------------------------
    // |        init membership
    // -------------------------------------------------------------------------------------------

    let mut membership =
        Membership::init(cluster_size, membership_receiver, shutdown_membership).await?;

    let membership_handle = tokio::spawn(async move {
        if let Err(error) = membership.run(node, peers).await {
            error!("membership -> {:?}", error);
        }
    });

    // -------------------------------------------------------------------------------------------
    // |        init system server
    // -------------------------------------------------------------------------------------------

    let mut system_server = server::Server::init(
        membership_channel,
        state_channel,
        system_leader_sender,
        shutdown_system_server,
    )
    .await?;

    let system_server_handle = tokio::spawn(async move {
        if let Err(error) = system_server.run().await {
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

    let mut rpc_communications_server = rpc::Server::init(
        rpc_communications_server_state_sender,
        rpc_server_heartbeat_sender,
        node_socket_address,
        shutdown_rpc_server,
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
                biased;
                ctrl_c = ctrl_c() => {
                    info!("received shutdown signal {:?}", ctrl_c);
                    info!("preparing to shut down...");

                if let Err(error) = shutdown.run() {
                    error!("error sending shutdown signal! -> {:?}", error);
                }
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
