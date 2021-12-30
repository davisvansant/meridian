use tokio::sync::{mpsc, oneshot};

use crate::node::Node;

pub type MembershipReceiver =
    mpsc::Receiver<(MembershipRequest, oneshot::Sender<MembershipResponse>)>;
pub type MembershipSender = mpsc::Sender<(MembershipRequest, oneshot::Sender<MembershipResponse>)>;

#[derive(Clone, Debug)]
pub enum MembershipRequest {
    JoinCluster(Node),
    Node,
    Members(u8),
    Status,
}

#[derive(Clone, Debug)]
pub enum MembershipResponse {
    JoinCluster(Node),
    Node(Node),
    Members(u8),
    Status,
}

pub async fn join_cluster(
    membership: &MembershipSender,
    node: Node,
) -> Result<Node, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership
        .send((MembershipRequest::JoinCluster(node), request))
        .await?;

    // let node = response.await?;

    // Ok(node)
    match response.await {
        Ok(MembershipResponse::JoinCluster(node)) => Ok(node),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn get_node(membership: &MembershipSender) -> Result<Node, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership.send((MembershipRequest::Node, request)).await?;

    match response.await {
        Ok(MembershipResponse::Node(node)) => Ok(node),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}
