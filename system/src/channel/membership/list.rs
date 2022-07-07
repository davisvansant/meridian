use std::fmt;
use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};

use crate::node::Node;

pub type ListReceiver = mpsc::Receiver<(ListRequest, oneshot::Sender<ListResponse>)>;
pub type ListSender = mpsc::Sender<(ListRequest, oneshot::Sender<ListResponse>)>;

#[derive(Clone, Debug)]
pub enum ListRequest {
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
pub enum ListResponse {
    Node(Node),
    Initial(Vec<SocketAddr>),
    Alive(Vec<Node>),
    Suspected(Vec<Node>),
    Confirmed(Vec<Node>),
}

impl fmt::Display for ListResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let list_response = "List Response |";

        match self {
            ListResponse::Node(node) => {
                write!(f, "{} node -> {:?}", list_response, node)
            }
            ListResponse::Initial(peers) => {
                write!(f, "{} initial -> {:?}", list_response, peers)
            }
            ListResponse::Alive(alive_members) => {
                write!(f, "{} alive -> {:?}", list_response, alive_members)
            }
            ListResponse::Suspected(suspected_members) => {
                write!(f, "{} suspected -> {:?}", list_response, suspected_members)
            }
            ListResponse::Confirmed(confirmed_members) => {
                write!(f, "{} confirmed -> {:?}", list_response, confirmed_members)
            }
        }
    }
}

impl std::error::Error for ListResponse {}

#[derive(Clone, Debug)]
pub struct ListChannel {
    request: ListSender,
}

impl ListChannel {
    pub async fn init() -> (ListChannel, ListReceiver) {
        let (request, response) = mpsc::channel::<(ListRequest, oneshot::Sender<ListResponse>)>(64);

        (ListChannel { request }, response)
    }

    pub async fn get_node(&self) -> Result<Node, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        self.request.send((ListRequest::GetNode, request)).await?;

        match response.await? {
            ListResponse::Node(node) => Ok(node),
            _ => Err(Box::from("unexpected list get node response!")),
        }
    }

    pub async fn get_initial(&self) -> Result<Vec<SocketAddr>, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        self.request
            .send((ListRequest::GetInitial, request))
            .await?;

        match response.await? {
            ListResponse::Initial(initial) => Ok(initial),
            _ => Err(Box::from("unexpected list get initial response!")),
        }
    }

    pub async fn get_alive(&self) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        self.request.send((ListRequest::GetAlive, request)).await?;

        match response.await? {
            ListResponse::Alive(alive) => Ok(alive),
            _ => Err(Box::from("unexpected list get alive response!")),
        }
    }

    pub async fn get_suspected(&self) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        self.request
            .send((ListRequest::GetSuspected, request))
            .await?;

        match response.await? {
            ListResponse::Suspected(suspected) => Ok(suspected),
            _ => Err(Box::from("unexpected list get suspected response!")),
        }
    }

    pub async fn get_confirmed(&self) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        self.request
            .send((ListRequest::GetConfirmed, request))
            .await?;

        match response.await? {
            ListResponse::Confirmed(confirmed) => Ok(confirmed),
            _ => Err(Box::from("unexpected list get confirmed response!")),
        }
    }

    pub async fn insert_alive(&self, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        let (_request, _response) = oneshot::channel();

        self.request
            .send((ListRequest::InsertAlive(*node), _request))
            .await?;

        Ok(())
    }

    pub async fn insert_suspected(&self, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        let (_request, _response) = oneshot::channel();

        self.request
            .send((ListRequest::InsertSuspected(*node), _request))
            .await?;

        Ok(())
    }

    pub async fn insert_confirmed(&self, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        let (_request, _response) = oneshot::channel();

        self.request
            .send((ListRequest::InsertConfirmed(*node), _request))
            .await?;

        Ok(())
    }

    pub async fn remove_alive(&self, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        let (_request, _response) = oneshot::channel();

        self.request
            .send((ListRequest::RemoveAlive(*node), _request))
            .await?;

        Ok(())
    }

    pub async fn remove_suspected(&self, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        let (_request, _response) = oneshot::channel();

        self.request
            .send((ListRequest::RemoveSuspected(*node), _request))
            .await?;

        Ok(())
    }

    pub async fn remove_confirmed(&self, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        let (_request, _response) = oneshot::channel();

        self.request
            .send((ListRequest::RemoveConfirmed(*node), _request))
            .await?;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (request, _response) = oneshot::channel();

        self.request.send((ListRequest::Shutdown, request)).await?;

        Ok(())
    }
}
