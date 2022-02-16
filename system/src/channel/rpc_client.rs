use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};

pub type ClientReceiver = mpsc::Receiver<(ClientRequest, oneshot::Sender<ClientResponse>)>;
pub type ClientSender = mpsc::Sender<(ClientRequest, oneshot::Sender<ClientResponse>)>;

#[derive(Clone, Debug)]
pub enum ClientRequest {
    StartElection,
    SendHeartbeat,
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum ClientResponse {
    Nodes(Vec<SocketAddr>),
    Status(u8),
    EndElection(()),
}

pub async fn start_election(client: &ClientSender) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    client.send((ClientRequest::StartElection, request)).await?;

    match response.await {
        Ok(ClientResponse::EndElection(())) => Ok(()),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn send_heartbeat(client: &ClientSender) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    client.send((ClientRequest::SendHeartbeat, request)).await?;

    Ok(())
}

pub async fn shutdown_client(client: &ClientSender) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    client.send((ClientRequest::Shutdown, request)).await?;

    Ok(())
}
