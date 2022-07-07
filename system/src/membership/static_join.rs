use crate::channel::membership::list::ListChannel;
use crate::channel::membership::sender::{Dissemination, DisseminationSender};
use crate::membership::Message;

pub struct StaticJoin {
    dissemination: DisseminationSender,
    list: ListChannel,
}

impl StaticJoin {
    pub async fn init(dissemination: DisseminationSender, list: ListChannel) -> StaticJoin {
        StaticJoin {
            dissemination,
            list,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let node = self.list.get_node().await?;
        let alive_list = self.list.get_alive().await?;
        let suspected_list = self.list.get_suspected().await?;
        let confirmed_list = self.list.get_confirmed().await?;
        let initial_nodes = self.list.get_initial().await?;

        let ping = Message::Ping
            .build_list(&node, None, &alive_list, &suspected_list, &confirmed_list)
            .await;

        for origin in initial_nodes {
            self.dissemination
                .send(Dissemination::Message(ping.to_owned(), origin))?;
        }

        Ok(())
    }
}
