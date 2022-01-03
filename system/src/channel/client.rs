use tokio::sync::{mpsc, oneshot};

pub type ClientReceiver = mpsc::Receiver<(ClientRequest, oneshot::Sender<ClientResponse>)>;
pub type ClientSender = mpsc::Sender<(ClientRequest, oneshot::Sender<ClientResponse>)>;

#[derive(Clone, Debug)]
pub enum ClientRequest {
    // Candidate,
    JoinCluster,
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
