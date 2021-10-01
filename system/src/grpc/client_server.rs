use crate::grpc::{Request, Response, Status};
pub use crate::meridian_client_v010::communications_server::{
    Communications, CommunicationsServer,
};
use crate::meridian_client_v010::{HelloRequest, HelloResponse};

pub struct ExternalClientGrpcServer {}

impl ExternalClientGrpcServer {
    pub async fn init() -> Result<ExternalClientGrpcServer, Box<dyn std::error::Error>> {
        Ok(ExternalClientGrpcServer {})
    }
}

#[tonic::async_trait]
impl Communications for ExternalClientGrpcServer {
    async fn hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let response = HelloResponse {
            hello: request.into_inner().hello,
        };

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_external_client_grpc_server = ExternalClientGrpcServer::init().await;
        assert!(test_external_client_grpc_server.is_ok());
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn hello() -> Result<(), Box<dyn std::error::Error>> {
        let test_external_client_grpc_server = ExternalClientGrpcServer::init().await?;
        let test_request = Request::new(HelloRequest {
            hello: String::from("test_hello"),
        });
        let test_response = test_external_client_grpc_server.hello(test_request).await?;
        assert_eq!(test_response.get_ref().hello.as_str(), "test_hello");
        Ok(())
    }
}
