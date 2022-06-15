// use crate::channel::rpc_client::{send_heartbeat, RpcClientSender};
use crate::channel::membership::{cluster_members, node};
// use crate::channel::shutdown::ShutdownReceiver;
use crate::channel::state::StateSender;
use crate::channel::state::{heartbeat, leader};
use crate::{error, info};
use tokio::time::{sleep, Duration};

pub struct Leader {
    // shutdown: ShutdownReceiver,
    enter_state: crate::channel::transition::LeaderReceiver,
    exit_state: crate::channel::transition::ServerStateSender,
    // shutdown: crate::channel::shutdown::ShutdownSender,
    shutdown: crate::channel::transition::ShutdownSender,
    membership: crate::channel::membership::MembershipSender,
    state: crate::channel::state::StateSender,
}

impl Leader {
    pub async fn init(
        enter_state: crate::channel::transition::LeaderReceiver,
        exit_state: crate::channel::transition::ServerStateSender,
        // shutdown: crate::channel::shutdown::ShutdownSender,
        shutdown: crate::channel::transition::ShutdownSender,
        membership: crate::channel::membership::MembershipSender,
        state: crate::channel::state::StateSender,
        // shutdown: ShutdownReceiver,
    ) -> Result<Leader, Box<dyn std::error::Error>> {
        Ok(Leader {
            enter_state,
            exit_state,
            shutdown,
            membership,
            state,
        })
    }

    pub async fn run(
        &mut self,
        // rpc_client: &RpcClientSender,
        // state: &StateSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut shutdown = self.shutdown.subscribe();

        // leader(state).await?;

        // loop {
        //     tokio::select! {
        //         biased;
        //         _ = self.shutdown.recv() => {
        //             info!("shutting down leader heartbeat...");

        //             break
        //         }
        //         _ = Leader::heartbeat(rpc_client) => {}
        //     }
        // }
        loop {
            tokio::select! {
                biased;
                _ = shutdown.recv() => {
                    info!("shutting down leader heartbeat...");

                    break
                }
                Some(run) = self.enter_state.recv() => {
                    info!("running leader! -> {:?}", run);
                    leader(&self.state).await?;

                    loop {
                        // self.heartbeat(&self.membership, &self.state).await?;
                        if let Err(error) = self.heartbeat(&self.membership, &self.state).await {
                            error!("leader heartbeat error -> {:?}", error);

                            break;
                        }
                    }

                    self.exit_state
                        .send(crate::channel::transition::ServerState::Shutdown)
                        .await?;

                    break;
                }
            }
        }

        Ok(())
    }

    // async fn heartbeat(rpc_client: &RpcClientSender) -> Result<(), Box<dyn std::error::Error>> {
    //     sleep(Duration::from_secs(15)).await;

    //     send_heartbeat(rpc_client).await?;

    //     Ok(())
    // }
    async fn heartbeat(
        &self,
        membership: &crate::channel::membership::MembershipSender,
        state: &crate::channel::state::StateSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sleep(Duration::from_secs(15)).await;

        let node = node(membership).await?;
        let append_entries_arguments = heartbeat(state, node.id.to_string()).await?;

        let cluster_members = cluster_members(membership).await?;

        for follower in cluster_members {
            let socket_address = follower.build_address(follower.cluster_port).await;

            let mut client = crate::rpc::Client::init(socket_address).await;

            let append_entries_results = client
                .send_append_entries(append_entries_arguments.to_owned())
                .await?;

            info!("append entries results! -> {:?}", append_entries_results);
        }

        // info!("send a heartbeat!");

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
        // let test_shutdown_receiver = test_shutdown_sender.subscribe();

        // drop(test_shutdown_sender);

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
