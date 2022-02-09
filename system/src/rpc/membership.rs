use crate::node::Node;

pub struct MembershipNode {
    pub id: String,
    pub address: String,
    pub client_port: String,
    pub cluster_port: String,
    pub membership_port: String,
}

impl MembershipNode {
    pub async fn build(node: &Node) -> Result<MembershipNode, Box<dyn std::error::Error>> {
        let id = node.id.to_string();
        let address = node.address.to_string();
        let client_port = node.client_port.to_string();
        let cluster_port = node.cluster_port.to_string();
        let membership_port = node.membership_port.to_string();

        Ok(MembershipNode {
            id,
            address,
            client_port,
            cluster_port,
            membership_port,
        })
    }
}

// pub struct Connected {
//     pub nodes: Vec<MembershipNode>,
// }

// pub struct Status {
//     pub details: String,
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread")]
    async fn membership_node() -> Result<(), Box<dyn std::error::Error>> {
        let test_node_address = std::net::IpAddr::from_str("0.0.0.0")?;
        let test_node = Node::init(test_node_address, 10000, 15000, 20000).await?;

        let test_membership_node = MembershipNode::build(&test_node).await?;

        // assert_eq!(test_membership_node.id, "some_node_id");
        assert_eq!(test_membership_node.address, "0.0.0.0");
        assert_eq!(test_membership_node.client_port, "10000");
        assert_eq!(test_membership_node.cluster_port, "15000");
        assert_eq!(test_membership_node.membership_port, "20000");

        Ok(())
    }
}
