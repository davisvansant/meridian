use rand::{thread_rng, Rng};
use std::net::SocketAddr;
use tokio::time::{timeout_at, Duration, Instant};

use crate::channel::{membership, server, state, transition};
use crate::rpc;
use crate::{error, info, warn};

pub struct Candidate {
    election_timeout: Duration,
    enter_state: transition::CandidateReceiver,
    exit_state: transition::ServerStateSender,
    shutdown: transition::ShutdownSender,
    heartbeat: server::LeaderSender,
    membership: membership::MembershipSender,
    state: state::StateSender,
}

impl Candidate {
    pub async fn init(
        enter_state: transition::CandidateReceiver,
        exit_state: transition::ServerStateSender,
        shutdown: transition::ShutdownSender,
        heartbeat: server::LeaderSender,
        membership: membership::MembershipSender,
        state: state::StateSender,
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

        let (election_result_sender, mut election_result_receiver) =
            server::ElectionResult::build().await;

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
                        Some(server::ElectionResult::Follower) => {
                            info!("received heartbeat...stepping down");

                            heartbeat_handle.abort();
                            start_election_handle.abort();

                            self.exit_state.send(transition::ServerState::Follower).await?;
                        }
                        Some(crate::channel::server::ElectionResult::Leader) => {
                            info!("transitioning server to leader...");

                            heartbeat_handle.abort();

                            self.exit_state.send(transition::ServerState::Leader).await?;
                        }
                        None => {
                            warn!("candidate election timeout lapsed...trying again...");

                            heartbeat_handle.abort();
                            start_election_handle.abort();

                            self.exit_state.send(transition::ServerState::Candidate).await?;
                        }
                    },
                    Err(error) => {
                        error!("{:?}", error);

                        warn!("trying candidate election again");

                        heartbeat_handle.abort();
                        start_election_handle.abort();

                        self.exit_state.send(transition::ServerState::Candidate).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn receive_heartbeat(
        heartbeat: server::LeaderSender,
        election_result: server::ElectionResultSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut heartbeat = heartbeat.subscribe();

        heartbeat.recv().await?;

        election_result
            .send(server::ElectionResult::Follower)
            .await?;

        Ok(())
    }

    async fn start_election(
        membership: &membership::MembershipSender,
        state: &state::StateSender,
        election_result: &server::ElectionResultSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut leader_votes = Vec::with_capacity(5);

        let node = membership::node(membership).await?;
        let request_vote_arguments = state::candidate(state, node.id.to_string()).await?;
        let peers = membership::cluster_members(membership).await?;

        let quorum = peers.len() / 2 + 1;

        if peers.is_empty() {
            election_result.send(server::ElectionResult::Leader).await?;
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
                election_result.send(server::ElectionResult::Leader).await?;
            } else {
                election_result
                    .send(server::ElectionResult::Follower)
                    .await?;
            }
        }

        Ok(())
    }

    async fn request_vote(
        socket_address: SocketAddr,
        arguments: rpc::request_vote::RequestVoteArguments,
        state: &state::StateSender,
        election_result: &server::ElectionResultSender,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        info!(
            "sending election request to socket address -> {:?}",
            &socket_address,
        );

        let mut client = rpc::Client::init(socket_address).await;
        let request_vote_results = client.send_request_vote(arguments).await?;

        info!("request vote results -> {:?}", &request_vote_results);

        let vote_granted = request_vote_results.vote_granted;
        let transition = state::request_vote_results(state, request_vote_results).await?;

        match transition {
            true => {
                info!("peer at heigher term, transition to follower...");

                election_result
                    .send(server::ElectionResult::Follower)
                    .await?;

                Ok(false)
            }
            false => {
                info!("continuing vote results...");

                Ok(vote_granted)
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test(flavor = "multi_thread")]
//     async fn init() -> Result<(), Box<dyn std::error::Error>> {
//         let test_candidate = Candidate::init().await?;
//         assert_eq!(test_candidate.election_timeout.as_millis(), 15000);
//         Ok(())
//     }
// }
