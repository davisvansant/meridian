use std::collections::HashMap;
use std::net::SocketAddr;

use uuid::Uuid;

use crate::node::Node;

pub struct List {
    pub initial: Vec<SocketAddr>,
    pub alive: HashMap<Uuid, Node>,
    suspected: HashMap<Uuid, Node>,
    confirmed: HashMap<Uuid, Node>,
}

impl List {
    pub async fn init(initial: Vec<SocketAddr>) -> Result<List, Box<dyn std::error::Error>> {
        let alive = HashMap::with_capacity(10);
        let suspected = HashMap::with_capacity(10);
        let confirmed = HashMap::with_capacity(10);

        Ok(List {
            initial,
            alive,
            suspected,
            confirmed,
        })
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