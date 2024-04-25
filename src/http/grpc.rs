use std::str::FromStr;

use crate::{configuration::GrpcServerConfig, file_info::FileInfo};
use tidybee_events::tidy_bee_events_client::TidyBeeEventsClient;
use tonic::{
    metadata::MetadataValue,
    service::Interceptor,
    transport::{Channel, Endpoint},
    Request, Status,
};
use tracing::{debug, info};

pub mod tidybee_events {
    tonic::include_proto!("tidybee_events");
}

// region: --- Interceptors

struct AuthInterceptor {
    agent_uuid: String,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<Request<()>, Status> {
        debug!(
            "Adding authorization header to gRPC request: {}",
            self.agent_uuid
        );

        match MetadataValue::from_str(format!("Bearer {}", self.agent_uuid).as_str()) {
            Ok(auth_header) => {
                request.metadata_mut().insert("authorization", auth_header);
                Ok(request)
            }
            Err(e) => {
                return Err(Status::internal(format!(
                    "Failed to create authorization header: {}",
                    e
                )));
            }
        }
    }
}

// endregion: --- Interceptors
#[allow(private_interfaces)]
pub struct GrpcClient {
    pub client: Option<
        TidyBeeEventsClient<
            tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>,
        >,
    >,
    agent_uuid: Option<String>,
    endpoint: Endpoint,
}

impl GrpcClient {
    pub fn new(grpc_server_config: GrpcServerConfig) -> Self {
        let endpoint = Channel::from_shared(format!(
            "{}://{}:{}",
            grpc_server_config.protocol, grpc_server_config.host, grpc_server_config.port
        ))
        .expect("Failed to create endpoint"); // TODO: Handle error

        Self {
            client: None,
            agent_uuid: None,
            endpoint,
        }
    }

    #[inline]
    pub fn set_agent_uuid(&mut self, agent_uuid: &String) {
        info!("Setting agent uuid for gRPC client: {}", agent_uuid);
        self.agent_uuid = Some(agent_uuid.clone());
    }

    // Connect before setting interceptors !
    pub async fn connect(&mut self) {
        if self.agent_uuid.is_none() {
            panic!("Agent UUID is not set for gRPC client");
            // TODO handle error
        }

        let channel = match self.endpoint.connect().await {
            Ok(channel) => channel,
            Err(e) => {
                // TODO: Manage connection failure
                panic!("Failed to connect to gRPC server: {}", e);
            }
        };
        let interceptor = AuthInterceptor {
            agent_uuid: self.agent_uuid.clone().unwrap(),
        };
        self.client = Some(TidyBeeEventsClient::with_interceptor(channel, interceptor));
    }
}

// region: --- FileInfo to gRPC message request conversions

impl From<FileInfo> for tidybee_events::FileInfoCreateRequest {
    fn from(file_info: FileInfo) -> Self {
        Self {
            pretty_path: file_info.pretty_path.display().to_string(),
            path: file_info.path.display().to_string(),
            size: file_info.size,
            hash: file_info.hash.clone().unwrap(),
            last_modified: Some(prost_types::Timestamp::from(file_info.last_modified)), // Since the type is `google.protobuf.Timestamp` whih is a nested type, protobuf will render the field as optional by default. This is not modifiable.
            last_accessed: Some(prost_types::Timestamp::from(file_info.last_accessed)), // (see https://github.com/protocolbuffers/protobuf/issues/249)
        }
    }
}

impl From<FileInfo> for tidybee_events::FileInfoUpdateRequest {
    fn from(file_info: FileInfo) -> Self {
        Self {
            pretty_path: file_info.pretty_path.display().to_string(),
            path: file_info.path.display().to_string(),
            size: Some(file_info.size),
            hash: Some(file_info.hash.clone().unwrap()),
            last_modified: Some(prost_types::Timestamp::from(file_info.last_modified)),
            last_accessed: Some(prost_types::Timestamp::from(file_info.last_accessed)),
        }
    }
}

impl From<FileInfo> for tidybee_events::FileInfoDeleteRequest {
    fn from(file_info: FileInfo) -> Self {
        Self {
            path: file_info.path.display().to_string(),
            pretty_path: file_info.pretty_path.display().to_string(),
        }
    }
}

// endregion: --- FileInfo to gRPC message request conversions
