use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};

pub type ClientReceiver = mpsc::Receiver<(ClientRequest, oneshot::Sender<ClientResponse>)>;
pub type ClientSender = mpsc::Sender<(ClientRequest, oneshot::Sender<ClientResponse>)>;

#[derive(Clone, Debug)]
pub enum ClientRequest {
    // Candidate,
    JoinCluster(SocketAddr),
    // Node,
    PeerNodes(SocketAddr),
    PeerStatus(SocketAddr),
    StartElection,
    SendHeartbeat,
}

#[derive(Clone, Debug)]
pub enum ClientResponse {
    JoinCluster(SocketAddr),
    // Node,
    Nodes(Vec<SocketAddr>),
    Status(u8),
    MemberNodes,
    MemberStatus,
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

    // Ok(())
}

pub async fn send_heartbeat(client: &ClientSender) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    client.send((ClientRequest::SendHeartbeat, request)).await?;

    // match response.await {
    //     Ok(StateResponse::Candidate(request_vote_arguments)) => Ok(request_vote_arguments),
    //     Err(error) => Err(Box::new(error)),
    //     _ => panic!("unexpected response!"),
    // }
    Ok(())
}

pub async fn join_cluster(
    client: &ClientSender,
    address: SocketAddr,
) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    client
        .send((ClientRequest::JoinCluster(address), request))
        .await?;

    match response.await {
        Ok(ClientResponse::JoinCluster(connected_node)) => Ok(connected_node),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }

    // Ok(())
}

pub async fn peer_nodes(
    client: &ClientSender,
    socket_address: SocketAddr,
) -> Result<Vec<SocketAddr>, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    client
        .send((ClientRequest::PeerNodes(socket_address), request))
        .await?;

    match response.await {
        Ok(ClientResponse::Nodes(peer_nodes)) => Ok(peer_nodes),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn peer_status(
    client: &ClientSender,
    socket_address: SocketAddr,
) -> Result<u8, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    client
        .send((ClientRequest::PeerStatus(socket_address), request))
        .await?;

    match response.await {
        Ok(ClientResponse::Status(peer_status)) => Ok(peer_status),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}
