use rand::{thread_rng, Rng};
use std::net::SocketAddr;
use tokio::time::{timeout, Duration};

use crate::channel::membership::MembershipChannel;
use crate::channel::server::{ElectionResult, ElectionResultSender, LeaderHeartbeatSender};
use crate::channel::server_state::candidate::EnterState;
use crate::channel::server_state::shutdown::Shutdown;
use crate::channel::server_state::ServerStateChannel;
use crate::channel::state::StateChannel;
use crate::rpc;
use crate::{error, info, warn};

pub struct Candidate {
    election_timeout: Duration,
    enter_state: EnterState,
    exit_state: ServerStateChannel,
    shutdown: Shutdown,
    heartbeat: LeaderHeartbeatSender,
    membership: MembershipChannel,
    state: StateChannel,
}

impl Candidate {
    pub async fn init(
        enter_state: EnterState,
        exit_state: ServerStateChannel,
        shutdown: Shutdown,
        heartbeat: LeaderHeartbeatSender,
        membership: MembershipChannel,
        state: StateChannel,
    ) -> Result<Candidate, Box<dyn std::error::Error>> {
        let mut rng = thread_rng();

        let election_timeout =
            rng.gen_range(Duration::from_millis(15000)..Duration::from_millis(30000));

        Ok(Candidate {
            election_timeout,
            enter_state,
            exit_state,
            shutdown,
            heartbeat,
            membership,
            state,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (election_result_sender, mut election_result) = ElectionResult::build().await;

        let heartbeat =
            Heartbeat::init(self.heartbeat.to_owned(), election_result_sender.to_owned()).await;

        let election = Election::init(
            self.membership.to_owned(),
            self.state.to_owned(),
            election_result_sender.to_owned(),
        )
        .await;

        let mut shutdown = self.shutdown.subscribe();

        tokio::select! {
            biased;

             _ = shutdown.recv() => {
                info!("shutting down candidate");
            }

            Some(run) = self.enter_state.recv() => {
                info!("candidate -> {:?} | starting election!", run);

                let heartbeat_handle = tokio::spawn(async move {
                    if let Err(error) = heartbeat.receive().await {
                        error!("receiving heartbeat -> {:?}", error);
                    }
                });

                let election_handle = tokio::spawn(async move {
                    if let Err(error) = election.start().await {
                        error!("start election error -> {:?}", error);
                    }
                });

                match timeout(self.election_timeout, election_result.recv()).await {
                    Ok(election_result) => match election_result {
                        Some(ElectionResult::Follower) => {
                            info!("received heartbeat...stepping down");

                            heartbeat_handle.abort();
                            election_handle.abort();

                            self.exit_state.follower().await?;
                        }
                        Some(ElectionResult::Leader) => {
                            info!("transitioning server to leader...");

                            heartbeat_handle.abort();

                            self.exit_state.leader().await?;
                        }
                        None => {
                            warn!("candidate election timeout lapsed...trying again...");

                            heartbeat_handle.abort();
                            election_handle.abort();

                            self.exit_state.candidate().await?;
                        }
                    },
                    Err(error) => {
                        error!("{:?}", error);

                        warn!("trying candidate election again");

                        heartbeat_handle.abort();
                        election_handle.abort();

                        self.exit_state.candidate().await?;
                    }
                }
            }
        }

        Ok(())
    }
}

struct Heartbeat {
    leader: LeaderHeartbeatSender,
    result: ElectionResultSender,
}

impl Heartbeat {
    async fn init(leader: LeaderHeartbeatSender, result: ElectionResultSender) -> Heartbeat {
        Heartbeat { leader, result }
    }

    async fn receive(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut heartbeat = self.leader.subscribe();

        heartbeat.recv().await?;

        self.result.send(ElectionResult::Follower).await?;

        Ok(())
    }
}

struct Election {
    membership: MembershipChannel,
    state: StateChannel,
    result: ElectionResultSender,
}

impl Election {
    async fn init(
        membership: MembershipChannel,
        state: StateChannel,
        result: ElectionResultSender,
    ) -> Election {
        Election {
            membership,
            state,
            result,
        }
    }
    async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut leader_votes = Vec::with_capacity(5);

        let node = self.membership.node().await?;
        let request_vote_arguments = self.state.candidate(node.id.to_string()).await?;
        let peers = self.membership.cluster_members().await?;

        let quorum = peers.len() / 2 + 1;

        if peers.is_empty() {
            self.result.send(ElectionResult::Leader).await?;
        } else {
            for peer in peers {
                let socket_address = peer.build_address(peer.cluster_port).await;

                match self
                    .request_vote(socket_address, request_vote_arguments.to_owned())
                    .await
                {
                    Ok(vote_granted) => {
                        if vote_granted {
                            leader_votes.push(1);
                        } else {
                            break;
                        }
                    }
                    Err(error) => {
                        error!("request vote results -> {:?}", error);

                        continue;
                    }
                }
            }

            info!(
                "election result -> leader votes {:?} | quorum {:?}",
                &leader_votes.len(),
                &quorum,
            );

            if leader_votes.len() >= quorum {
                self.result.send(ElectionResult::Leader).await?;
            } else {
                self.result.send(ElectionResult::Follower).await?;
            }
        }

        Ok(())
    }

    async fn request_vote(
        &self,
        socket_address: SocketAddr,
        arguments: rpc::request_vote::RequestVoteArguments,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        info!(
            "sending election request to socket address -> {:?}",
            &socket_address,
        );

        let mut client = rpc::Client::init(socket_address).await;
        let request_vote_results = client.send_request_vote(arguments).await?;

        info!("request vote results -> {:?}", &request_vote_results);

        let vote_granted = request_vote_results.vote_granted;
        let transition = self
            .state
            .request_vote_results(request_vote_results)
            .await?;

        match transition {
            true => {
                info!("peer at heigher term, transition to follower...");

                self.result.send(ElectionResult::Follower).await?;

                Ok(false)
            }
            false => {
                info!("continuing vote results...");

                Ok(vote_granted)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::server::Leader;
    use crate::channel::server_state::candidate::CandidateState;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_transition, test_receive) = CandidateState::init().await;

        let (test_server_transition_state_sender, _test_server_transition_state_receiver) =
            ServerStateChannel::init().await;
        let test_shutdown_signal = Shutdown::init();
        let test_leader_heartbeat_sender = Leader::build().await;
        let (test_membership_sender, _test_membership_receiver) = MembershipChannel::init().await;
        let (test_state_sender, _test_state_receiver) = StateChannel::init().await;

        let test_candidate = Candidate::init(
            test_receive,
            test_server_transition_state_sender,
            test_shutdown_signal,
            test_leader_heartbeat_sender,
            test_membership_sender,
            test_state_sender,
        )
        .await?;

        assert!(test_candidate.election_timeout.as_millis() >= 15000);
        assert!(test_candidate.election_timeout.as_millis() <= 30000);
        assert_eq!(test_candidate.heartbeat.receiver_count(), 0);

        Ok(())
    }
}
