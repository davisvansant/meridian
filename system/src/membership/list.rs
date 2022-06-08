use std::collections::HashMap;
use std::net::SocketAddr;

use uuid::Uuid;

use crate::channel::membership_list::{
    MembershipListReceiver, MembershipListRequest, MembershipListResponse,
};
use crate::node::Node;
use crate::{error, info, warn};

pub struct List {
    server: Node,
    initial: Vec<SocketAddr>,
    alive: HashMap<Uuid, Node>,
    suspected: HashMap<Uuid, Node>,
    confirmed: HashMap<Uuid, Node>,
    receiver: MembershipListReceiver,
}

impl List {
    pub async fn init(
        server: Node,
        initial: Vec<SocketAddr>,
        receiver: MembershipListReceiver,
    ) -> Result<List, Box<dyn std::error::Error>> {
        let alive = HashMap::with_capacity(10);
        let suspected = HashMap::with_capacity(10);
        let confirmed = HashMap::with_capacity(10);

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
        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                MembershipListRequest::GetNode => {
                    let node = self.server;

                    if let Err(error) = response.send(MembershipListResponse::Node(node)) {
                        // println!("error sending membership list response -> {:?}", error);
                        error!("error sending membership list response -> {:?}", error);
                    }
                }
                MembershipListRequest::GetInitial => {
                    let initial = self.initial.to_vec();

                    if let Err(error) = response.send(MembershipListResponse::Initial(initial)) {
                        // println!("error sending membership list response -> {:?}", error);
                        error!("error sending membership list response -> {:?}", error);
                    }
                }
                MembershipListRequest::GetAlive => {
                    let mut alive = Vec::with_capacity(self.alive.len());

                    for member in self.alive.values() {
                        alive.push(member.to_owned());
                    }

                    if let Err(error) = response.send(MembershipListResponse::Alive(alive)) {
                        // println!("error sending membership list response -> {:?}", error);
                        error!("error sending membership list response -> {:?}", error);
                    }
                }
                MembershipListRequest::GetSuspected => {
                    let mut suspected = Vec::with_capacity(self.suspected.len());

                    for node in self.suspected.values() {
                        suspected.push(node.to_owned());
                    }

                    if let Err(error) = response.send(MembershipListResponse::Suspected(suspected))
                    {
                        // println!("error sending membership list response -> {:?}", error);
                        error!("error sending membership list response -> {:?}", error);
                    }
                }
                MembershipListRequest::GetConfirmed => {
                    let mut confirmed = Vec::with_capacity(self.confirmed.len());

                    for node in self.confirmed.values() {
                        confirmed.push(node.to_owned());
                    }

                    if let Err(error) = response.send(MembershipListResponse::Confirmed(confirmed))
                    {
                        // println!("error sending membership list response -> {:?}", error);
                        error!("error sending membership list response -> {:?}", error);
                    }
                }
                MembershipListRequest::InsertAlive(node) => {
                    if node != self.server {
                        self.insert_alive(node).await?;
                    }
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
                    // println!("shutting down membership list");
                    info!("shutting down membership list");

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
                    // println!("updated node! {:?}", value)
                    info!("updated node! {:?}", value);
                }
                None => {
                    // println!("added node !"),
                    info!("added node !");
                }
            }
        }

        Ok(())
    }

    pub async fn insert_supsected(&mut self, node: Node) -> Result<(), Box<dyn std::error::Error>> {
        if node != self.server {
            match self.suspected.insert(node.id, node) {
                Some(value) => {
                    // println!("updated node in suspected list! {:?}", value),
                    info!("updated node in suspected list! {:?}", value);
                }
                None => {
                    // println!("added node to suspected list!"),
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
                    // println!("updated node in confirmed list! {:?}", value),
                    info!("updated node in confirmed list! {:?}", value);
                }
                None => {
                    // println!("added node to confirmed list!"),
                    info!("added node to confirmed list!");
                }
            }
        }

        Ok(())
    }

    pub async fn remove_alive(&mut self, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(remove_alive) = self.alive.remove(&node.id) {
            // println!("removed from alive group - > {:?}", remove_alive);
            info!("removed from alive group - > {:?}", remove_alive);
        }

        Ok(())
    }

    pub async fn remove_suspected(
        &mut self,
        node: &Node,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(remove_suspected) = self.suspected.remove(&node.id) {
            // println!("removed from suspected group - > {:?}", remove_suspected);
            info!("removed from suspected group - > {:?}", remove_suspected);
        }

        Ok(())
    }

    pub async fn remove_confirmed(
        &mut self,
        node: &Node,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(remove_confirmed) = self.confirmed.remove(&node.id) {
            // println!("removed from confirmed group - > {:?}", remove_confirmed);
            info!("removed from confirmed group - > {:?}", remove_confirmed);
        }

        Ok(())
    }
}
