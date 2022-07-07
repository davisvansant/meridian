use rand::{thread_rng, Rng};
use std::net::SocketAddr;
use tokio::time::{timeout_at, Duration, Instant};

use crate::channel::membership::MembershipChannel;
use crate::channel::server::{ElectionResult, ElectionResultSender, LeaderHeartbeatSender};
use crate::channel::state::StateChannel;
use crate::channel::transition::{
    CandidateStateReceiver, ShutdownSender, Transition, TransitionSender,
};
use crate::rpc;
use crate::{error, info, warn};

pub struct Candidate {
    election_timeout: Duration,
    enter_state: CandidateStateReceiver,
    exit_state: TransitionSender,
    shutdown: ShutdownSender,
    heartbeat: LeaderHeartbeatSender,
    membership: MembershipChannel,
    state: StateChannel,
}

impl Candidate {
    pub async fn init(
        enter_state: CandidateStateReceiver,
        exit_state: TransitionSender,
        shutdown: ShutdownSender,
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
        let mut shutdown = self.shutdown.subscribe();

        let start_election_membership_sender = self.membership.to_owned();

        let (election_result_sender, mut election_result_receiver) = ElectionResult::build().await;

        let heartbeat = self.heartbeat.to_owned();
        let receive_heartbeat_election_result_sender = election_result_sender.to_owned();

        let heartbeat_handle = tokio::spawn(async move {
            if let Err(error) =
                Self::receive_heartbeat(heartbeat, receive_heartbeat_election_result_sender).await
            {
                error!("{:?}", error);
            }
        });

        let start_election_state = self.state.to_owned();

        tokio::select! {
            biased;

             _ = shutdown.recv() => {
                info!("shutting down candidate");
            }

            Some(run) = self.enter_state.recv() => {
                info!("candidate -> {:?} | starting election!", run);

                let start_election_handle = tokio::spawn(async move {
                    if let Err(error) = Self::start_election(
                        &start_election_membership_sender.to_owned(),
                        &start_election_state.to_owned(),
                        &election_result_sender.to_owned(),
                    )
                    .await
                    {
                        error!("{:?}", error);
                    }
                });

                match timeout_at(Instant::now() + self.election_timeout, election_result_receiver.recv()).await {
                    Ok(election_result) => match election_result {
                        Some(ElectionResult::Follower) => {
                            info!("received heartbeat...stepping down");

                            heartbeat_handle.abort();
                            start_election_handle.abort();

                            self.exit_state.send(Transition::FollowerState).await?;
                        }
                        Some(ElectionResult::Leader) => {
                            info!("transitioning server to leader...");

                            heartbeat_handle.abort();

                            self.exit_state.send(Transition::LeaderState).await?;
                        }
                        None => {
                            warn!("candidate election timeout lapsed...trying again...");

                            heartbeat_handle.abort();
                            start_election_handle.abort();

                            self.exit_state.send(Transition::CandidateState).await?;
                        }
                    },
                    Err(error) => {
                        error!("{:?}", error);

                        warn!("trying candidate election again");

                        heartbeat_handle.abort();
                        start_election_handle.abort();

                        self.exit_state.send(Transition::CandidateState).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn receive_heartbeat(
        heartbeat: LeaderHeartbeatSender,
        election_result: ElectionResultSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut heartbeat = heartbeat.subscribe();

        heartbeat.recv().await?;

        election_result.send(ElectionResult::Follower).await?;

        Ok(())
    }

    async fn start_election(
        membership: &MembershipChannel,
        state: &StateChannel,
        election_result: &ElectionResultSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut leader_votes = Vec::with_capacity(5);

        let node = membership.node().await?;
        let request_vote_arguments = state.candidate(node.id.to_string()).await?;
        let peers = membership.cluster_members().await?;

        let quorum = peers.len() / 2 + 1;

        if peers.is_empty() {
            election_result.send(ElectionResult::Leader).await?;
        } else {
            for peer in peers {
                let socket_address = peer.build_address(peer.cluster_port).await;

                match Self::request_vote(
                    socket_address,
                    request_vote_arguments.to_owned(),
                    state,
                    election_result,
                )
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
                election_result.send(ElectionResult::Leader).await?;
            } else {
                election_result.send(ElectionResult::Follower).await?;
            }
        }

        Ok(())
    }

    async fn request_vote(
        socket_address: SocketAddr,
        arguments: rpc::request_vote::RequestVoteArguments,
        state: &StateChannel,
        election_result: &ElectionResultSender,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        info!(
            "sending election request to socket address -> {:?}",
            &socket_address,
        );

        let mut client = rpc::Client::init(socket_address).await;
        let request_vote_results = client.send_request_vote(arguments).await?;

        info!("request vote results -> {:?}", &request_vote_results);

        let vote_granted = request_vote_results.vote_granted;
        let transition = state.request_vote_results(request_vote_results).await?;

        match transition {
            true => {
                info!("peer at heigher term, transition to follower...");

                election_result.send(ElectionResult::Follower).await?;

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
    use crate::channel::transition::{CandidateState, Shutdown};

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_transition, test_receive) = CandidateState::build().await;
        let (test_server_transition_state_sender, _test_server_transition_state_receiver) =
            Transition::build().await;
        let test_shutdown_signal = Shutdown::build().await;
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
        assert_eq!(test_transition.capacity(), 64);
        assert_eq!(test_candidate.exit_state.capacity(), 64);
        assert_eq!(test_candidate.shutdown.receiver_count(), 0);
        assert_eq!(test_candidate.heartbeat.receiver_count(), 0);

        Ok(())
    }
}
