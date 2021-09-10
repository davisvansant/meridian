pub struct Volatile {
    pub commit_index: u32,
    pub last_applied: u32,
}

impl Volatile {
    pub async fn init() -> Result<Volatile, Box<dyn std::error::Error>> {
        let commit_index = 0;
        let last_applied = 0;

        Ok(Volatile {
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
        let test_volatile_state = Volatile::init().await?;
        assert_eq!(test_volatile_state.commit_index, 0);
        assert_eq!(test_volatile_state.last_applied, 0);
        Ok(())
    }
}
