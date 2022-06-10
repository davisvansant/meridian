use flexbuffers::{Builder, BuilderOptions};

use std::net::IpAddr;
use std::str::FromStr;

use uuid::Uuid;

use crate::node::Node;

#[derive(Debug, PartialEq)]
pub enum Message {
    Ack,
    Ping,
    PingReq,
}

impl Message {
    pub async fn build_list(
        &self,
        node: &Node,
        alive_list: &[Node],
        suspected_list: &[Node],
        confirmed_list: &[Node],
    ) -> Vec<u8> {
        let flexbuffer_options = BuilderOptions::SHARE_NONE;
        let mut flexbuffers_builder = Builder::new(flexbuffer_options);
        let mut message_data = flexbuffers_builder.start_map();

        match self {
            Message::Ack => {
                message_data.push("message", "ack");
            }
            Message::Ping => {
                message_data.push("message", "ping");
            }
            Message::PingReq => {
                message_data.push("message", "ping_req");
            }
        }

        let mut node_map = message_data.start_map("node");

        node_map.push("id", node.id.to_string().as_str());
        node_map.push("address", node.address.to_string().as_str());
        node_map.push("client_port", node.client_port);
        node_map.push("cluster_port", node.cluster_port);
        node_map.push("membership_port", node.membership_port);

        node_map.end_map();

        let mut alive_list_vector = message_data.start_vector("alive_list");

        if !alive_list.is_empty() {
            for alive_node in alive_list {
                let mut alive_node_map = alive_list_vector.start_map();

                alive_node_map.push("id", alive_node.id.to_string().as_str());
                alive_node_map.push("address", alive_node.address.to_string().as_str());
                alive_node_map.push("client_port", alive_node.client_port);
                alive_node_map.push("cluster_port", alive_node.cluster_port);
                alive_node_map.push("membership_port", alive_node.membership_port);
            }
        }

        alive_list_vector.end_vector();

        let mut suspected_list_vector = message_data.start_vector("suspected_list");

        if !suspected_list.is_empty() {
            for suspected_node in suspected_list {
                let mut suspected_node_map = suspected_list_vector.start_map();

                suspected_node_map.push("id", suspected_node.id.to_string().as_str());
                suspected_node_map.push("address", suspected_node.address.to_string().as_str());
                suspected_node_map.push("client_port", suspected_node.client_port);
                suspected_node_map.push("cluster_port", suspected_node.cluster_port);
                suspected_node_map.push("membership_port", suspected_node.membership_port);
            }
        }

        suspected_list_vector.end_vector();

        let mut confirmed_list_vector = message_data.start_vector("confirmed_list");

        if !confirmed_list.is_empty() {
            for confirmed_node in confirmed_list {
                let mut confirmed_node_map = confirmed_list_vector.start_map();

                confirmed_node_map.push("id", confirmed_node.id.to_string().as_str());
                confirmed_node_map.push("address", confirmed_node.address.to_string().as_str());
                confirmed_node_map.push("client_port", confirmed_node.client_port);
                confirmed_node_map.push("cluster_port", confirmed_node.cluster_port);
                confirmed_node_map.push("membership_port", confirmed_node.membership_port);
            }
        }

        confirmed_list_vector.end_vector();

        message_data.end_map();

        flexbuffers_builder.take_buffer()
    }
    pub async fn from_list(
        message_data: &[u8],
    ) -> Result<(Message, Node, Vec<Node>, Vec<Node>, Vec<Node>), Box<dyn std::error::Error>> {
        let flexbuffers_root = flexbuffers::Reader::get_root(message_data)?;

        let message = match flexbuffers_root.as_map().idx("message").as_str() {
            "ack" => Message::Ack,
            "ping" => Message::Ping,
            "ping-req" => Message::PingReq,
            _ => panic!("could not parse incoming udp message..."),
        };

        let flexbuffer_node = flexbuffers_root.as_map().idx("node").as_map();

        let id = Uuid::parse_str(flexbuffer_node.idx("id").as_str())?;
        let address = IpAddr::from_str(flexbuffer_node.idx("address").as_str())?;
        let client_port = flexbuffer_node.idx("client_port").as_u16();
        let cluster_port = flexbuffer_node.idx("cluster_port").as_u16();
        let membership_port = flexbuffer_node.idx("membership_port").as_u16();

        let node = Node {
            id,
            address,
            client_port,
            cluster_port,
            membership_port,
        };

        let flexbuffer_alive_list = flexbuffers_root.as_map().idx("alive_list").as_vector();
        let mut alive_list = Vec::with_capacity(flexbuffer_alive_list.len());

        for alive_node in flexbuffer_alive_list.iter() {
            let id = Uuid::parse_str(alive_node.as_map().idx("id").as_str())?;
            let address = IpAddr::from_str(alive_node.as_map().idx("address").as_str())?;
            let client_port = alive_node.as_map().idx("client_port").as_u16();
            let cluster_port = alive_node.as_map().idx("cluster_port").as_u16();
            let membership_port = alive_node.as_map().idx("membership_port").as_u16();

            let node = Node {
                id,
                address,
                client_port,
                cluster_port,
                membership_port,
            };

            alive_list.push(node);
        }

        let flexbuffer_suspected_list = flexbuffers_root.as_map().idx("suspected_list").as_vector();
        let mut suspected_list = Vec::with_capacity(flexbuffer_suspected_list.len());

        for suspected_node in flexbuffer_suspected_list.iter() {
            let id = Uuid::parse_str(suspected_node.as_map().idx("id").as_str())?;
            let address = IpAddr::from_str(suspected_node.as_map().idx("address").as_str())?;
            let client_port = suspected_node.as_map().idx("client_port").as_u16();
            let cluster_port = suspected_node.as_map().idx("cluster_port").as_u16();
            let membership_port = suspected_node.as_map().idx("membership_port").as_u16();

            let node = Node {
                id,
                address,
                client_port,
                cluster_port,
                membership_port,
            };

            suspected_list.push(node);
        }

        let flexbuffer_confirmed_list = flexbuffers_root.as_map().idx("confirmed_list").as_vector();
        let mut confirmed_list = Vec::with_capacity(flexbuffer_confirmed_list.len());

        for confirmed_node in flexbuffer_confirmed_list.iter() {
            let id = Uuid::parse_str(confirmed_node.as_map().idx("id").as_str())?;
            let address = IpAddr::from_str(confirmed_node.as_map().idx("address").as_str())?;
            let client_port = confirmed_node.as_map().idx("client_port").as_u16();
            let cluster_port = confirmed_node.as_map().idx("cluster_port").as_u16();
            let membership_port = confirmed_node.as_map().idx("membership_port").as_u16();

            let node = Node {
                id,
                address,
                client_port,
                cluster_port,
                membership_port,
            };

            confirmed_list.push(node);
        }

        Ok((message, node, alive_list, suspected_list, confirmed_list))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[tokio::test(flavor = "multi_thread")]
    // async fn message_ack() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_message_ack = Message::Ack.build().await;

    //     assert_eq!(test_message_ack, b"ack");
    //     assert_eq!(test_message_ack.len(), 3);

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn message_ping() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_message_ping = Message::Ping.build().await;

    //     assert_eq!(test_message_ping, b"ping");
    //     assert_eq!(test_message_ping.len(), 4);

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn message_ping_req() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_message_ping_req = Message::PingReq.build().await;

    //     assert_eq!(test_message_ping_req, b"ping-req");
    //     assert_eq!(test_message_ping_req.len(), 8);

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn message_from_ack_bytes() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_ack_bytes = b"ack"; //bar
    //     let test_message_ack = Message::from_bytes(test_ack_bytes).await;

    //     assert_eq!(test_message_ack, Message::Ack);

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn message_from_ping_bytes() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_ping_bytes = b"ping";
    //     let test_message_ping = Message::from_bytes(test_ping_bytes).await;

    //     assert_eq!(test_message_ping, Message::Ping);

    //     Ok(())
    // }

    // #[tokio::test(flavor = "multi_thread")]
    // async fn message_from_ping_req_bytes() -> Result<(), Box<dyn std::error::Error>> {
    //     let test_ping_req_bytes = b"ping-req";
    //     let test_message_ping_req = Message::from_bytes(test_ping_req_bytes).await;

    //     assert_eq!(test_message_ping_req, Message::PingReq);

    //     Ok(())
    // }
}
