use tokio::signal::unix::{signal, SignalKind};

use crate::channel::{leader, send_heartbeat};
// use crate::channel::{ClientSender, StateSender};
use crate::channel::{RpcClientSender, StateSender};

pub struct Leader {}

impl Leader {
    pub async fn init() -> Result<Leader, Box<dyn std::error::Error>> {
        Ok(Leader {})
    }

    pub async fn run(
        &mut self,
        rpc_client: &RpcClientSender,
        state: &StateSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        leader(state).await?;

        let mut interrupt = signal(SignalKind::interrupt())?;

        loop {
            tokio::select! {
                biased;
                _ = interrupt.recv() => {
                    println!("shutting down leader heartbeat...");

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
        let test_leader = Leader::init().await;

        assert!(test_leader.is_ok());

        Ok(())
    }
}
