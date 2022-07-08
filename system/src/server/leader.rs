use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

use crate::channel::membership::MembershipChannel;
use crate::channel::server_state::leader::EnterState;
use crate::channel::server_state::shutdown::Shutdown;
use crate::channel::server_state::ServerStateChannel;
use crate::channel::state::StateChannel;
use crate::rpc;
use crate::{error, info};

pub struct Leader {
    enter_state: EnterState,
    exit_state: ServerStateChannel,
    shutdown: Shutdown,
    membership: MembershipChannel,
    state: StateChannel,
}

impl Leader {
    pub async fn init(
        enter_state: EnterState,
        exit_state: ServerStateChannel,
        shutdown: Shutdown,
        membership: MembershipChannel,
        state: StateChannel,
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

                    self.state.init_leader().await?;

                    loop {
                        if let Err(error) = self.heartbeat(&self.membership, &self.state).await {
                            error!("leader heartbeat error -> {:?}", error);

                            break;
                        }
                    }

                    self.exit_state.shutdown().await?;

                    break;
                }
            }
        }

        Ok(())
    }

    async fn heartbeat(
        &self,
        membership: &MembershipChannel,
        state: &StateChannel,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sleep(Duration::from_secs(15)).await;

        let node = membership.node().await?;
        let arguments = state.heartbeat(node.id.to_string()).await?;
        let cluster_members = membership.cluster_members().await?;

        for follower in cluster_members {
            let socket_address = follower.build_address(follower.cluster_port).await;

            match self.invoke(socket_address, arguments.to_owned()).await {
                Ok(()) => info!("heartbeat sent!"),
                Err(error) => {
                    error!("append entries heartbeart -> {:?}", error);

                    continue;
                }
            }
        }

        Ok(())
    }

    async fn invoke(
        &self,
        socket_address: SocketAddr,
        arguments: rpc::append_entries::AppendEntriesArguments,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "sending heartbeat to socket address -> {:?}",
            &socket_address,
        );

        let mut client = rpc::Client::init(socket_address).await;
        let results = client.send_append_entries(arguments).await?;

        info!("append entries results -> {:?}", &results);

        self.state.append_entries_results(results).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::server_state::leader::LeaderState;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (_test_leader_sender, test_leader_receiver) = LeaderState::init().await;
        let (test_server_transition_state_sender, _test_server_transition_state_receiver) =
            ServerStateChannel::init().await;
        let test_shutdown_sender = Shutdown::init();
        let (test_membership_sender, _test_membership_receiver) = MembershipChannel::init().await;
        let (test_state_sender, _test_state_receiver) = StateChannel::init().await;

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
