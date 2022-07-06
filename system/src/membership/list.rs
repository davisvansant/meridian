use std::collections::HashMap;
use std::net::SocketAddr;

use uuid::Uuid;

use crate::channel::membership::list::{ListReceiver, ListRequest, ListResponse};
use crate::node::Node;
use crate::{error, info};

pub struct List {
    server: Node,
    initial: Vec<SocketAddr>,
    alive: HashMap<Uuid, Node>,
    suspected: HashMap<Uuid, Node>,
    confirmed: HashMap<Uuid, Node>,
    receiver: ListReceiver,
}

impl List {
    pub async fn init(
        server: Node,
        initial: Vec<SocketAddr>,
        receiver: ListReceiver,
    ) -> Result<List, Box<dyn std::error::Error>> {
        let alive = HashMap::with_capacity(10);
        let suspected = HashMap::with_capacity(10);
        let confirmed = HashMap::with_capacity(10);

        info!("initialized!");

        Ok(List {
            server,
            initial,
            alive,
            suspected,
            confirmed,
            receiver,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("running...");

        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                ListRequest::GetNode => {
                    let node = self.server;

                    response.send(ListResponse::Node(node))?;
                }
                ListRequest::GetInitial => {
                    let initial = self.initial.to_vec();

                    response.send(ListResponse::Initial(initial))?;
                }
                ListRequest::GetAlive => {
                    let mut alive = Vec::with_capacity(self.alive.len());

                    for member in self.alive.values() {
                        alive.push(member.to_owned());
                    }

                    response.send(ListResponse::Alive(alive))?;
                }
                ListRequest::GetSuspected => {
                    let mut suspected = Vec::with_capacity(self.suspected.len());

                    for node in self.suspected.values() {
                        suspected.push(node.to_owned());
                    }

                    response.send(ListResponse::Suspected(suspected))?;
                }
                ListRequest::GetConfirmed => {
                    let mut confirmed = Vec::with_capacity(self.confirmed.len());

                    for node in self.confirmed.values() {
                        confirmed.push(node.to_owned());
                    }

                    response.send(ListResponse::Confirmed(confirmed))?;
                }
                ListRequest::InsertAlive(node) => {
                    if node != self.server {
                        self.insert_alive(node).await?;
                    }
                }
                ListRequest::InsertSuspected(node) => {
                    self.insert_supsected(node).await?;
                }
                ListRequest::InsertConfirmed(node) => {
                    self.insert_confirmed(node).await?;
                }
                ListRequest::RemoveAlive(node) => {
                    self.remove_alive(&node).await?;
                }
                ListRequest::RemoveSuspected(node) => {
                    self.remove_suspected(&node).await?;
                }
                ListRequest::RemoveConfirmed(node) => {
                    self.remove_confirmed(&node).await?;
                }
                ListRequest::Shutdown => {
                    info!("shutting down...");

                    self.receiver.close();
                }
            }
        }

        Ok(())
    }

    pub async fn insert_alive(&mut self, node: Node) -> Result<(), Box<dyn std::error::Error>> {
        if node != self.server {
            match self.alive.insert(node.id, node) {
                Some(value) => {
                    info!("updated node! {:?}", value);
                }
                None => {
                    info!("added node -> {:?}!", &node);
                }
            }
        }

        Ok(())
    }

    pub async fn insert_supsected(&mut self, node: Node) -> Result<(), Box<dyn std::error::Error>> {
        if node != self.server {
            match self.suspected.insert(node.id, node) {
                Some(value) => {
                    info!("updated node in suspected list! {:?}", value);
                }
                None => {
                    info!("added node to suspected list!");
                }
            }
        }

        Ok(())
    }

    pub async fn insert_confirmed(&mut self, node: Node) -> Result<(), Box<dyn std::error::Error>> {
        if node != self.server {
            match self.confirmed.insert(node.id, node) {
                Some(value) => {
                    info!("updated node in confirmed list! {:?}", value);
                }
                None => {
                    info!("added node to confirmed list!");
                }
            }
        }

        Ok(())
    }

    pub async fn remove_alive(&mut self, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(remove_alive) = self.alive.remove(&node.id) {
            info!("removed from alive group - > {:?}", remove_alive);
        }

        Ok(())
    }

    pub async fn remove_suspected(
        &mut self,
        node: &Node,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(remove_suspected) = self.suspected.remove(&node.id) {
            info!("removed from suspected group - > {:?}", remove_suspected);
        }

        Ok(())
    }

    pub async fn remove_confirmed(
        &mut self,
        node: &Node,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(remove_confirmed) = self.confirmed.remove(&node.id) {
            info!("removed from confirmed group - > {:?}", remove_confirmed);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::membership::list::ListRequest;
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_node_address = std::net::IpAddr::from_str("127.0.0.1")?;
        let test_node = Node::init(test_node_address, 10000, 15000, 20000).await?;
        let test_initial_peers = Vec::with_capacity(0);
        let (test_list_sender, test_list_receiver) = ListRequest::build().await;
        let test_list = List::init(test_node, test_initial_peers, test_list_receiver).await?;

        assert!(test_list.initial.is_empty());
        assert!(test_list.alive.is_empty());
        assert!(test_list.suspected.is_empty());
        assert!(test_list.confirmed.is_empty());
        assert_eq!(test_list_sender.capacity(), 64);

        Ok(())
    }
}
