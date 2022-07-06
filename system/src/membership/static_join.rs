use crate::channel::membership::list::{ListRequest, ListSender};
use crate::channel::membership::sender::{Dissemination, DisseminationSender};
use crate::membership::Message;

pub struct StaticJoin {
    dissemination: DisseminationSender,
    list_sender: ListSender,
}

impl StaticJoin {
    pub async fn init(dissemination: DisseminationSender, list_sender: ListSender) -> StaticJoin {
        StaticJoin {
            dissemination,
            list_sender,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let node = ListRequest::get_node(&self.list_sender).await?;
        let alive_list = ListRequest::get_alive(&self.list_sender).await?;
        let suspected_list = ListRequest::get_suspected(&self.list_sender).await?;
        let confirmed_list = ListRequest::get_confirmed(&self.list_sender).await?;
        let initial_nodes = ListRequest::get_initial(&self.list_sender).await?;

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
