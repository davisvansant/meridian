use std::convert::TryFrom;
use tokio::time::{sleep, timeout, timeout_at, Duration, Instant};
use tonic::transport::Endpoint;

use crate::server::candidate::Candidate;
use crate::server::follower::Follower;
use crate::server::leader::Leader;

pub mod candidate;
pub mod follower;
pub mod leader;
mod preflight;

use crate::channel::ClientSender;
use crate::channel::MembershipSender;
use crate::channel::StateSender;
use crate::channel::{ServerReceiver, ServerSender, ServerState};

use crate::channel::{CandidateReceiver, CandidateTransition};

// use crate::runtime::sync::membership_receive_task::ChannelMembershipReceiveTask;
// use crate::runtime::sync::membership_send_server_task::ChannelMembershipSendServerTask;
// use crate::runtime::sync::state_receive_task::ChannelStateReceiveTask;
// use crate::runtime::sync::state_send_server_task::ChannelStateSendServerTask;

// use crate::runtime::sync::membership_receive_task::MembershipReceiveTask;
// use crate::runtime::sync::membership_send_server_task::MembershipSendServerTask;
// use crate::runtime::sync::state_receive_task::StateReceiveTask;
// use crate::runtime::sync::state_send_server_task::StateSendServerTask;

use tokio::sync::{broadcast, mpsc, oneshot, watch};

// #[derive(Debug, PartialEq)]
// pub enum ServerState {
//     Follower,
//     Candidate,
//     Leader,
// }
// pub struct System {
//     state: ServerState,
// }

pub struct Server {
    pub server_state: ServerState,
    // receive_task: ChannelStateSendServerTask,
    // send_task: ChannelStateReceiveTask,
    // membership_send_task: ChannelMembershipReceiveTask,
    // membership_receive_task: ChannelMembershipSendServerTask,
    client: ClientSender,
    membership: MembershipSender,
    // receiver: ServerReceiver,
    state: StateSender,
    tx: ServerSender,
    rx: ServerReceiver,
}

