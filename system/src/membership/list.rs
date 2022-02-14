use std::collections::HashMap;
use std::net::SocketAddr;

use uuid::Uuid;

use crate::node::Node;

pub struct List {
    initial: Vec<SocketAddr>,
    alive: HashMap<Uuid, Node>,
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
}
