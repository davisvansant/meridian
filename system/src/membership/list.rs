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

    pub async fn remove_alive(&mut self, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(remove_alive) = self.alive.remove(&node.id) {
            println!("removed from alive group - > {:?}", remove_alive);
        }

        Ok(())
    }
}
