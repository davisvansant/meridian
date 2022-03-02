use flexbuffers::{Builder, BuilderOptions};

use crate::node::Node;

#[derive(Debug, PartialEq)]
pub enum Message {
    Ack,
    Ping,
    PingReq,
}

impl Message {
    pub async fn build(&self) -> &[u8] {
        match self {
            Message::Ack => "ack".as_bytes(),
            Message::Ping => "ping".as_bytes(),
            Message::PingReq => "ping-req".as_bytes(),
        }
    }
    pub async fn from_bytes(bytes: &[u8]) -> Message {
        match bytes {
            b"ack" => Message::Ack,
            b"ping" => Message::Ping,
            b"ping-req" => Message::PingReq,
            _ => panic!("cannot build requested bytes into message"),
        }
    }
    pub async fn build_list(&self, node: Node) -> Vec<u8> {
        let flexbuffer_options = BuilderOptions::SHARE_NONE;
        let mut flexbuffers_builder = Builder::new(flexbuffer_options);
        let mut flexbuffers_data = flexbuffers_builder.start_map();

        match self {
            Message::Ack => {
                flexbuffers_data.push("message", "ack");

                let mut node_map = flexbuffers_data.start_map("node");

                node_map.push("id", node.id.to_string().as_str());
                node_map.push("address", node.address.to_string().as_str());
                node_map.push("client_port", node.client_port);
                node_map.push("cluster_port", node.cluster_port);
                node_map.push("membership_port", node.membership_port);

                node_map.end_map();
                flexbuffers_data.end_map();

                flexbuffers_builder.take_buffer()
            }
            Message::Ping => {
                flexbuffers_data.push("message", "ping");

                let mut node_map = flexbuffers_data.start_map("node");

                node_map.push("id", node.id.to_string().as_str());
                node_map.push("address", node.address.to_string().as_str());
                node_map.push("client_port", node.client_port);
                node_map.push("cluster_port", node.cluster_port);
                node_map.push("membership_port", node.membership_port);

                node_map.end_map();
                flexbuffers_data.end_map();

                flexbuffers_builder.take_buffer()
            }
            Message::PingReq => {
                flexbuffers_data.push("message", "ping_req");

                let mut node_map = flexbuffers_data.start_map("node");

                node_map.push("id", node.id.to_string().as_str());
                node_map.push("address", node.address.to_string().as_str());
                node_map.push("client_port", node.client_port);
                node_map.push("cluster_port", node.cluster_port);
                node_map.push("membership_port", node.membership_port);

                node_map.end_map();
                flexbuffers_data.end_map();

                flexbuffers_builder.take_buffer()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn message_ack() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_ack = Message::Ack.build().await;

        assert_eq!(test_message_ack, b"ack");
        assert_eq!(test_message_ack.len(), 3);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_ping() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_ping = Message::Ping.build().await;

        assert_eq!(test_message_ping, b"ping");
        assert_eq!(test_message_ping.len(), 4);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_ping_req() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_ping_req = Message::PingReq.build().await;

        assert_eq!(test_message_ping_req, b"ping-req");
        assert_eq!(test_message_ping_req.len(), 8);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_ack_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_ack_bytes = b"ack"; //bar
        let test_message_ack = Message::from_bytes(test_ack_bytes).await;

        assert_eq!(test_message_ack, Message::Ack);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_ping_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_ping_bytes = b"ping";
        let test_message_ping = Message::from_bytes(test_ping_bytes).await;

        assert_eq!(test_message_ping, Message::Ping);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_from_ping_req_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let test_ping_req_bytes = b"ping-req";
        let test_message_ping_req = Message::from_bytes(test_ping_req_bytes).await;

        assert_eq!(test_message_ping_req, Message::PingReq);

        Ok(())
    }
}
