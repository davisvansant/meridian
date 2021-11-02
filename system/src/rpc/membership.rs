pub struct MembershipNode {
    pub id: String,
    pub address: String,
    pub client_port: String,
    pub cluster_port: String,
    pub membership_port: String,
}

impl MembershipNode {
    pub async fn build() -> Result<MembershipNode, Box<dyn std::error::Error>> {
        let id = String::from("some_node_id");
        let address = String::from("some_node_address");
        let client_port = String::from("some_client_port");
        let cluster_port = String::from("some_cluster_port");
        let membership_port = String::from("some_membership_port");

        Ok(MembershipNode {
            id,
            address,
            client_port,
            cluster_port,
            membership_port,
        })
    }
}

pub struct Connected {
    nodes: Vec<MembershipNode>,
}

impl Connected {
    pub async fn build() -> Result<Connected, Box<dyn std::error::Error>> {
        let nodes = Vec::with_capacity(5);

        Ok(Connected { nodes })
    }
}

pub struct Status {
    details: String,
}

impl Status {
    pub async fn build() -> Result<Status, Box<dyn std::error::Error>> {
        let details = String::from("some_node_status");

        Ok(Status { details })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn membership_node() -> Result<(), Box<dyn std::error::Error>> {
        let test_membership_node = MembershipNode::build().await?;

        assert_eq!(test_membership_node.id, "some_node_id");
        assert_eq!(test_membership_node.address, "some_node_address");
        assert_eq!(test_membership_node.client_port, "some_client_port");
        assert_eq!(test_membership_node.cluster_port, "some_cluster_port");
        assert_eq!(test_membership_node.membership_port, "some_membership_port");

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn connected() -> Result<(), Box<dyn std::error::Error>> {
        let test_connected = Connected::build().await?;

        assert_eq!(test_connected.nodes.len(), 0);
        assert_eq!(test_connected.nodes.capacity(), 5);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn status() -> Result<(), Box<dyn std::error::Error>> {
        let test_status = Status::build().await?;

        assert_eq!(test_status.details, "some_node_status");

        Ok(())
    }
}
