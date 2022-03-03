use crate::channel::MembershipCommunicationsSender;
use crate::channel::MembershipListSender;
use crate::channel::{get_initial, send_message};
use crate::membership::Message;

use crate::channel::get_alive;
use crate::channel::get_confirmed;
use crate::channel::get_node;
use crate::channel::get_suspected;

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
        let node = get_node(&self.membership_list_sender).await?;
        let alive_list = get_alive(&self.membership_list_sender).await?;
        let suspected_list = get_suspected(&self.membership_list_sender).await?;
        let confirmed_list = get_confirmed(&self.membership_list_sender).await?;
        let initial_nodes = get_initial(&self.membership_list_sender).await?;
        let ping = Message::Ping
            .build_list(node, alive_list, suspected_list, confirmed_list)
            .await;

        for origin in initial_nodes {
            // let ping = Message::Ping.build().await.to_vec();
            // let node = get_node(&self.membership_list_sender).await?;
            // let alive_list = get_alive(&self.membership_list_sender).await?;
            // let suspected_list = get_suspected(&self.membership_list_sender).await?;
            // let confirmed_list = get_confirmed(&self.membership_list_sender).await?;

            // let ping = Message::Ping
            //     .build_list(node, alive_list, suspected_list, confirmed_list)
            //     .await;

            send_message(&self.membership_communications_sender, &ping, origin).await?;
        }

        Ok(())
    }
}
