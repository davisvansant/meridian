use flexbuffers::{Builder, BuilderOptions, Reader};

use std::net::{IpAddr, SocketAddr};
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
        suspect: Option<(&SocketAddr, &SocketAddr)>,
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

        let mut suspect_message_vector = message_data.start_vector("suspect");

        if let Some(suspect) = suspect {
            let mut suspect_message_map = suspect_message_vector.start_map();

            suspect_message_map.push("forward_address", suspect.0.to_string().as_str());
            suspect_message_map.push("suspected_address", suspect.1.to_string().as_str());

            suspect_message_map.end_map();
        }

        suspect_message_vector.end_vector();

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
    ) -> Result<
        (
            Message,
            Node,
            Option<(SocketAddr, SocketAddr)>,
            Vec<Node>,
            Vec<Node>,
            Vec<Node>,
        ),
        Box<dyn std::error::Error>,
    > {
        let flexbuffers_root = flexbuffers::Reader::get_root(message_data)?;

        let message = match flexbuffers_root.as_map().idx("message").as_str() {
            "ack" => Message::Ack,
            "ping" => Message::Ping,
            "ping_req" => Message::PingReq,
            _ => panic!("could not parse incoming udp message..."),
        };

        let flexbuffer_node = flexbuffers_root.as_map().idx("node");
        let node = Self::build_node(flexbuffer_node).await?;

        let flexbuffer_suspect = flexbuffers_root.as_map().idx("suspect").as_vector();
        let suspect = if flexbuffer_suspect.is_empty() {
            None
        } else {
            let flexbuffer_suspect_message = flexbuffer_suspect.idx(0).as_map();
            let forward_address = flexbuffer_suspect_message.idx("forward_address").as_str();
            let suspect_address = flexbuffer_suspect_message.idx("suspected_address").as_str();

            Some((
                SocketAddr::from_str(forward_address)?,
                SocketAddr::from_str(suspect_address)?,
            ))
        };

        let flexbuffer_alive_list = flexbuffers_root.as_map().idx("alive_list").as_vector();
        let mut alive_list = Vec::with_capacity(flexbuffer_alive_list.len());

        for alive_node in flexbuffer_alive_list.iter() {
            let node = Self::build_node(alive_node).await?;

            alive_list.push(node);
        }

        let flexbuffer_suspected_list = flexbuffers_root.as_map().idx("suspected_list").as_vector();
        let mut suspected_list = Vec::with_capacity(flexbuffer_suspected_list.len());

        for suspected_node in flexbuffer_suspected_list.iter() {
            let node = Self::build_node(suspected_node).await?;

            suspected_list.push(node);
        }

        let flexbuffer_confirmed_list = flexbuffers_root.as_map().idx("confirmed_list").as_vector();
        let mut confirmed_list = Vec::with_capacity(flexbuffer_confirmed_list.len());

        for confirmed_node in flexbuffer_confirmed_list.iter() {
            let node = Self::build_node(confirmed_node).await?;

            confirmed_list.push(node);
        }

        Ok((
            message,
            node,
            suspect,
            alive_list,
            suspected_list,
            confirmed_list,
        ))
    }

    async fn build_node(node: Reader<&[u8]>) -> Result<Node, Box<dyn std::error::Error>> {
        let id = Uuid::parse_str(node.as_map().idx("id").as_str())?;
        let address = IpAddr::from_str(node.as_map().idx("address").as_str())?;
        let client_port = node.as_map().idx("client_port").as_u16();
        let cluster_port = node.as_map().idx("cluster_port").as_u16();
        let membership_port = node.as_map().idx("membership_port").as_u16();

        Ok(Node {
            id,
            address,
            client_port,
            cluster_port,
            membership_port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread")]
    async fn ack() -> Result<(), Box<dyn std::error::Error>> {
        let test_node_address = std::net::IpAddr::from_str("0.0.0.0")?;
        let test_node = crate::node::Node::init(test_node_address, 10000, 15000, 20000).await?;

        let mut test_alive_list = Vec::with_capacity(1);
        let mut test_suspected_list = Vec::with_capacity(1);
        let mut test_confirmed_list = Vec::with_capacity(1);

        test_alive_list.push(test_node.to_owned());
        test_suspected_list.push(test_node.to_owned());
        test_confirmed_list.push(test_node.to_owned());

        let test_ack = Message::Ack
            .build_list(
                &test_node,
                None,
                &test_alive_list,
                &test_suspected_list,
                &test_confirmed_list,
            )
            .await;

        assert!(!test_ack.is_empty());

        let (
            test_from_list_message,
            test_from_list_node,
            test_from_list_suspect,
            test_from_list_alive_list,
            test_from_list_suspected_list,
            test_from_list_confirmed_list,
        ) = Message::from_list(&test_ack).await?;

        assert_eq!(test_from_list_message, Message::Ack);
        assert_eq!(test_from_list_node.id.get_version_num(), 4);
        assert!(test_from_list_suspect.is_none());
        assert_eq!(test_from_list_alive_list.len(), 1);
        assert_eq!(test_from_list_suspected_list.len(), 1);
        assert_eq!(test_from_list_confirmed_list.len(), 1);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ping() -> Result<(), Box<dyn std::error::Error>> {
        let test_node_address = std::net::IpAddr::from_str("0.0.0.0")?;
        let test_node = crate::node::Node::init(test_node_address, 10000, 15000, 20000).await?;

        let mut test_alive_list = Vec::with_capacity(1);
        let mut test_suspected_list = Vec::with_capacity(1);
        let mut test_confirmed_list = Vec::with_capacity(1);

        test_alive_list.push(test_node.to_owned());
        test_suspected_list.push(test_node.to_owned());
        test_confirmed_list.push(test_node.to_owned());

        let test_ping = Message::Ping
            .build_list(
                &test_node,
                None,
                &test_alive_list,
                &test_suspected_list,
                &test_confirmed_list,
            )
            .await;

        assert!(!test_ping.is_empty());

        let (
            test_from_list_message,
            test_from_list_node,
            test_from_list_suspect,
            test_from_list_alive_list,
            test_from_list_suspected_list,
            test_from_list_confirmed_list,
        ) = Message::from_list(&test_ping).await?;

        assert_eq!(test_from_list_message, Message::Ping);
        assert_eq!(test_from_list_node.id.get_version_num(), 4);
        assert!(test_from_list_suspect.is_none());
        assert_eq!(test_from_list_alive_list.len(), 1);
        assert_eq!(test_from_list_suspected_list.len(), 1);
        assert_eq!(test_from_list_confirmed_list.len(), 1);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ping_req() -> Result<(), Box<dyn std::error::Error>> {
        let test_node_address = std::net::IpAddr::from_str("0.0.0.0")?;
        let test_node = crate::node::Node::init(test_node_address, 10000, 15000, 20000).await?;

        let mut test_alive_list = Vec::with_capacity(1);
        let mut test_suspected_list = Vec::with_capacity(1);
        let mut test_confirmed_list = Vec::with_capacity(1);

        test_alive_list.push(test_node.to_owned());
        test_suspected_list.push(test_node.to_owned());
        test_confirmed_list.push(test_node.to_owned());

        let test_ping_req = Message::PingReq
            .build_list(
                &test_node,
                None,
                &test_alive_list,
                &test_suspected_list,
                &test_confirmed_list,
            )
            .await;

        assert!(!test_ping_req.is_empty());

        let (
            test_from_list_message,
            test_from_list_node,
            test_from_list_suspect,
            test_from_list_alive_list,
            test_from_list_suspected_list,
            test_from_list_confirmed_list,
        ) = Message::from_list(&test_ping_req).await?;

        assert_eq!(test_from_list_message, Message::PingReq);
        assert_eq!(test_from_list_node.id.get_version_num(), 4);
        assert!(test_from_list_suspect.is_none());
        assert_eq!(test_from_list_alive_list.len(), 1);
        assert_eq!(test_from_list_suspected_list.len(), 1);
        assert_eq!(test_from_list_confirmed_list.len(), 1);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[should_panic]
    async fn from_list_panic() {
        let bad_message_data = b"bad_data";

        let (
            _test_from_list_message,
            _test_from_list_node,
            _test_from_list_suspect,
            _test_from_list_alive_list,
            _test_from_list_suspected_list,
            _test_from_list_confirmed_list,
        ) = Message::from_list(bad_message_data).await.unwrap();
    }
}
