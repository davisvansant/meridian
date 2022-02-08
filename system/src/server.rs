use tokio::signal::unix::{signal, SignalKind};
// use tokio::sync::{broadcast, mpsc, oneshot};
// use tokio::sync::broadcast;
// use tokio::time::{sleep, timeout, timeout_at, Duration, Instant};
use tokio::time::{sleep, Duration};

use crate::server::candidate::Candidate;
use crate::server::follower::Follower;
use crate::server::leader::Leader;

pub mod candidate;
pub mod follower;
pub mod leader;
mod preflight;

use crate::channel::{CandidateReceiver, CandidateSender, CandidateTransition};
use crate::channel::{ClientSender, MembershipSender, StateSender};
use crate::channel::{LeaderReceiver, LeaderSender};
// use crate::channel::{
//     SendServerShutdown, ServerReceiver, ServerSender, ServerShutdown, ServerState,
// };

pub enum ServerState {
    Candidate,
    Follower,
    Leader,
    Preflight,
    Shutdown,
}

pub struct Server {
    pub server_state: ServerState,
    client: ClientSender,
    membership: MembershipSender,
    // receiver: ServerReceiver,
    state: StateSender,
    // tx: ServerSender,
    // rx: ServerReceiver,
    candidate_sender: CandidateSender,
    candidate_receiver: CandidateReceiver,
    heartbeat: LeaderReceiver,
    // shutdown: ServerShutdown,
}

impl Server {
    pub async fn init(
        client: ClientSender,
        membership: MembershipSender,
        // receiver: ServerReceiver,
        state: StateSender,
        // tx: ServerSender,
        // rx: ServerReceiver,
        candidate_sender: CandidateSender,
        candidate_receiver: CandidateReceiver,
        heartbeat: LeaderReceiver,
        // shutdown: ServerShutdown,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        let server_state = ServerState::Follower;

        Ok(Server {
            server_state,
            client,
            membership,
            // receiver,
            state,
            // tx,
            // rx,
            candidate_sender,
            candidate_receiver,
            heartbeat,
            // shutdown,
        })
    }

    pub async fn run(
        &mut self,
        test_heartbeat: LeaderSender,
        candidate_receiver: CandidateSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sleep(Duration::from_secs(5)).await;

        let mut stream = signal(SignalKind::interrupt())?;

        loop {
            tokio::select! {
                biased;
                _ = stream.recv() => {
                    println!("shutting down system server...");

                    self.server_state = ServerState::Shutdown;

                    break
                }
                _ = self.server_state() => {
                    println!("why of why");
                }
            }
        }

        Ok(())
    }

    async fn server_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.server_state {
            ServerState::Preflight => {
                println!("running preflight tasks...");

                preflight::run(&self.client, &self.membership).await?;

                self.server_state = ServerState::Follower;

                Ok(())
            }
            ServerState::Follower => {
                println!("server > follower!");

                let mut follower = Follower::init().await?;
                follower.run(&mut self.heartbeat).await?;

                self.server_state = ServerState::Candidate;

                Ok(())
            }
            ServerState::Candidate => {
                println!("server > candidate!");

                let mut candidate = Candidate::init().await?;

                match candidate
                    // .run(&self.client, &mut self.candidate_receiver, &self.tx)
                    .run(&self.client, &mut self.candidate_receiver)
                    .await
                {
                    Ok(CandidateTransition::Follower) => {
                        self.server_state = ServerState::Follower;
                    }
                    Ok(CandidateTransition::Leader) => {
                        self.server_state = ServerState::Leader;
                    }
                    Err(error) => {
                        println!("candidate error -> {:?}", error);

                        self.server_state = ServerState::Candidate;
                    }
                }

                Ok(())
            }
            ServerState::Leader => {
                println!("server > leader!");

                let mut leader = Leader::init().await?;
                leader.run(&self.client, &self.state).await?;

                self.server_state = ServerState::Candidate;

                Ok(())
            }
            ServerState::Shutdown => {
                println!("server > shutdown...");

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::CandidateTransition;
    use crate::channel::Leader;
    // use crate::channel::ServerState;
    use crate::channel::{ClientRequest, ClientResponse};
    use crate::channel::{MembershipRequest, MembershipResponse};
    use crate::channel::{StateRequest, StateResponse};
    use tokio::sync::{broadcast, mpsc, oneshot};

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_client_sender, _test_client_receiver) =
            mpsc::channel::<(ClientRequest, oneshot::Sender<ClientResponse>)>(64);
        let (test_membership_sender, _test_membership_receiver) =
            mpsc::channel::<(MembershipRequest, oneshot::Sender<MembershipResponse>)>(64);
        let (test_state_sender, _test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);
        // let (test_transition_sender, test_transition_receiver) =
        //     broadcast::channel::<ServerState>(64);
        let (test_transition_sender, test_transition_receiver) = mpsc::channel::<ServerState>(64);
        // let (test_candidate_sender, test_candidate_receiver) =
        //     mpsc::channel::<CandidateTransition>(64);
        let (test_candidate_sender, test_candidate_receiver) =
            broadcast::channel::<CandidateTransition>(64);
        // let (test_leader_sender, test_leader_receiver) = mpsc::channel::<Leader>(64);
        let (test_leader_sender, test_leader_receiver) = broadcast::channel::<Leader>(64);
        let (test_shutdown_sender, test_shutdown_receiver) = mpsc::channel::<()>(1);

        let test_server = Server::init(
            test_client_sender,
            test_membership_sender,
            test_state_sender,
            // test_transition_sender,
            // test_transition_receiver,
            test_candidate_sender,
            test_candidate_receiver,
            test_leader_receiver,
            // test_shutdown_receiver,
        )
        .await?;

        assert!(!test_server.client.is_closed());
        assert!(!test_server.membership.is_closed());
        assert!(!test_server.state.is_closed());
        // assert_eq!(test_server.tx.receiver_count(), 1);
        // assert!(!test_server.candidate_sender.is_closed());
        // assert!(!test_leader_sender.is_closed());
        // assert!(!test_shutdown_sender.is_closed());

        Ok(())
    }
}
