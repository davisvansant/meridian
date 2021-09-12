use tokio::sync::broadcast::Sender;
use tokio::time::{sleep, timeout, Duration};

use crate::server::candidate::Candidate;
use crate::server::follower::Follower;
use crate::server::leader::Leader;
use crate::Actions;

pub mod candidate;
pub mod follower;
pub mod leader;

#[derive(Debug, PartialEq)]
pub enum ServerState {
    Follower,
    Candidate,
    Leader,
}

pub struct Server {
    pub server_state: ServerState,
    receive_actions: Sender<Actions>,
    send_actions: Sender<Actions>,
}

impl Server {
    pub async fn init(
        receive_actions: Sender<Actions>,
        send_actions: Sender<Actions>,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        let server_state = ServerState::Follower;

        Ok(Server {
            server_state,
            receive_actions,
            send_actions,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while self.server_state == ServerState::Follower {
            println!("doing follwer stuff!");

            if let Err(error) = self.follower().await {
                println!("Something went wrong with the follower - {:?}", error);
            }

            println!("transitioning to candidate...");
        }

        while self.server_state == ServerState::Candidate {
            println!("doing candidate stuff!");

            if let Err(error) = self.candidate().await {
                println!("something went wrong with the candidate {:?}", error);
            }

            println!("transitioning to leader...");
        }

        while self.server_state == ServerState::Leader {
            sleep(Duration::from_secs(10)).await;
            println!("Leader!");

            if let Err(error) = self.leader().await {
                println!("the leader had an error - {:?}", error);
            }
        }

        Ok(())
    }

    pub async fn follower(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let follower = Follower::init().await?;

        if timeout(follower.election_timeout, follower.election_timeout())
            .await
            .is_ok()
        {
            self.server_state = ServerState::Candidate;
        }

        Ok(())
    }

    pub async fn candidate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut receiver = self.receive_actions.subscribe();
        let candidate_id = String::from("some_candidate_id");
        let candidate = Candidate::init().await?;

        if let Err(error) = self.send_actions.send(Actions::Candidate(candidate_id)) {
            println!("error sending candidate action - {:?}", error);
        }

        if let Ok(Actions::RequestVoteRequest(request)) = receiver.recv().await {
            println!("sending vote request {:?}", &request);

            let start_election = candidate.start_election(request);

            if timeout(candidate.election_timeout, start_election)
                .await
                .is_ok()
            {
                self.server_state = ServerState::Leader;
            }
        }

        Ok(())
    }

    pub async fn leader(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let leader = Leader::init().await?;
        // let request = self
        //     .build_append_entires_request(&leader.volatile_state)
        //     .await?;
        //
        // leader.send_heartbeat(request).await?;

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
