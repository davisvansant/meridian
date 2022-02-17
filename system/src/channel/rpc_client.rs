// use std::net::SocketAddr;
// use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc;

// pub type RpcClientReceiver = mpsc::Receiver<(RpcClientRequest, oneshot::Sender<RpcClientResponse>)>;
// pub type RpcClientSender = mpsc::Sender<(RpcClientRequest, oneshot::Sender<RpcClientResponse>)>;
pub type RpcClientReceiver = mpsc::Receiver<RpcClientRequest>;
pub type RpcClientSender = mpsc::Sender<RpcClientRequest>;

#[derive(Clone, Debug)]
pub enum RpcClientRequest {
    StartElection,
    SendHeartbeat,
    Shutdown,
}

// #[derive(Clone, Debug)]
// pub enum RpcClientResponse {
//     // Nodes(Vec<SocketAddr>),
//     // Status(u8),
//     EndElection(()),
// }

// pub async fn start_election(
//     rpc_client: &RpcClientSender,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let (request, response) = oneshot::channel();

//     rpc_client
//         .send((RpcClientRequest::StartElection, request))
//         .await?;

//     match response.await {
//         Ok(RpcClientResponse::EndElection(())) => Ok(()),
//         Err(error) => Err(Box::new(error)),
//         // _ => panic!("unexpected response!"),
//     }
// }
pub async fn start_election(
    rpc_client: &RpcClientSender,
) -> Result<(), Box<dyn std::error::Error>> {
    // let (request, response) = oneshot::channel();

    // rpc_client
    //     .send((RpcClientRequest::StartElection, request))
    //     .await?;
    // let (request, response) = oneshot::channel();

    rpc_client.send(RpcClientRequest::StartElection).await?;

    // match response.await {
    //     Ok(RpcClientResponse::EndElection(())) => Ok(()),
    //     Err(error) => Err(Box::new(error)),
    //     // _ => panic!("unexpected response!"),
    // }
    Ok(())
}

pub async fn send_heartbeat(
    rpc_client: &RpcClientSender,
) -> Result<(), Box<dyn std::error::Error>> {
    // let (request, _response) = oneshot::channel();

    // rpc_client
    //     .send((RpcClientRequest::SendHeartbeat, request))
    //     .await?;
    rpc_client.send(RpcClientRequest::SendHeartbeat).await?;

    Ok(())
}

pub async fn shutdown_rpc_client(
    rpc_client: &RpcClientSender,
) -> Result<(), Box<dyn std::error::Error>> {
    // let (request, _response) = oneshot::channel();

    // rpc_client
    //     .send((RpcClientRequest::Shutdown, request))
    //     .await?;
    rpc_client.send(RpcClientRequest::Shutdown).await?;

    Ok(())
}
