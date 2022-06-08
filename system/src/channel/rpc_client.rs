use tokio::sync::mpsc;

pub type RpcClientReceiver = mpsc::Receiver<RpcClientRequest>;
pub type RpcClientSender = mpsc::Sender<RpcClientRequest>;

#[derive(Clone, Debug)]
pub enum RpcClientRequest {
    StartElection,
    SendHeartbeat,
    Shutdown,
}

pub async fn build() -> (RpcClientSender, RpcClientReceiver) {
    let (rpc_client_sender, rpc_client_receiver) = mpsc::channel::<RpcClientRequest>(64);

    (rpc_client_sender, rpc_client_receiver)
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

pub async fn shutdown(rpc_client: &RpcClientSender) -> Result<(), Box<dyn std::error::Error>> {
    rpc_client.send(RpcClientRequest::Shutdown).await?;

    Ok(())
}
