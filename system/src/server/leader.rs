use tokio::time::{sleep, Duration};

use crate::channel::{membership, state, transition};
use crate::rpc;
use crate::{error, info};

pub struct Leader {
    enter_state: transition::LeaderReceiver,
    exit_state: transition::ServerStateSender,
    shutdown: transition::ShutdownSender,
    membership: membership::MembershipSender,
    state: state::StateSender,
}

impl Leader {
    pub async fn init(
        enter_state: transition::LeaderReceiver,
        exit_state: transition::ServerStateSender,
        shutdown: transition::ShutdownSender,
        membership: membership::MembershipSender,
        state: state::StateSender,
    ) -> Result<Leader, Box<dyn std::error::Error>> {
        Ok(Leader {
            enter_state,
            exit_state,
            shutdown,
            membership,
            state,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut shutdown = self.shutdown.subscribe();

        loop {
            tokio::select! {
                biased;
                _ = shutdown.recv() => {
                    info!("shutting down leader heartbeat...");

                    break
                }
                Some(run) = self.enter_state.recv() => {
                    info!("running leader! -> {:?}", run);

                    state::leader(&self.state).await?;

                    loop {
                        if let Err(error) = self.heartbeat(&self.membership, &self.state).await {
                            error!("leader heartbeat error -> {:?}", error);

                            break;
                        }
                    }

                    self.exit_state
                        .send(transition::ServerState::Shutdown)
                        .await?;

                    break;
                }
            }
        }

        Ok(())
    }

    async fn heartbeat(
        &self,
        membership: &membership::MembershipSender,
        state: &state::StateSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sleep(Duration::from_secs(15)).await;

        let node = membership::node(membership).await?;
        let append_entries_arguments = state::heartbeat(state, node.id.to_string()).await?;
        let cluster_members = membership::cluster_members(membership).await?;

        for follower in cluster_members {
            let socket_address = follower.build_address(follower.cluster_port).await;

            info!(
                "sending heartbeat to socket address -> {:?}",
                &socket_address,
            );

            let mut client = rpc::Client::init(socket_address).await;

            match client
                .send_append_entries(append_entries_arguments.to_owned())
                .await
            {
                Ok(append_entries_results) => info!("result -> {:?}", append_entries_results),
                Err(error) => error!("result -> {:?}", error),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_leader_sender, test_leader_receiver) =
            crate::channel::transition::Leader::build().await;
        let (test_server_transition_state_sender, _test_server_transition_state_receiver) =
            crate::channel::transition::ServerState::build().await;
        let test_shutdown_sender = crate::channel::transition::Shutdown::build().await;
        let (test_membership_sender, _test_membership_receiver) =
            crate::channel::membership::build().await;
        let (test_state_sender, _test_state_receiver) = crate::channel::state::build().await;

        let test_leader = Leader::init(
            test_leader_receiver,
            test_server_transition_state_sender,
            test_shutdown_sender,
            test_membership_sender,
            test_state_sender,
        )
        .await;

        assert!(test_leader.is_ok());

        Ok(())
    }
}
