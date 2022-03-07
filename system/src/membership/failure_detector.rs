use tokio::time::{sleep, Duration};

use crate::channel::MembershipListSender;
use crate::channel::ShutdownReceiver;
use crate::channel::{get_alive, insert_confirmed, remove_alive, remove_suspected};
use crate::channel::{MembershipFailureDetectorReceiver, MembershipFailureDetectorRequest};

pub struct FailureDectector {
    protocol_period: Duration,
    list_sender: MembershipListSender,
    receiver: MembershipFailureDetectorReceiver,
}

impl FailureDectector {
    pub async fn init(
        list_sender: MembershipListSender,
        receiver: MembershipFailureDetectorReceiver,
    ) -> FailureDectector {
        let protocol_period = Duration::from_secs(10);

        FailureDectector {
            protocol_period,
            list_sender,
            receiver,
        }
    }

    pub async fn run(
        &mut self,
        shutdown: &mut ShutdownReceiver,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(request) = self.receiver.recv().await {
            match request {
                MembershipFailureDetectorRequest::Launch => {
                    println!("launching membership failure detector!");
                }
            }
        }

        loop {
            tokio::select! {
                biased;
                _ = shutdown.recv() => {
                    println!("shutting down failure dectector...");

                    self.receiver.close();

                    break
                }
                result = self.probe() => {
                    match result {
                        Ok(()) => println!("probe complete!"),
                        Err(error) => println!("probe failed with error -> {:?}", error),
                    }
                }
            }
        }

        Ok(())
    }

    async fn probe(&self) -> Result<(), Box<dyn std::error::Error>> {
        sleep(self.protocol_period).await;

        // let alive = get_alive(&self.list_sender).await?;

        // for node in alive {
        //     // placeholder...
        //     remove_alive(&self.list_sender, node).await?;
        //     remove_suspected(&self.list_sender, node).await?;
        //     insert_confirmed(&self.list_sender, node).await?;
        // }

        Ok(())
    }
}
