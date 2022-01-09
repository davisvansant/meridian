use std::net::SocketAddr;

use tokio::sync::{broadcast, mpsc, oneshot};

use crate::channel::CandidateTransition;
use crate::channel::Leader;
use crate::channel::ServerState;
use crate::channel::{ClientRequest, ClientResponse};
use crate::channel::{MembershipRequest, MembershipResponse};
use crate::channel::{StateRequest, StateResponse};

use crate::membership::{ClusterSize, Membership};

use crate::node::Node;

use crate::rpc::{Client, Data, Interface, Server};

use crate::server::Server as SystemServer;

use crate::state::State;

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

    let (rpc_client_sender, mut rpc_client_receiver) =
        mpsc::channel::<(ClientRequest, oneshot::Sender<ClientResponse>)>(64);

    // -------------------------------------------------------------------------------------------
    // |        init membership channel
    // -------------------------------------------------------------------------------------------

    let (membership_sender, mut membership_receiver) =
        mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);

    let preflight_membership_sender = membership_sender.to_owned();
    let server_membership_sender = membership_sender.to_owned();
    let rpc_communications_server_membership_sender = membership_sender.to_owned();
    let rpc_membership_server_membership_sender = membership_sender.to_owned();

    // -------------------------------------------------------------------------------------------
    // |        init system server channel
    // -------------------------------------------------------------------------------------------

    let (server_sender, server_receiver) = mpsc::channel::<ServerState>(64);

    // -------------------------------------------------------------------------------------------
    // |        init state channel
    // -------------------------------------------------------------------------------------------

    let (state_sender, state_receiver) =
        mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

    let rpc_communications_server_state_sender = state_sender.clone();
    let rpc_membership_server_state_sender = state_sender.clone();
    let client_state_sender = state_sender.clone();

    // -------------------------------------------------------------------------------------------
    // |        init state transition channel
    // -------------------------------------------------------------------------------------------

    let (tx, mut rx) = broadcast::channel::<ServerState>(64);

    let client_transition_sender = tx.clone();
    let rpc_communications_server_transition_sender = tx.clone();
    let rpc_membership_server_transition_sender = tx.clone();

    // -------------------------------------------------------------------------------------------
    // |        init server candidate transition channel
    // -------------------------------------------------------------------------------------------

    let (candidate_sender, mut candidate_receiver) = mpsc::channel::<CandidateTransition>(64);
    let server_candidate_sender = candidate_sender.clone();

    // -------------------------------------------------------------------------------------------
    // |        init server leader heartbeat channel
    // -------------------------------------------------------------------------------------------

    let (leader_sender, mut leader_receiver) = mpsc::channel::<Leader>(64);

    // -------------------------------------------------------------------------------------------
    // |        init membership
    // -------------------------------------------------------------------------------------------

    let mut membership = Membership::init(cluster_size, peers, node, membership_receiver).await?;

    let membership_handle = tokio::spawn(async move {
        if let Err(error) = membership.run().await {
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
        tx,
        rx,
        server_candidate_sender,
        candidate_receiver,
        leader_receiver,
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
        Interface::Communications,
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
        Interface::Communications,
        rpc_client_receiver,
        membership_sender,
        client_state_sender,
        client_transition_sender,
        candidate_sender,
    )
    .await?;

    let client_handle = tokio::spawn(async move {
        if let Err(error) = client.run().await {
            println!("error running client... {:?}", error);
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
    )?;

    Ok(())
}
