pub mod sync;
pub mod tasks;

use self::sync::launch;
use self::sync::membership_receive_task;
use self::sync::membership_send_grpc_task;
use self::sync::membership_send_server_task;
use self::sync::state_receive_action;
use self::sync::state_send_grpc_action;
use self::sync::state_send_server_action;

use self::tasks::client_grpc;
use self::tasks::cluster_grpc;
use self::tasks::membership_grpc;
use self::tasks::membership_service;
use self::tasks::server_service;
use self::tasks::state_service;

use crate::membership::ClusterSize;
use crate::node::Node;

pub async fn launch(
    cluster_size: &str,
    peers: Vec<String>,
    node: Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let cluster_size = match cluster_size {
        "1" => ClusterSize::One,
        "3" => ClusterSize::Three,
        "5" => ClusterSize::Five,
        _ => panic!("Expected a cluster size of 1, 3, or 5"),
    };

    let (runtime_sender, runtime_client_grpc_receiver, runtime_cluster_grpc_receiver) =
        launch::build_channel().await;

    let (
        membership_receive_action,
        membership_grpc_send_membership_action,
        server_send_membership_action,
    ) = membership_receive_task::build_channel().await;

    let (membership_send_membership_grpc_action, membership_grpc_receive_membership_action) =
        membership_send_grpc_task::build_channel().await;

    let (membership_send_server_action, server_receive_membership_action) =
        membership_send_server_task::build_channel().await;

    let (state_send_handle, grpc_send_action, server_send_action) =
        state_receive_action::build_channel().await;

    let (state_receive_grpc_action, state_send_grpc_action) =
        state_send_grpc_action::build_channel().await;

    let (state_receive_server_action, state_send_server_action) =
        state_send_server_action::build_channel().await;

    let client_grpc_handle = client_grpc::run_task(
        node.build_address(node.client_port).await,
        runtime_client_grpc_receiver,
    )
    .await?;

    let cluster_grpc_handle = cluster_grpc::run_task(
        state_receive_grpc_action,
        grpc_send_action,
        // "0.0.0.0:10000".parse()?,
        node.build_address(node.cluster_port).await,
        runtime_cluster_grpc_receiver,
    )
    .await?;

    let membership_grpc_handle = membership_grpc::run_task(
        membership_grpc_receive_membership_action,
        membership_grpc_send_membership_action,
        // "0.0.0.0:15000".parse()?,
        node.build_address(node.membership_port).await,
    )
    .await?;

    let membership_service_handle = membership_service::run_task(
        cluster_size,
        node,
        peers,
        membership_receive_action,
        membership_send_membership_grpc_action,
        membership_send_server_action,
        runtime_sender,
    )
    .await?;

    let server_service_handle = server_service::run_task(
        state_receive_server_action,
        server_send_action,
        server_send_membership_action,
        server_receive_membership_action,
    )
    .await?;

    let state_service_handle = state_service::run_task(
        state_send_handle,
        state_send_server_action,
        state_send_grpc_action,
    )
    .await?;

    tokio::try_join!(
        state_service_handle,
        server_service_handle,
        membership_service_handle,
        membership_grpc_handle,
        cluster_grpc_handle,
        client_grpc_handle,
    )?;

    Ok(())
}
