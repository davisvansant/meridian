use tokio::time::{sleep, Duration};

use crate::channel::membership::MembershipChannel;
use crate::channel::server_state::preflight::EnterState;
use crate::channel::server_state::shutdown::Shutdown;
use crate::channel::server_state::ServerStateChannel;
use crate::{error, info};

pub struct Preflight {
    enter_state: EnterState,
    exit_state: ServerStateChannel,
    shutdown: Shutdown,
    membership: MembershipChannel,
}

impl Preflight {
    pub async fn init(
        enter_state: EnterState,
        exit_state: ServerStateChannel,
        shutdown: Shutdown,
        membership: MembershipChannel,
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
            let (active, expected) = self.membership.static_join().await?;

            if active == expected {
                self.membership.launch_failure_detector().await?;
                self.exit_state.follower().await?;

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

                        self.exit_state.shutdown().await?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
