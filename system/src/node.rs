use std::net::{IpAddr, SocketAddr};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub id: Uuid,
    pub address: IpAddr,
    pub client_port: u16,
    pub cluster_port: u16,
    pub membership_port: u16,
}

impl Node {
    pub async fn init(
        address: IpAddr,
        client_port: u16,
        cluster_port: u16,
        membership_port: u16,
    ) -> Result<Node, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4();

        Ok(Node {
            id,
            address,
            client_port,
            cluster_port,
            membership_port,
        })
    }

    pub async fn build_address(&self, client_port: u16) -> SocketAddr {
        SocketAddr::new(self.address, client_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_node_address = std::net::IpAddr::from_str("0.0.0.0")?;
        let test_node = Node::init(test_node_address, 10000, 15000, 20000).await?;
        assert_eq!(test_node.id.get_version_num(), 4);
        assert_eq!(test_node.address.to_string().as_str(), "0.0.0.0");
        assert_eq!(test_node.client_port, 10000);
        assert_eq!(test_node.cluster_port, 15000);
        assert_eq!(test_node.membership_port, 20000);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn build_address() -> Result<(), Box<dyn std::error::Error>> {
        let test_node_address = std::net::IpAddr::from_str("0.0.0.0")?;
        let test_node = Node::init(test_node_address, 10000, 15000, 20000).await?;
        let test_socket_address = test_node.build_address(test_node.client_port).await;

        assert_eq!(test_socket_address.to_string().as_str(), "0.0.0.0:10000");

        Ok(())
    }
}
