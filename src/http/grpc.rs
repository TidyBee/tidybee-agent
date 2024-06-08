use self::tidybee_events::{FileEventRequest, FileEventType};
use crate::{
    configuration::GrpcServerConfig,
    error::GrpcClientError::*,
    file_info::{self, FileInfo},
};

use anyhow::{bail, ensure, Result};
use notify::event::ModifyKind;
use notify_debouncer_full::DebouncedEvent;
use std::str::FromStr;
use tidybee_events::tidy_bee_events_client::TidyBeeEventsClient;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tonic::{
    metadata::MetadataValue,
    service::Interceptor,
    transport::{Channel, Endpoint},
    Request, Status,
};
use tracing::{debug, info, warn};

pub mod tidybee_events {
    tonic::include_proto!("tidybee_events");
}

// region: --- Interceptors

pub struct AuthInterceptor {
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
    pub fn new(grpc_server_config: &GrpcServerConfig) -> Result<Self> {
        match Channel::from_shared(format!(
            "{}://{}:{}",
            grpc_server_config.protocol, grpc_server_config.host, grpc_server_config.port
        )) {
            Ok(endpoint) => Ok(Self {
                client: None,
                agent_uuid: None,
                endpoint,
            }),
            Err(e) => bail!(e),
        }
    }

    #[inline]
    pub fn set_agent_uuid(&mut self, agent_uuid: &String) {
        info!("Setting agent uuid for gRPC client: {}", agent_uuid);
        self.agent_uuid = Some(agent_uuid.clone());
    }

    // Connect before setting interceptors !
    pub async fn connect(&mut self) -> Result<()> {
        ensure!(self.agent_uuid.is_some(), AgentUuidNotSet());

        let channel = match self.endpoint.connect().await {
            Ok(channel) => channel,
            Err(e) => {
                bail!(InvalidEndpoint(e));
            }
        };
        info!("Connected to gRPC server");
        let interceptor = AuthInterceptor {
            agent_uuid: self.agent_uuid.clone().unwrap(),
        };
        self.client = Some(TidyBeeEventsClient::with_interceptor(channel, interceptor));
        Ok(())
    }

    pub async fn send_create_events_once(&mut self, events: Vec<FileInfo>) {
        if self.client.is_none() {
            panic!("gRPC client is not connected");
            // TODO handle error
        }
        let stream = tokio_stream::iter(events.into_iter().map(|f| FileEventRequest {
            event_type: FileEventType::Created as i32,
            pretty_path: f.pretty_path.display().to_string(),
            path: vec![f.path.display().to_string()],
            size: Some(f.size),
            hash: f.hash,
            last_accessed: Some(f.last_accessed.into()),
            last_modified: Some(f.last_modified.into()),
        }));
        let _ = self.client.as_mut().unwrap().file_event(stream).await;
    }

    pub async fn send_events(&mut self, file_watcher_receiver: UnboundedReceiver<DebouncedEvent>) {
        if self.client.is_none() {
            panic!("gRPC client is not connected");
            // TODO handle error
        }
        let stream: UnboundedReceiverStream<DebouncedEvent> =
            UnboundedReceiverStream::new(file_watcher_receiver);

        let manip = stream.filter_map(map_notify_events_to_grpc);

        if self
            .client
            .as_mut()
            .unwrap()
            .file_event(manip)
            .await
            .is_err()
        {
            warn!("Failed to send file event to gRPC server");
        }
    }
}

fn map_modify_notify_events_to_grpc(
    modify_kind: ModifyKind,
    file_event: DebouncedEvent,
) -> Option<FileEventRequest> {
    match modify_kind {
        ModifyKind::Name(_) => Some(FileEventRequest {
            event_type: FileEventType::Moved as i32,
            pretty_path: file_event.paths[1].display().to_string(),
            last_modified: Some(std::time::SystemTime::now().into()),
            path: vec![
                file_info::fix_canonicalize_path(&file_event.paths[0])
                    .display()
                    .to_string(),
                file_info::fix_canonicalize_path(&file_event.paths[1])
                    .display()
                    .to_string(),
            ],
            ..Default::default()
        }),
        _ => {
            let info = file_info::create_file_info(&file_event.paths[0].clone());

            match info {
                Some(info) => Some(FileEventRequest {
                    event_type: FileEventType::Updated as i32,
                    pretty_path: info.pretty_path.display().to_string(),
                    path: vec![info.path.display().to_string()],
                    size: Some(info.size),
                    hash: info.hash,
                    last_accessed: Some(info.last_accessed.into()),
                    last_modified: Some(info.last_modified.into()),
                }),
                None => None,
            }
        }
    }
}

pub fn map_notify_events_to_grpc(file_event: DebouncedEvent) -> Option<FileEventRequest> {
    match file_event.kind {
        notify::EventKind::Create(_) => {
            let info = file_info::create_file_info(&file_event.paths[0].clone())?;
            Some(FileEventRequest {
                event_type: FileEventType::Created as i32,
                pretty_path: info.pretty_path.display().to_string(),
                path: vec![info.path.display().to_string()],
                size: Some(info.size),
                hash: info.hash,
                last_accessed: Some(info.last_accessed.into()),
                last_modified: Some(info.last_modified.into()),
            })
        }
        notify::EventKind::Modify(modify_kind) => {
            map_modify_notify_events_to_grpc(modify_kind, file_event)
        }
        notify::EventKind::Remove(_) => Some(FileEventRequest {
            event_type: FileEventType::Deleted as i32,
            pretty_path: file_event.paths[0].display().to_string(),
            path: vec![file_info::fix_canonicalize_path(&file_event.paths[0])
                .display()
                .to_string()],
            ..Default::default()
        }),
        _ => None,
    }
}
