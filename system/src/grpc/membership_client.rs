use crate::grpc::{Channel, Response, Status};
pub use crate::meridian_membership_v010::communications_client::CommunicationsClient;
use crate::meridian_membership_v010::{Empty, MembershipNode, NodeStatus, Nodes};

pub struct ExternalMembershipGrpcClient {
    transport: CommunicationsClient<Channel>,
}

impl ExternalMembershipGrpcClient {
    pub async fn init(
        endpoint: String,
    ) -> Result<ExternalMembershipGrpcClient, Box<dyn std::error::Error>> {
        let transport = CommunicationsClient::connect(endpoint).await?;

        Ok(ExternalMembershipGrpcClient { transport })
    }

    pub async fn join_cluster(
        &mut self,
        request: MembershipNode,
    ) -> Result<Response<MembershipNode>, Status> {
        let mut transport = self.transport.clone();
        let response = transport.join_cluster(request).await?;

        Ok(response)
    }

    pub async fn get_nodes(&mut self) -> Result<Response<Nodes>, Status> {
        let mut transport = self.transport.clone();
        let request = Empty {};
        let response = transport.get_nodes(request).await?;

        Ok(response)
    }

    pub async fn get_node_status(&mut self) -> Result<Response<NodeStatus>, Status> {
        let mut transport = self.transport.clone();
        let request = Empty {};
        let response = transport.get_node_status(request).await?;

        Ok(response)
    }
}
