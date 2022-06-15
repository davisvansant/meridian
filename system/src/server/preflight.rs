use tokio::time::{sleep, Duration};

use crate::channel::membership::MembershipSender;
use crate::channel::membership::{launch_failure_detector, static_join};
use crate::{error, info, warn};

pub struct Preflight {
    enter_state: crate::channel::transition::PreflightReceiver,
    exit_state: crate::channel::transition::ServerStateSender,
    // shutdown: crate::channel::shutdown::ShutdownSender,
    shutdown: crate::channel::transition::ShutdownSender,
    membership: crate::channel::membership::MembershipSender,
}

impl Preflight {
    pub async fn init(
        enter_state: crate::channel::transition::PreflightReceiver,
        exit_state: crate::channel::transition::ServerStateSender,
        // shutdown: crate::channel::shutdown::ShutdownSender,
        shutdown: crate::channel::transition::ShutdownSender,
        membership: crate::channel::membership::MembershipSender,
    ) -> Result<Preflight, Box<dyn std::error::Error>> {
        Ok(Preflight {
            enter_state,
            exit_state,
            shutdown,
            membership,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut shutdown = self.shutdown.subscribe();

        tokio::select! {
            biased;
            _ = shutdown.recv() => {
                info!("shutting down preflight task...");

                // break
            }
            Some(run) = self.enter_state.recv() => {
                info!("preflight! -> {:?}", run);

                self.task().await?;
            }
        }

        Ok(())
    }

    async fn task(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut errors = 0;

        while errors <= 4 {
            let (active, expected) = static_join(&self.membership).await?;

            if active == expected {
                launch_failure_detector(&self.membership).await?;

                self.exit_state
                    .send(crate::channel::transition::ServerState::Follower)
                    .await?;

                break;
            } else {
                error!("expecting {} peers | {} active peers", &expected, &active);

                errors += 1;

                match errors {
                    1 => {
                        error!("attempting preflight again... (1/3)");
                        sleep(Duration::from_secs(10)).await;
                    }
                    2 => {
                        error!("attempting preflight again... (2/3)");
                        sleep(Duration::from_secs(20)).await;
                    }
                    3 => {
                        error!("attempting preflight again... (3/3)");
                        sleep(Duration::from_secs(30)).await;
                    }
                    4 => {
                        error!("preflight tasks failed...shutting down...");

                        self.exit_state
                            .send(crate::channel::transition::ServerState::Shutdown)
                            .await?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
