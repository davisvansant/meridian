use crate::channel::rpc_client::{send_heartbeat, RpcClientSender};
use crate::channel::shutdown::ShutdownReceiver;
use crate::channel::state::leader;
use crate::channel::state::StateSender;
use crate::info;

pub struct Leader {
    shutdown: ShutdownReceiver,
}

impl Leader {
    pub async fn init(shutdown: ShutdownReceiver) -> Result<Leader, Box<dyn std::error::Error>> {
        Ok(Leader { shutdown })
    }

    pub async fn run(
        &mut self,
        rpc_client: &RpcClientSender,
        state: &StateSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        leader(state).await?;

        loop {
            tokio::select! {
                biased;
                _ = self.shutdown.recv() => {
                    info!("shutting down leader heartbeat...");

                    break
                }
                _ = send_heartbeat(rpc_client) => {}
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
        let test_shutdown_sender = crate::channel::shutdown::build().await;
        let test_shutdown_receiver = test_shutdown_sender.subscribe();

        drop(test_shutdown_sender);

        let test_leader = Leader::init(test_shutdown_receiver).await;

        assert!(test_leader.is_ok());

        Ok(())
    }
}
