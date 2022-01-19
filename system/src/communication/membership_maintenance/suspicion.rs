#[derive(Debug, PartialEq)]
enum Suspicion {
    Alive,
    Confirm,
    Suspect,
}

#[derive(Debug)]
pub struct GroupMember {
    suspicion: Suspicion,
    incarnation: u32,
}

impl GroupMember {
    pub async fn init() -> GroupMember {
        let suspicion = Suspicion::Alive;
        let incarnation = 0;

        GroupMember {
            suspicion,
            incarnation,
        }
    }

    pub async fn alive(&mut self) {
        self.suspicion = Suspicion::Alive
    }

    pub async fn confirm(&mut self) {
        self.suspicion = Suspicion::Confirm
    }

    pub async fn suspect(&mut self) {
        self.suspicion = Suspicion::Suspect;

        self.incarnation += 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn group_member_init() -> Result<(), Box<dyn std::error::Error>> {
        let test_group_member = GroupMember::init().await;

        assert_eq!(test_group_member.suspicion, Suspicion::Alive);
        assert_eq!(test_group_member.incarnation, 0);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn group_member_alive() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_group_member = GroupMember::init().await;

        assert_eq!(test_group_member.suspicion, Suspicion::Alive);

        test_group_member.suspicion = Suspicion::Confirm;

        assert_eq!(test_group_member.suspicion, Suspicion::Confirm);

        test_group_member.alive().await;

        assert_eq!(test_group_member.suspicion, Suspicion::Alive);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn group_member_confirm() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_group_member = GroupMember::init().await;

        assert_eq!(test_group_member.suspicion, Suspicion::Alive);

        test_group_member.confirm().await;

        assert_eq!(test_group_member.suspicion, Suspicion::Confirm);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn group_member_suspect() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_group_member = GroupMember::init().await;

        assert_eq!(test_group_member.suspicion, Suspicion::Alive);

        test_group_member.suspect().await;

        assert_eq!(test_group_member.suspicion, Suspicion::Suspect);

        Ok(())
    }
}
