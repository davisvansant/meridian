pub(crate) struct VolatileState {
    pub(crate) next_index: u32,
    pub(crate) match_index: u32,
}

impl VolatileState {
    pub(crate) async fn init() -> Result<VolatileState, Box<dyn std::error::Error>> {
        let next_index = 0;
        let match_index = 0;

        println!("leader volatile state initialized ...");

        Ok(VolatileState {
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
        let test_volatile_state = VolatileState::init().await?;
        assert_eq!(test_volatile_state.next_index, 0);
        assert_eq!(test_volatile_state.match_index, 0);
        Ok(())
    }
}
