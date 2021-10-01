use crate::grpc::{Channel, Response, Status};
pub use crate::meridian_membership_v010::communications_client::CommunicationsClient;
use crate::meridian_membership_v010::{JoinClusterRequest, JoinClusterResponse};

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
        request: JoinClusterRequest,
    ) -> Result<Response<JoinClusterResponse>, Status> {
        let mut transport = self.transport.clone();
        let response = transport.join_cluster(request).await?;

        Ok(response)
    }
}
