use std::net::Ipv4Addr;
use uuid::Uuid;

pub struct Node {
    id: Uuid,
    address: Ipv4Addr,
    port: u16,
}

impl Node {
    pub async fn init(address: Ipv4Addr, port: u16) -> Result<Node, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4();

        Ok(Node { id, address, port })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_node_address = std::net::Ipv4Addr::new(0, 0, 0, 0);
        let test_node = Node::init(test_node_address, 10000).await?;
        assert_eq!(test_node.id.get_version_num(), 4);
        assert_eq!(test_node.address.to_string().as_str(), "0.0.0.0");
        assert_eq!(test_node.port, 10000);
        Ok(())
    }
}
