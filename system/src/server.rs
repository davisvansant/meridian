use tokio::time::{sleep, timeout, timeout_at, Duration, Instant};

use crate::server::candidate::Candidate;
use crate::server::follower::Follower;
use crate::server::leader::Leader;

pub mod candidate;
pub mod follower;
pub mod leader;

use crate::channels::ChannelMembershipReceiveAction;
use crate::channels::ChannelMembershipSendServerAction;
use crate::channels::ChannelStateReceiveAction;
use crate::channels::ChannelStateSendServerAction;

use crate::channels::MembershipReceiveAction;
use crate::channels::MembershipSendServerAction;
use crate::channels::StateReceiveAction;
use crate::channels::StateSendServerAction;

#[derive(Debug, PartialEq)]
pub enum ServerState {
    Follower,
    Candidate,
    Leader,
}

pub struct Server {
    pub server_state: ServerState,
    receive_actions: ChannelStateSendServerAction,
    send_actions: ChannelStateReceiveAction,
    membership_send_action: ChannelMembershipReceiveAction,
    membership_receive_action: ChannelMembershipSendServerAction,
}

impl Server {
    pub async fn init(
        receive_actions: ChannelStateSendServerAction,
        send_actions: ChannelStateReceiveAction,
        membership_send_action: ChannelMembershipReceiveAction,
        membership_receive_action: ChannelMembershipSendServerAction,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        let server_state = ServerState::Follower;

        Ok(Server {
            server_state,
            receive_actions,
            send_actions,
            membership_send_action,
            membership_receive_action,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut receiver = self.receive_actions.subscribe();

        tokio::spawn(async move {
            while let Ok(action) = receiver.recv().await {
                match action {
                    StateSendServerAction::AppendEntriesRequest(_) => {
                        println!("received append entries");
                    }
                    StateSendServerAction::Follower => println!("follower"),
                    StateSendServerAction::RequestVoteRequest(request) => println!("{:?}", request),
                }
            }
        });

        loop {
            match self.server_state {
                ServerState::Follower => {
                    println!("doing follwer stuff!");

                    if let Err(error) = self.follower().await {
                        println!("Something went wrong with the follower - {:?}", error);
                        break;
                    }

                    println!("transitioning to candidate...");
                }
                ServerState::Candidate => {
                    // sleep(Duration::from_secs(10)).await;
                    println!("doing candidate stuff!");

                    if let Err(error) = self.candidate().await {
                        println!("something went wrong with the candidate {:?}", error);
                        break;
                    }

                    // println!("transitioning to leader...");
                }
                ServerState::Leader => {
                    sleep(Duration::from_secs(10)).await;
                    println!("Leader!");

                    if let Err(error) = self.leader().await {
                        println!("the leader had an error - {:?}", error);
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn follower(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut receiver = self.receive_actions.subscribe();
        let follower = Follower::init().await?;

        while let Ok(result) =
            timeout_at(Instant::now() + follower.election_timeout, receiver.recv()).await
        {
            if let Ok(StateSendServerAction::Follower) = result {
                println!("receiving heartbeat...");
            }
        }

        println!("timeout ending...starting election");
        self.server_state = ServerState::Candidate;

        Ok(())
    }

    pub async fn candidate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut receiver = self.receive_actions.subscribe();
        let mut membership_receiver = self.membership_receive_action.subscribe();

        self.membership_send_action
            .send(MembershipReceiveAction::Node)?;

        if let Ok(MembershipSendServerAction::NodeResponse(node)) = membership_receiver.recv().await
        {
            self.send_actions
                .send(StateReceiveAction::Candidate(node.id.to_string()))?;
        };

        let candidate = Candidate::init().await?;

        while let Ok(action) =
            timeout_at(Instant::now() + candidate.election_timeout, receiver.recv()).await
        {
            match action {
                Ok(StateSendServerAction::Follower) => {
                    println!("received heartbeat...stepping down");
                    self.server_state = ServerState::Follower;
                    break;
                }
                Ok(StateSendServerAction::RequestVoteRequest(request)) => {
                    println!("sending receive request to cluster members - {:?}", request);

                    self.membership_send_action
                        .send(MembershipReceiveAction::Members)?;

                    if let Ok(MembershipSendServerAction::MembersResponse(members)) =
                        membership_receiver.recv().await
                    {
                        println!("members ! {:?}", &members);

                        let mut nodes = Vec::with_capacity(members.len());

                        for member in &members {
                            let address = member.address;
                            let port = member.cluster_port;
                            let mut node = String::with_capacity(20);

                            node.push_str("http://");
                            node.push_str(&address.to_string());
                            node.push(':');
                            node.push_str(&port.to_string());
                            node.shrink_to_fit();

                            nodes.push(node)
                        }

                        if nodes.is_empty() {
                            self.server_state = ServerState::Leader;
                        } else {
                            let mut vote_true = Vec::with_capacity(3);
                            let mut vote_false = Vec::with_capacity(3);

                            for node in nodes {
                                println!("sending request to node - {:?}", &node);
                                if candidate.start_election(request.clone(), node).await? {
                                    vote_true.push(1);
                                } else {
                                    vote_false.push(1);
                                }
                            }
                            if vote_true >= vote_false {
                                self.server_state = ServerState::Leader;
                            } else {
                                self.server_state = ServerState::Follower;
                            }
                        }
                    }
                }
                _ => println!("cannot do anyhting with other requests"),
            }
        }

        Ok(())
    }

    pub async fn leader(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut receiver = self.receive_actions.subscribe();
        let mut membership_receiver = self.membership_receive_action.subscribe();

        let leader = Leader::init().await?;

        self.membership_send_action
            .send(MembershipReceiveAction::Node)?;

        if let Ok(MembershipSendServerAction::NodeResponse(node)) = membership_receiver.recv().await
        {
            println!("server uuid - {:?}", &node);

            self.send_actions
                .send(StateReceiveAction::Leader(node.id.to_string()))?;

            if let Ok(StateSendServerAction::AppendEntriesRequest(request)) = receiver.recv().await
            {
                self.membership_send_action
                    .send(MembershipReceiveAction::Members)?;

                if let Ok(members) = membership_receiver.recv().await {
                    println!("{:?}", &members);
                    println!("sending heartbeat ... {:?}", &request);

                    leader.send_heartbeat(request).await?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn server_state_candidate() -> Result<(), Box<dyn std::error::Error>> {
        let test_server_state = ServerState::Candidate;
        assert_eq!(test_server_state, ServerState::Candidate);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn server_state_follower() -> Result<(), Box<dyn std::error::Error>> {
        let test_server_state = ServerState::Follower;
        assert_eq!(test_server_state, ServerState::Follower);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn server_state_leader() -> Result<(), Box<dyn std::error::Error>> {
        let test_server_state = ServerState::Leader;
        assert_eq!(test_server_state, ServerState::Leader);
        Ok(())
    }

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
