use std::net::SocketAddr;

use tokio::signal::ctrl_c;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::channel::CandidateTransition;
use crate::channel::Leader;
use crate::channel::{ClientRequest, ClientResponse};
// use crate::channel::{MembershipMaintenanceRequest, MembershipMaintenanceResponse};
use crate::channel::{MembershipRequest, MembershipResponse};
use crate::channel::{StateRequest, StateResponse};
use crate::membership::{ClusterSize, Membership};
use crate::node::Node;
use crate::rpc::{Client, Server};
use crate::server::Server as SystemServer;
use crate::state::State;
// use crate::communication::membership_maintenance::MembershipMaintenance;
// use crate::communication::membership_dynamic_join::MembershipDynamicJoin;

pub async fn launch(
    cluster_size: &str,
    peers: Vec<SocketAddr>,
    node: Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let cluster_size = match cluster_size {
        "1" => ClusterSize::One,
        "3" => ClusterSize::Three,
        "5" => ClusterSize::Five,
        _ => panic!("Expected a cluster size of 1, 3, or 5"),
    };

    // -------------------------------------------------------------------------------------------
    // |         init internal rpc client channel
    // -------------------------------------------------------------------------------------------

    let (rpc_client_sender, rpc_client_receiver) =
        mpsc::channel::<(ClientRequest, oneshot::Sender<ClientResponse>)>(64);
    let shutdown_client = rpc_client_sender.clone();

    // -------------------------------------------------------------------------------------------
    // |        init membership channel
    // -------------------------------------------------------------------------------------------

    let (membership_sender, membership_receiver) =
        mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);

    // let preflight_membership_sender = membership_sender.to_owned();
    let server_membership_sender = membership_sender.to_owned();
    let rpc_communications_server_membership_sender = membership_sender.to_owned();
    // let rpc_membership_server_membership_sender = membership_sender.to_owned();
    let shutdown_membership = membership_sender.to_owned();

    // -------------------------------------------------------------------------------------------
    // |        init state channel
    // -------------------------------------------------------------------------------------------

    let (state_sender, state_receiver) =
        mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

    let rpc_communications_server_state_sender = state_sender.clone();
    // let rpc_membership_server_state_sender = state_sender.clone();
    let client_state_sender = state_sender.clone();
    let shutdown_state = state_sender.to_owned();

    // -------------------------------------------------------------------------------------------
    // |        init server candidate transition channel
    // -------------------------------------------------------------------------------------------

    let (candidate_sender, _candidate_receiver) = broadcast::channel::<CandidateTransition>(64);
    let server_candidate_sender = candidate_sender.clone();
    let client_transition_sender = candidate_sender.clone();

    drop(candidate_sender);

    // -------------------------------------------------------------------------------------------
    // |        init server leader heartbeat channel
    // -------------------------------------------------------------------------------------------

    let (leader_sender, _leader_receiver) = broadcast::channel::<Leader>(64);
    let system_leader_sender = leader_sender.to_owned();

    // -------------------------------------------------------------------------------------------
    // |        init membership
    // -------------------------------------------------------------------------------------------

    // let mut membership = Membership::init(cluster_size, peers, node, membership_receiver).await?;
    let mut membership = Membership::init(cluster_size, node, membership_receiver).await?;

    let membership_handle = tokio::spawn(async move {
        if let Err(error) = membership.run(peers).await {
            println!("error running membership -> {:?}", error);
        }
    });

    // -------------------------------------------------------------------------------------------
    // |        init system server
    // -------------------------------------------------------------------------------------------

    let mut system_server = SystemServer::init(
        rpc_client_sender,
        server_membership_sender,
        state_sender,
        server_candidate_sender,
        system_leader_sender,
    )
    .await?;

    let system_server_handle = tokio::spawn(async move {
        if let Err(error) = system_server.run().await {
            println!("error running server -> {:?}", error);
        }
    });

    // -------------------------------------------------------------------------------------------
    // |        init state
    // -------------------------------------------------------------------------------------------

    let mut state = State::init(state_receiver).await?;

    let state_handle = tokio::spawn(async move {
        if let Err(error) = state.run().await {
            println!("error with state -> {:?}", error);
        }
    });

    // -------------------------------------------------------------------------------------------
    // |        init internal rpc server channel
    // -------------------------------------------------------------------------------------------

    let node_socket_address = node.build_address(node.cluster_port).await;

    let mut rpc_communications_server = Server::init(
        rpc_communications_server_membership_sender,
        rpc_communications_server_state_sender,
        leader_sender,
        node_socket_address,
    )
    .await?;

    let rpc_communications_server_handle = tokio::spawn(async move {
        if let Err(error) = rpc_communications_server.run().await {
            println!("error with rpc communications server {:?}", error);
        }
    });

    // -------------------------------------------------------------------------------------------
    // |        init internal rpc client
    // -------------------------------------------------------------------------------------------

    let mut client = Client::init(
        rpc_client_receiver,
        membership_sender,
        client_state_sender,
        client_transition_sender,
    )
    .await?;

    let client_handle = tokio::spawn(async move {
        if let Err(error) = client.run().await {
            println!("error running client... {:?}", error);
        }
    });

    // -------------------------------------------------------------------------------------------
    // |        init membership dynamic join
    // -------------------------------------------------------------------------------------------
    // let (send_membership_dynamic_join_shutdown, receive_membership_dynamic_join_shutdown) =
    //     mpsc::channel::<bool>(1);
    // let membership_dynamic_join_socket_address = node.build_address(node.membership_port).await;

    // let mut membership_dynamic_join = MembershipDynamicJoin::init(
    //     membership_dynamic_join_socket_address,
    //     receive_membership_dynamic_join_shutdown,
    // )
    // .await?;

    // let membership_dynamic_join_handle = tokio::spawn(async move {
    //     if let Err(error) = membership_dynamic_join.run().await {
    //         println!("error running membership dynamic join -> {:?}", error);
    //     }
    // });

    // -------------------------------------------------------------------------------------------
    // |        init membership maintenance
    // -------------------------------------------------------------------------------------------
    // let (membership_maintenance_sender, membership_maintenance_receiver) = mpsc::channel::<(
    //     MembershipMaintenanceRequest,
    //     oneshot::Sender<MembershipMaintenanceResponse>,
    // )>(64);
    // let shutdown_membership_maintenance = membership_maintenance_sender.clone();

    // let membership_socket_address = node.build_address(node.membership_port).await;

    // let mut membership_maintenance = MembershipMaintenance::init(
    //     membership_socket_address,
    //     membership_maintenance_receiver,
    //     membership_maintenance_sender,
    // )
    // .await?;

    // let membership_maintenance_handle = tokio::spawn(async move {
    //     if let Err(error) = membership_maintenance.run().await {
    //         println!("erroe with membership maintenance -> {:?}", error);
    //     }
    // });

    // -------------------------------------------------------------------------------------------
    // |        init shutdown signal
    // -------------------------------------------------------------------------------------------

    let shutdown_signal = tokio::spawn(async move {
        if let Ok(()) = ctrl_c().await {
            println!("received shutdown signal...");
            println!("shutting down...");

            if let Ok(()) = crate::channel::shutdown_state(&shutdown_state).await {
                println!("system state shutdown...");
            }

            if let Ok(()) = crate::channel::shutdown_client(&shutdown_client).await {
                println!("rpc client shutdown...");
            }

            if let Ok(()) = crate::channel::shutdown_membership(&shutdown_membership).await {
                println!("initiating membership shutdown...");
            }

            // println!("shutting down rpc server interface ....");

            // drop(send_rpc_server_shutdown);

            // println!("shutting down membership dynamic join interface ....");

            // drop(send_membership_dynamic_join_shutdown);

            // if let Ok(()) =
            //     crate::channel::shutdown_membership_maintenance(&shutdown_membership_maintenance)
            //         .await
            // {
            //     println!("initiating membership maintenance shutdown...");
            // }
        }
    });

    // -------------------------------------------------------------------------------------------
    // |        launch!!!
    // -------------------------------------------------------------------------------------------

    tokio::try_join!(
        state_handle,
        membership_handle,
        system_server_handle,
        client_handle,
        rpc_communications_server_handle,
        // membership_maintenance_handle,
        // membership_dynamic_join_handle,
        shutdown_signal,
    )?;

    Ok(())
}
