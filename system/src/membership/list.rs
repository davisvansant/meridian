use std::collections::HashMap;
use std::net::SocketAddr;

use uuid::Uuid;

use crate::channel::{MembershipListReceiver, MembershipListRequest, MembershipListResponse};
use crate::node::Node;

pub struct List {
    pub initial: Vec<SocketAddr>,
    pub alive: HashMap<Uuid, Node>,
    suspected: HashMap<Uuid, Node>,
    confirmed: HashMap<Uuid, Node>,
    receiver: MembershipListReceiver,
}

impl List {
    pub async fn init(
        initial: Vec<SocketAddr>,
        receiver: MembershipListReceiver,
    ) -> Result<List, Box<dyn std::error::Error>> {
        let alive = HashMap::with_capacity(10);
        let suspected = HashMap::with_capacity(10);
        let confirmed = HashMap::with_capacity(10);

        Ok(List {
            initial,
            alive,
            suspected,
            confirmed,
            receiver,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                MembershipListRequest::GetInitial => {
                    let initial = self.initial.to_vec();
                }
                MembershipListRequest::GetAlive => {
                    // let alive = self.alive.clone();

                    let mut alive = Vec::with_capacity(self.alive.len());

                    for member in self.alive.values() {
                        alive.push(member.to_owned());
                    }

                    if let Err(error) = response.send(MembershipListResponse::Alive(alive)) {
                        println!("error sending membership list response -> {:?}", error);
                    }
                }
                MembershipListRequest::GetSuspected => {
                    let suspected = self.suspected.clone();
                }
                MembershipListRequest::GetConfirmed => {
                    let confirmed = self.confirmed.clone();
                }
                MembershipListRequest::InsertAlive(node) => {
                    self.insert_alive(node).await?;
                }
                MembershipListRequest::InsertSuspected(node) => {
                    self.insert_supsected(node).await?;
                }
                MembershipListRequest::InsertConfirmed(node) => {
                    self.insert_confirmed(node).await?;
                }
                MembershipListRequest::RemoveAlive(node) => {
                    self.remove_alive(&node).await?;
                }
                MembershipListRequest::RemoveSuspected(node) => {
                    self.remove_suspected(&node).await?;
                }
                MembershipListRequest::RemoveConfirmed(node) => {
                    self.remove_confirmed(&node).await?;
                }
                MembershipListRequest::Shutdown => {
                    println!("shutting down membership list");

                    self.receiver.close();
                }
            }
        }

        Ok(())
    }

    pub async fn insert_alive(&mut self, node: Node) -> Result<(), Box<dyn std::error::Error>> {
        match self.alive.insert(node.id, node) {
            Some(value) => println!("updated node! {:?}", value),
            None => println!("added node !"),
        }

        Ok(())
    }

    pub async fn insert_supsected(&mut self, node: Node) -> Result<(), Box<dyn std::error::Error>> {
        match self.suspected.insert(node.id, node) {
            Some(value) => println!("updated node in suspected list! {:?}", value),
            None => println!("added node to suspected list!"),
        }

        Ok(())
    }

    pub async fn insert_confirmed(&mut self, node: Node) -> Result<(), Box<dyn std::error::Error>> {
        match self.confirmed.insert(node.id, node) {
            Some(value) => println!("updated node in confirmed list! {:?}", value),
            None => println!("added node to confirmed list!"),
        }

        Ok(())
    }

    pub async fn remove_alive(&mut self, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(remove_alive) = self.alive.remove(&node.id) {
            println!("removed from alive group - > {:?}", remove_alive);
        }

        Ok(())
    }

    pub async fn remove_suspected(
        &mut self,
        node: &Node,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(remove_suspected) = self.suspected.remove(&node.id) {
            println!("removed from suspected group - > {:?}", remove_suspected);
        }

        Ok(())
    }

    pub async fn remove_confirmed(
        &mut self,
        node: &Node,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(remove_confirmed) = self.confirmed.remove(&node.id) {
            println!("removed from confirmed group - > {:?}", remove_confirmed);
        }

        Ok(())
    }
}
