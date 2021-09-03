struct VolatileState {
    commit_index: u32,
    last_applied: u32,
}

pub struct Leader {
    volatile_state: VolatileState,
}

impl Leader {
    pub async fn init() -> Result<Leader, Box<dyn std::error::Error>> {
        let volatile_state = VolatileState {
            commit_index: 0,
            last_applied: 0,
        };

        Ok(Leader { volatile_state })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_leader = Leader::init().await?;
        assert_eq!(test_leader.volatile_state.commit_index, 0);
        assert_eq!(test_leader.volatile_state.last_applied, 0);
        Ok(())
    }
}
