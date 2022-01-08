// pub mod sync;
pub mod tasks;

use self::tasks::membership_service;
// use self::tasks::preflight;
use self::tasks::server_service;
use self::tasks::state_service;

use crate::membership::ClusterSize;
use crate::node::Node;

use crate::rpc::{Data, Interface, Server};

use std::net::SocketAddr;

use tokio::sync::{broadcast, mpsc, oneshot};

use crate::channel::CandidateTransition;
use crate::channel::ServerState;
use crate::channel::{ClientRequest, ClientResponse};
use crate::channel::{MembershipRequest, MembershipResponse};
use crate::channel::{StateRequest, StateResponse};

use crate::channel::Leader;

use crate::rpc::Client;

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

    let (rpc_client_sender, mut rpc_client_receiver) =
        mpsc::channel::<(ClientRequest, oneshot::Sender<ClientResponse>)>(64);

    let (membership_sender, mut membership_receiver) =
        mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);
    let preflight_membership_sender = membership_sender.to_owned();
    let server_membership_sender = membership_sender.to_owned();

    let rpc_communications_server_membership_sender = membership_sender.to_owned();
    let rpc_membership_server_membership_sender = membership_sender.to_owned();

    let (server_sender, server_receiver) = mpsc::channel::<ServerState>(64);

    let (state_sender, state_receiver) =
        mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

    let rpc_communications_server_state_sender = state_sender.clone();
    let rpc_membership_server_state_sender = state_sender.clone();
    let client_state_sender = state_sender.clone();

    let (tx, mut rx) = broadcast::channel::<ServerState>(64);
    let client_transition_sender = tx.clone();
    let rpc_communications_server_transition_sender = tx.clone();
    let rpc_membership_server_transition_sender = tx.clone();

    let (candidate_sender, mut candidate_receiver) = mpsc::channel::<CandidateTransition>(64);
    let server_candidate_sender = candidate_sender.clone();

    let (leader_sender, mut leader_receiver) = mpsc::channel::<Leader>(64);

    let membership_service_handle =
        membership_service::run_task(cluster_size, peers, node, membership_receiver).await?;

    let server_service_handle = server_service::run_task(
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

    let state_service_handle = state_service::run_task(state_receiver).await?;

    // let mut rpc_membership_server = Server::init(
    //     Interface::Membership,
    //     rpc_membership_server_membership_sender,
    //     rpc_membership_server_state_sender,
    //     rpc_membership_server_transition_sender,
    // )
    // .await?;

    // let rpc_membership_server_handle = tokio::spawn(async move {
    //     if let Err(error) = rpc_membership_server.run().await {
    //         println!("error with rpc server {:?}", error);
    //     }
    // });
    let node_socket_address = node.build_address(node.cluster_port).await;

    let mut rpc_communications_server = Server::init(
        Interface::Communications,
        rpc_communications_server_membership_sender,
        rpc_communications_server_state_sender,
        // rpc_communications_server_transition_sender,
        leader_sender,
        node_socket_address,
    )
    .await?;
    let rpc_communications_server_handle = tokio::spawn(async move {
        if let Err(error) = rpc_communications_server.run().await {
            println!("error with rpc communications server {:?}", error);
        }
    });

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

    tokio::try_join!(
        client_handle,
        state_service_handle,
        server_service_handle,
        // rpc_membership_server_handle,
        rpc_communications_server_handle,
    )?;

    Ok(())
}
