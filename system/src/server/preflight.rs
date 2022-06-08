use tokio::time::{sleep, Duration};

use crate::channel::membership::MembershipSender;
use crate::channel::membership::{launch_failure_detector, static_join};
use crate::error;

pub async fn run(membership: &MembershipSender) -> Result<(), Box<dyn std::error::Error>> {
    let mut errors = 0;

    while errors <= 4 {
        let (active, expected) = static_join(membership).await?;

        if active == expected {
            launch_failure_detector(membership).await?;

            return Ok(());
        } else {
            error!("expecting {} peers | {} active peers", &expected, &active);
            error!("attempting preflight again...");

            errors += 1;

            match errors {
                1 => sleep(Duration::from_secs(10)).await,
                2 => sleep(Duration::from_secs(20)).await,
                3 => sleep(Duration::from_secs(30)).await,
                4 => sleep(Duration::from_secs(40)).await,
                _ => {}
            }
        }

        if errors == 4 {
            let error = String::from("preflight tasks failed...shutting down...");

            return Err(Box::from(error));
        }
    }

    Ok(())
}
