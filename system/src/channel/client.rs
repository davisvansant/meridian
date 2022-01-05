use tokio::sync::{mpsc, oneshot};

pub type ClientReceiver = mpsc::Receiver<(ClientRequest, oneshot::Sender<ClientResponse>)>;
pub type ClientSender = mpsc::Sender<(ClientRequest, oneshot::Sender<ClientResponse>)>;

#[derive(Clone, Debug)]
pub enum ClientRequest {
    // Candidate,
    JoinCluster(String),
    // Node,
    PeerNodes,
    PeerStatus,
    StartElection,
    SendHeartbeat,
}

#[derive(Clone, Debug)]
pub enum ClientResponse {
    JoinCluster,
    // Node,
    MemberNodes,
    MemberStatus,
}

pub async fn start_election(client: &ClientSender) -> Result<(), Box<dyn std::error::Error>> {
    let (_request, _response) = oneshot::channel();

    client
        .send((ClientRequest::StartElection, _request))
        .await?;

    Ok(())
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
    address: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_request, _response) = oneshot::channel();

    client
        .send((ClientRequest::JoinCluster(address), _request))
        .await?;

    Ok(())
}

pub async fn peer_nodes(client: &ClientSender) -> Result<(), Box<dyn std::error::Error>> {
    let (_request, _response) = oneshot::channel();

    client.send((ClientRequest::PeerNodes, _request)).await?;

    Ok(())
}
