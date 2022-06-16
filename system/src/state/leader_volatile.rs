use crate::info;

#[derive(Copy, Clone)]
pub struct LeaderVolatile {
    pub next_index: u32,
    pub match_index: u32,
}

impl LeaderVolatile {
    pub async fn init() -> Result<LeaderVolatile, Box<dyn std::error::Error>> {
        let next_index = 0;
        let match_index = 0;

        info!("leader volatile state initialized ...");

        Ok(LeaderVolatile {
            next_index,
            match_index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_leader_volatile_state = LeaderVolatile::init().await?;
        assert_eq!(test_leader_volatile_state.next_index, 0);
        assert_eq!(test_leader_volatile_state.match_index, 0);
        Ok(())
    }
}
