use tokio::time::Duration;
use tonic::transport::Endpoint;

// use crate::grpc::cluster_client::InternalClusterGrpcClient;
// use crate::meridian_cluster_v010::RequestVoteRequest;

// use tokio::time::Duration;

use crate::server::ServerState;

use tokio::time::timeout_at;

use tokio::time::Instant;

// use tokio::sync::broadcast::Receiver;

use tokio::sync::broadcast::Sender;
// use tokio::sync::mpsc::Sender;

use tokio::sync::mpsc::Receiver;

use crate::channel::{CandidateReceiver, CandidateTransition};

use crate::channel::start_election;

use crate::channel::ClientSender;

use crate::server::ServerSender;

pub struct Candidate {
    pub election_timeout: Duration,
}

impl Candidate {
    pub async fn init() -> Result<Candidate, Box<dyn std::error::Error>> {
        let election_timeout = Duration::from_millis(150);
        Ok(Candidate { election_timeout })
    }

    // pub async fn start_election(
    //     &self,
    //     request: RequestVoteRequest,
    //     address: Endpoint,
    // ) -> Result<bool, Box<dyn std::error::Error>> {
    //     // println!("starting election...");

    //     // increment term
    //     // vote for self
    //     // reset election timer
    //     // send request vote rpc to servers

    //     let mut transport = InternalClusterGrpcClient::init(address).await?;
    //     let result = transport.request_vote(request).await?;

    //     match result.into_inner().vote_granted.as_str() {
    //         "true" => Ok(true),
    //         "false" => Ok(false),
    //         _ => Ok(false),
    //     }
    // }

    pub async fn run(
        &mut self,
        // transition: &mut mpsc::Receiver<CandidateTransition>,
        client: &ClientSender,
        transition: &mut CandidateReceiver,
        // tx: &Sender<ServerState>,
        tx: &ServerSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        start_election(client).await?;

        loop {
            match timeout_at(Instant::now() + self.election_timeout, transition.recv()).await {
                Ok(state) => {
                    match state {
                        Some(CandidateTransition::Follower) => {
                            println!("received heartbeat...stepping down");

                            // if let Err(error) = tx.send(ServerState::Follower) {
                            //     println!("error sending server state {:?}", error);
                            // }

                            break;
                        }
                        Some(CandidateTransition::Leader) => {
                            // if let Err(error) = tx.send(ServerState::Leader) {
                            //     println!("error sending server state {:?}", error);
                            // }
                            println!("transitioning server to leader...");
                            break;
                        }
                        None => break,
                    }
                }
                Err(error) => println!("candidate election timeout lapsed...trying again..."),
            }
        }
        // while let Ok(action) =
        //     timeout_at(Instant::now() + self.election_timeout, transition.recv()).await
        // {
        //     match action {
        //         Some(CandidateTransition::Follower) => {
        //             println!("received heartbeat...stepping down");

        //             // if let Err(error) = tx.send(ServerState::Follower) {
        //             //     println!("error sending server state {:?}", error);
        //             // }

        //             // break;
        //         }
        //         Some(CandidateTransition::Leader) => {
        //             if let Err(error) = tx.send(ServerState::Leader) {
        //                 println!("error sending server state {:?}", error);
        //             }
        //             // println!("sending receive request to cluster members - {:?}", request);

        //             // self.membership_send_task
        //             //     .send(MembershipReceiveTask::Members(2))?;

        //             // if let Ok(MembershipSendServerTask::MembersResponse(members)) =
        //             //     membership_receiver.recv().await
        //             // {
        //             //     println!("members ! {:?}", &members);

        //             //     let mut nodes = Vec::with_capacity(members.len());

        //             //     for member in &members {
        //             //         let address = member.address;
        //             //         let port = member.cluster_port;
        //             //         let mut node = String::with_capacity(20);

        //             //         node.push_str("http://");
        //             //         node.push_str(&address.to_string());
        //             //         node.push(':');
        //             //         node.push_str(&port.to_string());
        //             //         node.shrink_to_fit();

        //             //         nodes.push(node)
        //             //     }

        //             //     if nodes.is_empty() {
        //             //         self.server_state = ServerState::Leader;
        //             //     } else {
        //             //         let mut vote_true = Vec::with_capacity(3);
        //             //         let mut vote_false = Vec::with_capacity(3);

        //             //         for node in nodes {
        //             //             println!("sending request to node - {:?}", &node);
        //             //             let endpoint = Endpoint::try_from(node)?;
        //             //             if candidate.start_election(request.clone(), endpoint).await? {
        //             //                 vote_true.push(1);
        //             //             } else {
        //             //                 vote_false.push(1);
        //             //             }
        //             //         }
        //             //         if vote_true >= vote_false {
        //             //             self.server_state = ServerState::Leader;
        //             //         } else {
        //             //             self.server_state = ServerState::Follower;
        //             //         }
        //             //     }
        //             // }
        //         }
        //     }
        // }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_candidate = Candidate::init().await?;
        assert_eq!(test_candidate.election_timeout.as_millis(), 150);
        Ok(())
    }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn start_election() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_candidate = Candidate::init().await?;
    //     // let test_server = Server::init().await?;
    //     let test_server = test_server().await?;
    //     let test_request = test_server.build_request_vote_request().await?;
    //     assert!(test_candidate.start_election(test_request).await.is_ok());
    //     Ok(())
    // }
}
