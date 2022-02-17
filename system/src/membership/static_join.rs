use crate::channel::MembershipCommunicationsSender;
use crate::channel::MembershipListSender;
use crate::channel::{get_initial, send_message};

pub struct StaticJoin {
    membership_communications_sender: MembershipCommunicationsSender,
    membership_list_sender: MembershipListSender,
}

impl StaticJoin {
    pub async fn init(
        membership_communications_sender: MembershipCommunicationsSender,
        membership_list_sender: MembershipListSender,
    ) -> StaticJoin {
        StaticJoin {
            membership_communications_sender,
            membership_list_sender,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let initial_nodes = get_initial(&self.membership_list_sender).await?;

        for origin in initial_nodes {
            let bytes = b"ping".to_vec();

            send_message(&self.membership_communications_sender, bytes, origin).await?;
        }

        Ok(())
    }
}