impl Server {
    pub async fn init(
        // receive_task: ChannelStateSendServerTask,
        // send_task: ChannelStateReceiveTask,
        // membership_send_task: ChannelMembershipReceiveTask,
        // membership_receive_task: ChannelMembershipSendServerTask,
        client: ClientSender,
        membership: MembershipSender,
        // receiver: ServerReceiver,
        state: StateSender,
        tx: ServerSender,
        rx: ServerReceiver,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        let server_state = ServerState::Follower;

        Ok(Server {
            server_state,
            // receive_task,
            // send_task,
            // membership_send_task,
            // membership_receive_task,
            client,
            membership,
            // receiver,
            state,
            tx,
            rx,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // let mut receiver = self.receive_task.subscribe();

        // println!("waiting for nodes to join...");
        // sleep(Duration::from_secs(15)).await;

        // let await_
        // loop {
        //     match self.server_state {
        //         ServerState::Follower => println!("im a follower"),
        //         ServerState::Candidate => println!("im a candidate"),
        //         ServerState::Leader => println!("im a leader"),
        //         ServerState::Shutdown => println!("time to shutdown..."),
        //     }
        // }
        // let (tx, mut rx) = watch::channel(String::from("preflight"));
        // let (tx, mut rx) = mpsc::channel::<ServerState>(64);
        // let (tx, mut rx) = broadcast::channel::<ServerState>(64);
        let (sender, mut receiver) = broadcast::channel::<ServerState>(64);

        let (candidate_sender, mut candidate_receiver) = mpsc::channel::<CandidateTransition>(64);

        let launch = sender.clone();
        // let mut follower_receiver = sender.subscribe();
        // let mut candidate_receiver = sender.subscribe();
        // let mut follower_receiver = tx.clone();
        // let mut candidate_receiver = tx.clone();
        let mut follower_receiver = self.tx.to_owned().subscribe();
        let mut rx = self.tx.to_owned().subscribe();
        // let mut candidate_receiver = tx.subscribe();

        tokio::spawn(async move {
            // let mut system = System {
            //     state: ServerState::Shutdown,
            // };

            while let Ok(transition) = rx.recv().await {
                match transition {
                    ServerState::Preflight => {
                        println!("some preflight stuff");

                        // system.state = ServerState::Preflight;

                        if let Err(error) = sender.send(ServerState::Preflight) {
                            println!("preflight error {:?}", error);
                        }
                    }
                    ServerState::Follower => {
                        println!("follwer");

                        if let Err(error) = sender.send(ServerState::Follower) {
                            println!("preflight error {:?}", error);
                        }
                    }
                    ServerState::Candidate => {
                        println!("candidate");

                        if let Err(error) = sender.send(ServerState::Candidate) {
                            println!("preflight error {:?}", error);
                        }
                    }
                    ServerState::Leader => {
                        println!("leader");

                        // system.state = ServerState::Leader;

                        if let Err(error) = candidate_sender.send(CandidateTransition::Leader).await
                        {
                            println!("error sending transition...");
                        }

                        if let Err(error) = sender.send(ServerState::Leader) {
                            println!("leader error {:?}", error);
                        }
                    }
                    ServerState::Shutdown => {
                        println!("shutdown");

                        if let Err(error) = sender.send(ServerState::Shutdown) {
                            println!("leader error {:?}", error);
                        }

                        break;
                    }
                }
            }
        });

        // launch.send(ServerState::Preflight)?;
        self.tx.send(ServerState::Preflight)?;

        while let Ok(transition) = receiver.recv().await {
            match transition {
                ServerState::Preflight => {
                    println!("running preflight tasks...");

                    preflight::run(&self.client, &self.membership).await?;

                    if let Err(error) = self.tx.send(ServerState::Follower) {
                        println!("error sending server state {:?}", error);
                    }
                }
                ServerState::Follower => {
                    println!("server > follower!");

                    let mut follower = Follower::init().await?;
                    follower.run(&mut follower_receiver).await?;

                    if let Err(error) = self.tx.send(ServerState::Candidate) {
                        println!("error sending server state {:?}", error);
                    }
                }
                ServerState::Candidate => {
                    println!("server > candidate!");

                    let mut candidate = Candidate::init().await?;
                    candidate
                        .run(&self.client, &mut candidate_receiver, &self.tx)
                        .await?;
                }
                ServerState::Leader => {
                    println!("server > leader!");

                    let mut leader = Leader::init().await?;
                    leader.run(&self.client, &self.state).await?;
                }
                ServerState::Shutdown => {
                    println!("server > shutdown...");
                    // self.server_state = ServerState::Shutdown;

                    // drop(self.client.to_owned());
                    // drop(self.membership.to_owned());
                    // drop(self.state.to_owned());

                    break;
                }
            }
        }

        // tokio::spawn(async move {
        //     while let Ok(action) = receiver.recv().await {
        //         match action {
        //             StateSendServerTask::AppendEntriesRequest(_) => {
        //                 println!("received append entries");
        //             }
        //             StateSendServerTask::Follower => println!("follower"),
        //             StateSendServerTask::RequestVoteRequest(request) => println!("{:?}", request),
        //         }
        //     }
        // });

        // loop {
        //     match self.server_state {
        //         ServerState::Follower => {
        //             println!("doing follwer stuff!");

        //             if let Err(error) = self.follower().await {
        //                 println!("Something went wrong with the follower - {:?}", error);
        //                 break;
        //             }

        //             println!("transitioning to candidate...");
        //         }
        //         ServerState::Candidate => {
        //             // sleep(Duration::from_secs(10)).await;
        //             println!("doing candidate stuff!");

        //             if let Err(error) = self.candidate().await {
        //                 println!("something went wrong with the candidate {:?}", error);
        //                 break;
        //             }

        //             // println!("transitioning to leader...");
        //         }
        //         ServerState::Leader => {
        //             // sleep(Duration::from_secs(10)).await;
        //             println!("Leader!");

        //             if let Err(error) = self.leader().await {
        //                 println!("the leader had an error - {:?}", error);
        //                 break;
        //             }
        //         }
        //     }
        // }

        Ok(())
    }

    // pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut receiver = self.receive_task.subscribe();

    //     println!("waiting for nodes to join...");
    //     sleep(Duration::from_secs(15)).await;

    //     tokio::spawn(async move {
    //         while let Ok(action) = receiver.recv().await {
    //             match action {
    //                 StateSendServerTask::AppendEntriesRequest(_) => {
    //                     println!("received append entries");
    //                 }
    //                 StateSendServerTask::Follower => println!("follower"),
    //                 StateSendServerTask::RequestVoteRequest(request) => println!("{:?}", request),
    //             }
    //         }
    //     });

    //     loop {
    //         match self.server_state {
    //             ServerState::Follower => {
    //                 println!("doing follwer stuff!");

    //                 if let Err(error) = self.follower().await {
    //                     println!("Something went wrong with the follower - {:?}", error);
    //                     break;
    //                 }

    //                 println!("transitioning to candidate...");
    //             }
    //             ServerState::Candidate => {
    //                 // sleep(Duration::from_secs(10)).await;
    //                 println!("doing candidate stuff!");

    //                 if let Err(error) = self.candidate().await {
    //                     println!("something went wrong with the candidate {:?}", error);
    //                     break;
    //                 }

    //                 // println!("transitioning to leader...");
    //             }
    //             ServerState::Leader => {
    //                 // sleep(Duration::from_secs(10)).await;
    //                 println!("Leader!");

    //                 if let Err(error) = self.leader().await {
    //                     println!("the leader had an error - {:?}", error);
    //                     break;
    //                 }
    //             }
    //         }
    //     }

    //     Ok(())
    // }

    // pub async fn follower(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut receiver = self.receive_task.subscribe();
    //     let follower = Follower::init().await?;

    //     while let Ok(result) =
    //         timeout_at(Instant::now() + follower.election_timeout, receiver.recv()).await
    //     {
    //         if let Ok(StateSendServerTask::Follower) = result {
    //             println!("receiving heartbeat...");
    //         }
    //     }

    //     println!("timeout ending...starting election");
    //     self.server_state = ServerState::Candidate;

    //     Ok(())
    // }

    // pub async fn candidate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    //     // let mut receiver = self.receive_task.subscribe();
    //     // let mut membership_receiver = self.membership_receive_task.subscribe();

    //     // self.membership_send_task
    //     //     .send(MembershipReceiveTask::Node(2))?;

    //     // if let Ok(MembershipSendServerTask::NodeResponse(node)) = membership_receiver.recv().await {
    //     //     self.send_task
    //     //         .send(StateReceiveTask::Candidate(node.id.to_string()))?;
    //     // };

    //     let candidate = Candidate::init().await?;

    //     while let Ok(action) =
    //         timeout_at(Instant::now() + candidate.election_timeout, receiver.recv()).await
    //     {
    //         match action {
    //             Ok(StateSendServerTask::Follower) => {
    //                 println!("received heartbeat...stepping down");
    //                 self.server_state = ServerState::Follower;
    //                 break;
    //             }
    //             Ok(StateSendServerTask::RequestVoteRequest(request)) => {
    //                 println!("sending receive request to cluster members - {:?}", request);

    //                 self.membership_send_task
    //                     .send(MembershipReceiveTask::Members(2))?;

    //                 if let Ok(MembershipSendServerTask::MembersResponse(members)) =
    //                     membership_receiver.recv().await
    //                 {
    //                     println!("members ! {:?}", &members);

    //                     let mut nodes = Vec::with_capacity(members.len());

    //                     for member in &members {
    //                         let address = member.address;
    //                         let port = member.cluster_port;
    //                         let mut node = String::with_capacity(20);

    //                         node.push_str("http://");
    //                         node.push_str(&address.to_string());
    //                         node.push(':');
    //                         node.push_str(&port.to_string());
    //                         node.shrink_to_fit();

    //                         nodes.push(node)
    //                     }

    //                     if nodes.is_empty() {
    //                         self.server_state = ServerState::Leader;
    //                     } else {
    //                         let mut vote_true = Vec::with_capacity(3);
    //                         let mut vote_false = Vec::with_capacity(3);

    //                         for node in nodes {
    //                             println!("sending request to node - {:?}", &node);
    //                             let endpoint = Endpoint::try_from(node)?;
    //                             if candidate.start_election(request.clone(), endpoint).await? {
    //                                 vote_true.push(1);
    //                             } else {
    //                                 vote_false.push(1);
    //                             }
    //                         }
    //                         if vote_true >= vote_false {
    //                             self.server_state = ServerState::Leader;
    //                         } else {
    //                             self.server_state = ServerState::Follower;
    //                         }
    //                     }
    //                 }
    //             }
    //             _ => println!("cannot do anyhting with other requests"),
    //         }
    //     }

    //     Ok(())
    // }

    // pub async fn candidate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut receiver = self.receive_task.subscribe();
    //     let mut membership_receiver = self.membership_receive_task.subscribe();

    //     self.membership_send_task
    //         .send(MembershipReceiveTask::Node(2))?;

    //     if let Ok(MembershipSendServerTask::NodeResponse(node)) = membership_receiver.recv().await {
    //         self.send_task
    //             .send(StateReceiveTask::Candidate(node.id.to_string()))?;
    //     };

    //     let candidate = Candidate::init().await?;

    //     while let Ok(action) =
    //         timeout_at(Instant::now() + candidate.election_timeout, receiver.recv()).await
    //     {
    //         match action {
    //             Ok(StateSendServerTask::Follower) => {
    //                 println!("received heartbeat...stepping down");
    //                 self.server_state = ServerState::Follower;
    //                 break;
    //             }
    //             Ok(StateSendServerTask::RequestVoteRequest(request)) => {
    //                 println!("sending receive request to cluster members - {:?}", request);

    //                 self.membership_send_task
    //                     .send(MembershipReceiveTask::Members(2))?;

    //                 if let Ok(MembershipSendServerTask::MembersResponse(members)) =
    //                     membership_receiver.recv().await
    //                 {
    //                     println!("members ! {:?}", &members);

    //                     let mut nodes = Vec::with_capacity(members.len());

    //                     for member in &members {
    //                         let address = member.address;
    //                         let port = member.cluster_port;
    //                         let mut node = String::with_capacity(20);

    //                         node.push_str("http://");
    //                         node.push_str(&address.to_string());
    //                         node.push(':');
    //                         node.push_str(&port.to_string());
    //                         node.shrink_to_fit();

    //                         nodes.push(node)
    //                     }

    //                     if nodes.is_empty() {
    //                         self.server_state = ServerState::Leader;
    //                     } else {
    //                         let mut vote_true = Vec::with_capacity(3);
    //                         let mut vote_false = Vec::with_capacity(3);

    //                         for node in nodes {
    //                             println!("sending request to node - {:?}", &node);
    //                             let endpoint = Endpoint::try_from(node)?;
    //                             if candidate.start_election(request.clone(), endpoint).await? {
    //                                 vote_true.push(1);
    //                             } else {
    //                                 vote_false.push(1);
    //                             }
    //                         }
    //                         if vote_true >= vote_false {
    //                             self.server_state = ServerState::Leader;
    //                         } else {
    //                             self.server_state = ServerState::Follower;
    //                         }
    //                     }
    //                 }
    //             }
    //             _ => println!("cannot do anyhting with other requests"),
    //         }
    //     }

    //     Ok(())
    // }

    // pub async fn leader(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut receiver = self.receive_task.subscribe();
    //     let mut membership_receiver = self.membership_receive_task.subscribe();

    //     let leader = Leader::init().await?;

    //     self.membership_send_task
    //         .send(MembershipReceiveTask::Node(2))?;

    //     if let Ok(MembershipSendServerTask::NodeResponse(node)) = membership_receiver.recv().await {
    //         println!("server uuid - {:?}", &node);

    //         self.send_task
    //             .send(StateReceiveTask::Leader(node.id.to_string()))?;

    //         if let Ok(StateSendServerTask::AppendEntriesRequest(request)) = receiver.recv().await {
    //             self.membership_send_task
    //                 .send(MembershipReceiveTask::Members(2))?;

    //             if let Ok(MembershipSendServerTask::MembersResponse(members)) =
    //                 membership_receiver.recv().await
    //             {
    //                 println!("{:?}", &members);
    //                 println!("sending heartbeat ... {:?}", &request);

    //                 let mut nodes = Vec::with_capacity(members.len());

    //                 for member in &members {
    //                     let address = member.address;
    //                     let port = member.cluster_port;
    //                     let mut node = String::with_capacity(20);

    //                     node.push_str("http://");
    //                     node.push_str(&address.to_string());
    //                     node.push(':');
    //                     node.push_str(&port.to_string());
    //                     node.shrink_to_fit();

    //                     nodes.push(node)
    //                 }

    //                 if !nodes.is_empty() {
    //                     for node in nodes {
    //                         let endpoint = Endpoint::try_from(node)?;
    //                         leader.send_heartbeat(request.clone(), endpoint).await?;
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[tokio::test(flavor = "multi_thread")]
    // async fn server_state_candidate() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_server_state = ServerState::Candidate;
    //     assert_eq!(test_server_state, ServerState::Candidate);
    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn server_state_follower() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_server_state = ServerState::Follower;
    //     assert_eq!(test_server_state, ServerState::Follower);
    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn server_state_leader() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_server_state = ServerState::Leader;
    //     assert_eq!(test_server_state, ServerState::Leader);
    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn init() -> Result<(), Box<dyn std::error::Error>> {
    //     // let (test_sender, _) = test_channel().await;
    //     // let (test_subscriber, _) = test_channel().await;
    //     // let test_subscriber_clone = test_subscriber.clone();
    //     // let test_server = Server::init(test_subscriber, test_subscriber_clone).await?;
    //     let test_server = test_server().await?;
    //     // assert_eq!(test_server.server_state, ServerState::Follower);
    //     assert_eq!(test_server.persistent_state.current_term, 0);
    //     assert_eq!(test_server.persistent_state.voted_for, None);
    //     assert_eq!(test_server.persistent_state.log.len(), 0);
    //     assert_eq!(test_server.persistent_state.log.capacity(), 4096);
    //     assert_eq!(test_server.volatile_state.commit_index, 0);
    //     assert_eq!(test_server.volatile_state.last_applied, 0);
    //     Ok(())
    // }
}
