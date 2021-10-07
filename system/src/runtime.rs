pub mod sync;
pub mod tasks;

use self::sync::launch;
use self::sync::membership_receive_task;
use self::sync::membership_send_grpc_task;
use self::sync::membership_send_preflight_task;
use self::sync::membership_send_server_task;
use self::sync::state_receive_task;
use self::sync::state_send_grpc_task;
use self::sync::state_send_server_task;

use self::tasks::client_grpc;
use self::tasks::cluster_grpc;
use self::tasks::membership_grpc;
use self::tasks::membership_service;
use self::tasks::preflight;
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

    let (
        runtime_sender,
        runtime_client_grpc_receiver,
        runtime_cluster_grpc_receiver,
        launch_server_service,
    ) = launch::build_channel().await;

    let (
        membership_receive_task,
        membership_grpc_send_membership_task,
        preflight_send_membership_task,
        server_send_membership_task,
    ) = membership_receive_task::build_channel().await;

    let (membership_send_membership_grpc_task, membership_grpc_receive_membership_task) =
        membership_send_grpc_task::build_channel().await;

    let (membership_send_preflight_task, preflight_receive_membership_task) =
        membership_send_preflight_task::build_channel().await;

    let (membership_send_server_task, server_receive_membership_task) =
        membership_send_server_task::build_channel().await;

    let (state_send_handle, grpc_send_task, server_send_task) =
        state_receive_task::build_channel().await;

    let (state_receive_grpc_task, state_send_grpc_task) =
        state_send_grpc_task::build_channel().await;

    let (state_receive_server_task, state_send_server_task) =
        state_send_server_task::build_channel().await;

    let client_grpc_handle = client_grpc::run_task(
        node.build_address(node.client_port).await,
        runtime_client_grpc_receiver,
    )
    .await?;

    let cluster_grpc_handle = cluster_grpc::run_task(
        state_receive_grpc_task,
        grpc_send_task,
        // "0.0.0.0:10000".parse()?,
        node.build_address(node.cluster_port).await,
        runtime_cluster_grpc_receiver,
    )
    .await?;

    let membership_grpc_handle = membership_grpc::run_task(
        membership_grpc_receive_membership_task,
        membership_grpc_send_membership_task,
        // "0.0.0.0:15000".parse()?,
        node.build_address(node.membership_port).await,
    )
    .await?;

    let membership_service_handle = membership_service::run_task(
        cluster_size,
        node,
        // peers,
        membership_receive_task,
        membership_send_membership_grpc_task,
        membership_send_preflight_task,
        membership_send_server_task,
        // runtime_sender,
    )
    .await?;

    let preflight_handle = preflight::run_task(
        preflight_send_membership_task,
        preflight_receive_membership_task,
        runtime_sender,
        peers,
    )
    .await?;

    let server_service_handle = server_service::run_task(
        state_receive_server_task,
        server_send_task,
        server_send_membership_task,
        server_receive_membership_task,
        launch_server_service,
    )
    .await?;

    let state_service_handle = state_service::run_task(
        state_send_handle,
        state_send_server_task,
        state_send_grpc_task,
    )
    .await?;

    tokio::try_join!(
        preflight_handle,
        state_service_handle,
        server_service_handle,
        membership_service_handle,
        membership_grpc_handle,
        cluster_grpc_handle,
        client_grpc_handle,
    )?;

    Ok(())
}
