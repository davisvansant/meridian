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
    GetNode,
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
    Node(Node),
    Initial(Vec<SocketAddr>),
    Alive(Vec<Node>),
    Suspected(Vec<Node>),
    Confirmed(Vec<Node>),
}

pub async fn build() -> (MembershipListSender, MembershipListReceiver) {
    let (list_sender, list_receiver) = mpsc::channel::<(
        MembershipListRequest,
        oneshot::Sender<MembershipListResponse>,
    )>(64);

    (list_sender, list_receiver)
}

pub async fn get_node(
    membership_list: &MembershipListSender,
) -> Result<Node, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::GetNode, request))
        .await?;

    match response.await? {
        MembershipListResponse::Node(node) => Ok(node),
        _ => panic!("unexpected response!"),
    }
}

pub async fn get_initial(
    membership_list: &MembershipListSender,
) -> Result<Vec<SocketAddr>, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::GetInitial, request))
        .await?;

    match response.await? {
        MembershipListResponse::Initial(initial) => Ok(initial),
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

    match response.await? {
        MembershipListResponse::Alive(alive) => Ok(alive),
        _ => panic!("unexpected response!"),
    }
}

pub async fn get_suspected(
    membership_list: &MembershipListSender,
) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::GetSuspected, request))
        .await?;

    match response.await? {
        MembershipListResponse::Suspected(suspected) => Ok(suspected),
        _ => panic!("unexpected response!"),
    }
}

pub async fn get_confirmed(
    membership_list: &MembershipListSender,
) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::GetConfirmed, request))
        .await?;

    match response.await? {
        MembershipListResponse::Confirmed(confirmed) => Ok(confirmed),
        _ => panic!("unexpected response!"),
    }
}

pub async fn insert_alive(
    membership_list: &MembershipListSender,
    node: &Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_request, _response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::InsertAlive(*node), _request))
        .await?;

    Ok(())
}

pub async fn insert_suspected(
    membership_list: &MembershipListSender,
    node: &Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_request, _response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::InsertSuspected(*node), _request))
        .await?;

    Ok(())
}

pub async fn insert_confirmed(
    membership_list: &MembershipListSender,
    node: &Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::InsertConfirmed(*node), request))
        .await?;

    Ok(())
}

pub async fn remove_alive(
    membership_list: &MembershipListSender,
    node: &Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_request, _response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::RemoveAlive(*node), _request))
        .await?;

    Ok(())
}

pub async fn remove_suspected(
    membership_list: &MembershipListSender,
    node: &Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_request, _response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::RemoveSuspected(*node), _request))
        .await?;

    Ok(())
}

pub async fn remove_confirmed(
    membership_list: &MembershipListSender,
    node: &Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_request, _response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::RemoveConfirmed(*node), _request))
        .await?;

    Ok(())
}

pub async fn shutdown(
    membership_list: &MembershipListSender,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    membership_list
        .send((MembershipListRequest::Shutdown, request))
        .await?;

    Ok(())
}
