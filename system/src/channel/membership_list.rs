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
    Suspected(Vec<Node>),
    LaunchNodes(Vec<SocketAddr>),
    Node(Node),
    Members(Vec<Node>),
    Status(u8),
    Ok,
}

pub async fn get_initial(
    membership_list: &MembershipListSender,
) -> Result<Vec<SocketAddr>, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::GetInitial, request))
        .await?;

    match response.await {
        Ok(MembershipListResponse::LaunchNodes(launch_nodes)) => Ok(launch_nodes),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
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

pub async fn get_suspected(
    membership_list: &MembershipListSender,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::GetSuspected, request))
        .await?;

    match response.await {
        Ok(MembershipListResponse::Ok) => Ok(()),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn get_confirmed(
    membership_list: &MembershipListSender,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::GetConfirmed, request))
        .await?;

    match response.await {
        Ok(MembershipListResponse::Ok) => Ok(()),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn insert_alive(
    membership_list: &MembershipListSender,
    node: Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::InsertAlive(node), request))
        .await?;

    match response.await {
        Ok(MembershipListResponse::Ok) => Ok(()),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn insert_suspected(
    membership_list: &MembershipListSender,
    node: Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::InsertSuspected(node), request))
        .await?;

    match response.await {
        Ok(MembershipListResponse::Ok) => Ok(()),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn insert_confirmed(
    membership_list: &MembershipListSender,
    node: Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::InsertConfirmed(node), request))
        .await?;

    match response.await {
        Ok(MembershipListResponse::Ok) => Ok(()),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn remove_alive(
    membership_list: &MembershipListSender,
    node: Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::RemoveAlive(node), request))
        .await?;

    match response.await {
        Ok(MembershipListResponse::Ok) => Ok(()),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn remove_suspected(
    membership_list: &MembershipListSender,
    node: Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::RemoveSuspected(node), request))
        .await?;

    match response.await {
        Ok(MembershipListResponse::Ok) => Ok(()),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn remove_confirmed(
    membership_list: &MembershipListSender,
    node: Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::RemoveConfirmed(node), request))
        .await?;

    match response.await {
        Ok(MembershipListResponse::Ok) => Ok(()),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}

pub async fn shutdown_membership_list(
    membership_list: &MembershipListSender,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::Shutdown, request))
        .await?;

    match response.await {
        Ok(MembershipListResponse::Ok) => Ok(()),
        Err(error) => Err(Box::new(error)),
        _ => panic!("unexpected response!"),
    }
}
