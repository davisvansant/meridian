use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};

use crate::node::Node;

pub type MembershipDynamicJoinShutdown = mpsc::Receiver<bool>;
pub type MembershipReceiver =
    mpsc::Receiver<(MembershipRequest, oneshot::Sender<MembershipResponse>)>;
pub type MembershipSender = mpsc::Sender<(MembershipRequest, oneshot::Sender<MembershipResponse>)>;

#[derive(Clone, Debug)]
pub enum MembershipRequest {
    // JoinCluster(Node),
    AddMember(Node),
    LaunchNodes,
    Members,
    Node,
    RemoveMember,
    Status,
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum MembershipResponse {
    // JoinCluster(Node),
    LaunchNodes(Vec<SocketAddr>),
    Node(Node),
    Members(Vec<Node>),
    Status(u8),
    Ok,
}

// pub async fn join_cluster(
//     membership: &MembershipSender,
//     node: Node,
// ) -> Result<Node, Box<dyn std::error::Error>> {
//     let (request, response) = oneshot::channel();

//     membership
//         .send((MembershipRequest::JoinCluster(node), request))
//         .await?;

//     // let node = response.await?;

//     // Ok(node)
//     match response.await {
//         Ok(MembershipResponse::JoinCluster(node)) => Ok(node),
//         Err(error) => Err(Box::new(error)),
//         _ => panic!("unexpected response!"),
//     }
// }

pub async fn add_member(
    membership: &MembershipSender,
    node: Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership
        .send((MembershipRequest::AddMember(node), request))
        .await?;

    match response.await {
        Ok(MembershipResponse::Ok) => Ok(()),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn launch_nodes(
    membership: &MembershipSender,
) -> Result<Vec<SocketAddr>, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership
        .send((MembershipRequest::LaunchNodes, request))
        .await?;

    match response.await {
        Ok(MembershipResponse::LaunchNodes(launch_nodes)) => Ok(launch_nodes),
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

pub async fn cluster_members(
    membership: &MembershipSender,
) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership
        .send((MembershipRequest::Members, request))
        .await?;

    match response.await {
        Ok(MembershipResponse::Members(cluster_members)) => Ok(cluster_members),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn status(membership: &MembershipSender) -> Result<u8, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership
        .send((MembershipRequest::Status, request))
        .await?;

    match response.await {
        Ok(MembershipResponse::Status(connected_nodes)) => Ok(connected_nodes),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn shutdown_membership(
    membership: &MembershipSender,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    membership
        .send((MembershipRequest::Shutdown, request))
        .await?;

    Ok(())
}
