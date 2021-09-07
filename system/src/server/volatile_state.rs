pub struct VolatileState {
    pub commit_index: u32,
    pub last_applied: u32,
}

impl VolatileState {
    pub async fn init() -> Result<VolatileState, Box<dyn std::error::Error>> {
        let commit_index = 0;
        let last_applied = 0;

        Ok(VolatileState {
            commit_index,
            last_applied,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_volatile_state = VolatileState::init().await?;
        assert_eq!(test_volatile_state.commit_index, 0);
        assert_eq!(test_volatile_state.last_applied, 0);
        Ok(())
    }
}
