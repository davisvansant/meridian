use std::fmt;

use tokio::sync::{mpsc, oneshot};

use crate::node::Node;

pub mod failure_detector;
pub mod list;
pub mod sender;

pub type MembershipReceiver =
    mpsc::Receiver<(MembershipRequest, oneshot::Sender<MembershipResponse>)>;
pub type MembershipSender = mpsc::Sender<(MembershipRequest, oneshot::Sender<MembershipResponse>)>;

#[derive(Clone, Debug)]
pub enum MembershipRequest {
    FailureDectector,
    Members,
    Node,
    StaticJoin,
    // Status,
    Shutdown,
}

impl MembershipRequest {
    pub async fn build() -> (MembershipSender, MembershipReceiver) {
        let (membership_sender, membership_receiver) =
            mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);

        (membership_sender, membership_receiver)
    }

    pub async fn launch_failure_detector(
        membership: &MembershipSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (_request, _response) = oneshot::channel();

        membership
            .send((MembershipRequest::FailureDectector, _request))
            .await?;

        Ok(())
    }

    pub async fn node(membership: &MembershipSender) -> Result<Node, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        membership.send((MembershipRequest::Node, request)).await?;

        match response.await? {
            MembershipResponse::Node(node) => Ok(node),
            _ => Err(Box::from(
                "unexpected response for membership node request!",
            )),
        }
    }

    pub async fn cluster_members(
        membership: &MembershipSender,
    ) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        membership
            .send((MembershipRequest::Members, request))
            .await?;

        match response.await? {
            MembershipResponse::Members(cluster_members) => Ok(cluster_members),
            _ => Err(Box::from(
                "unexpected response for membership members request!",
            )),
        }
    }

    pub async fn static_join(
        membership: &MembershipSender,
    ) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let (request, response) = oneshot::channel();

        membership
            .send((MembershipRequest::StaticJoin, request))
            .await?;

        match response.await? {
            MembershipResponse::Status((active_peers, expected_peers)) => {
                Ok((active_peers, expected_peers))
            }
            _ => Err(Box::from("unexpected response for membership static join!")),
        }
    }

    // pub async fn status(membership: &MembershipSender) -> Result<u8, Box<dyn std::error::Error>> {
    //     let (request, response) = oneshot::channel();

    //     membership
    //         .send((MembershipRequest::Status, request))
    //         .await?;

    //     match response.await {
    //         Ok(MembershipResponse::Status(connected_nodes)) => Ok(connected_nodes),
    //         Err(error) => Err(Box::new(error)),
    //         _ => panic!("unexpected response!"),
    //     }
    // }

    pub async fn shutdown(membership: &MembershipSender) -> Result<(), Box<dyn std::error::Error>> {
        let (_request, _response) = oneshot::channel();

        membership
            .send((MembershipRequest::Shutdown, _request))
            .await?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum MembershipResponse {
    Node(Node),
    Members(Vec<Node>),
    Status((usize, usize)),
}

impl fmt::Display for MembershipResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let membership_response = "Membership Response |";

        match self {
            MembershipResponse::Node(node) => {
                write!(f, "{} node -> {:?}", membership_response, node)
            }
            MembershipResponse::Members(members) => {
                write!(f, "{} members -> {:?}", membership_response, members)
            }
            MembershipResponse::Status((active, received)) => {
                write!(
                    f,
                    "{} status -> active {} | received {}",
                    membership_response, active, received,
                )
            }
        }
    }
}

impl std::error::Error for MembershipResponse {}
