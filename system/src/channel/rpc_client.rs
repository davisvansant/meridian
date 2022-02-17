use tokio::sync::mpsc;

pub type RpcClientReceiver = mpsc::Receiver<RpcClientRequest>;
pub type RpcClientSender = mpsc::Sender<RpcClientRequest>;

#[derive(Clone, Debug)]
pub enum RpcClientRequest {
    StartElection,
    SendHeartbeat,
    Shutdown,
}

pub async fn start_election(
    rpc_client: &RpcClientSender,
) -> Result<(), Box<dyn std::error::Error>> {
    rpc_client.send(RpcClientRequest::StartElection).await?;

    Ok(())
}

pub async fn send_heartbeat(
    rpc_client: &RpcClientSender,
) -> Result<(), Box<dyn std::error::Error>> {
    rpc_client.send(RpcClientRequest::SendHeartbeat).await?;

    Ok(())
}

pub async fn shutdown_rpc_client(
    rpc_client: &RpcClientSender,
) -> Result<(), Box<dyn std::error::Error>> {
    rpc_client.send(RpcClientRequest::Shutdown).await?;

    Ok(())
}
