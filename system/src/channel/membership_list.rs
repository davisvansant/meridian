use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};

use crate::node::Node;

pub type MembershipListReceiver = mpsc::Receiver<(
    MembershipListRequest,
    oneshot::Sender<MembershipListResponse>,
)>;
pub type MembershipListSender = mpsc::Sender<(
    MembershipListRequest,
    oneshot::Sender<MembershipListResponse>,
)>;

#[derive(Clone, Debug)]
pub enum MembershipListRequest {
    GetInitial,
    GetAlive,
    GetSuspected,
    GetConfirmed,
    InsertAlive(Node),
    InsertSuspected(Node),
    InsertConfirmed(Node),
    RemoveAlive(Node),
    RemoveSuspected(Node),
    RemoveConfirmed(Node),
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum MembershipListResponse {
    // JoinCluster(Node),
    Alive(Vec<Node>),
    LaunchNodes(Vec<SocketAddr>),
    Node(Node),
    Members(Vec<Node>),
    Status(u8),
    Ok,
}

pub async fn get_alive(
    membership_list: &MembershipListSender,
) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::GetAlive, request))
        .await?;

    match response.await {
        Ok(MembershipListResponse::Alive(alive)) => Ok(alive),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}
