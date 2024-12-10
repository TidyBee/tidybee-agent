use self::tidybee_events::{FileEventRequest, FileEventType};
use crate::{
    configuration::GrpcServerConfig,
    error::GrpcClientError,
    file_info::{self, FileInfo},
    file_lister,
};

use anyhow::{bail, ensure, Error, Result};
use notify::event::ModifyKind;
use notify_debouncer_full::DebouncedEvent;
use std::{str::FromStr, vec};
use tidybee_events::{tidy_bee_events_client::TidyBeeEventsClient, FolderEventRequest};
use tokio::sync::mpsc::UnboundedReceiver;
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
            Err(e) => Err(Status::internal(format!(
                "Failed to create authorization header: {}",
                e
            ))),
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
        ensure!(
            self.agent_uuid.is_some(),
            GrpcClientError::AgentUuidNotSet()
        );
        let channel = match self.endpoint.connect().await {
            Ok(channel) => channel,
            Err(e) => {
                bail!(GrpcClientError::InvalidEndpoint(e));
            }
        };
        info!("Connected to gRPC server");
        let interceptor = AuthInterceptor {
            agent_uuid: self.agent_uuid.clone().unwrap(),
        };
        self.client = Some(TidyBeeEventsClient::with_interceptor(channel, interceptor));
        Ok(())
    }

    pub async fn send_create_events_once(
        &mut self,
        events: Vec<FileInfo>,
    ) -> Result<(), GrpcClientError> {
        if self.client.is_none() {
            return Err(GrpcClientError::ClientNotConnected());
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
        Ok(())
    }

    pub async fn send_events(
        &mut self,
        mut file_watcher_receiver: UnboundedReceiver<DebouncedEvent>,
    ) -> Result<(), Error> {
        if self.client.is_none() {
            bail!(GrpcClientError::ClientNotConnected());
        }

        while let Some(file_event) = file_watcher_receiver.recv().await {
            if file_event.kind
                == notify::event::EventKind::Access(notify::event::AccessKind::Open(
                    notify::event::AccessMode::Any,
                ))
            {
                continue;
            }
            println!("{:?}", file_event);
            match file_event.kind {
                notify::EventKind::Create(notify::event::CreateKind::File) => {
                    let info = match file_info::create_file_info(&file_event.paths[0].clone()) {
                        Some(info) => info,
                        None => continue,
                    };
                    let event = FileEventRequest {
                        event_type: FileEventType::Created as i32,
                        pretty_path: info.pretty_path.display().to_string(),
                        path: vec![info.path.display().to_string()],
                        size: Some(info.size),
                        hash: info.hash,
                        last_accessed: Some(info.last_accessed.into()),
                        last_modified: Some(info.last_modified.into()),
                    };
                    if self
                        .client
                        .as_mut()
                        .unwrap()
                        .file_event(tokio_stream::iter(vec![event]))
                        .await
                        .is_err()
                    {
                        warn!("Failed to send file event to gRPC server");
                    }
                }
                notify::EventKind::Modify(modify_kind) => {
                    self.handle_modify_events(modify_kind, file_event).await?
                }
                notify::EventKind::Remove(remove_kind) => {
                    self.handle_remove_events(remove_kind, file_event).await?
                }
                _ => (),
            };
        }

        Ok(())
    }

    // region: --- event handlers

    async fn handle_modify_events(
        &mut self,
        modify_kind: notify::event::ModifyKind,
        file_event: DebouncedEvent,
    ) -> Result<(), Error> {
        match modify_kind {
            ModifyKind::Data(_) => {
                let info = match file_info::create_file_info(&file_event.paths[0].clone()) {
                    Some(info) => info,
                    None => bail!(GrpcClientError::FileInfoError()),
                };
                let event = FileEventRequest {
                    event_type: FileEventType::Created as i32,
                    pretty_path: info.pretty_path.display().to_string(),
                    path: vec![info.path.display().to_string()],
                    size: Some(info.size),
                    hash: info.hash,
                    last_accessed: Some(info.last_accessed.into()),
                    last_modified: Some(info.last_modified.into()),
                };
                if self
                    .client
                    .as_mut()
                    .unwrap()
                    .file_event(tokio_stream::iter(vec![event]))
                    .await
                    .is_err()
                {
                    warn!("Failed to send file event to gRPC server");
                    bail!(GrpcClientError::EventSendError());
                }
            }
            // The ModifyKind::Name documentation is a bit unprecise, notify::event::RenameMode::To represent a new file or folder that was moved in the scope of the watcher
            ModifyKind::Name(notify::event::RenameMode::To) => {
                if file_event.paths[0].is_dir() {
                    match file_lister::list_directories(vec![file_event.paths[0].clone()]) {
                        Ok(file_info_vec) => {
                            let events = file_info_vec.into_iter().map(|f| FileEventRequest {
                                event_type: FileEventType::Created as i32,
                                pretty_path: f.pretty_path.display().to_string(),
                                path: vec![f.path.display().to_string()],
                                size: Some(f.size),
                                hash: f.hash,
                                last_accessed: Some(f.last_accessed.into()),
                                last_modified: Some(f.last_modified.into()),
                            });
                            if self
                                .client
                                .as_mut()
                                .unwrap()
                                .file_event(tokio_stream::iter(events))
                                .await
                                .is_err()
                            {
                                warn!("Failed to send file event to gRPC server");
                                bail!(GrpcClientError::EventSendError());
                            }
                        }
                        Err(e) => {
                            warn!("Failed to list directory: {:?}", e);
                            bail!(GrpcClientError::FileInfoError());
                        }
                    }
                } else {
                    let info = match file_info::create_file_info(&file_event.paths[0].clone()) {
                        Some(info) => info,
                        None => bail!(GrpcClientError::FileInfoError()),
                    };
                    let event = FileEventRequest {
                        event_type: FileEventType::Created as i32,
                        pretty_path: info.pretty_path.display().to_string(),
                        path: vec![info.path.display().to_string()],
                        size: Some(info.size),
                        hash: info.hash,
                        last_accessed: Some(info.last_accessed.into()),
                        last_modified: Some(info.last_modified.into()),
                    };
                    if self
                        .client
                        .as_mut()
                        .unwrap()
                        .file_event(tokio_stream::iter(vec![event]))
                        .await
                        .is_err()
                    {
                        warn!("Failed to send file event to gRPC server");
                        bail!(GrpcClientError::EventSendError());
                    }
                }
            }
            // The ModifyKind::Name documentation is a bit unprecise, notify::event::RenameMode::From represent a file or folder that was moved out of the scope of the watcher
            // Thus files associated with this event should be deleted from the database
            ModifyKind::Name(notify::event::RenameMode::From) => {
                if file_event.paths[0].is_dir() {
                    let event = FolderEventRequest {
                        event_type: FileEventType::Deleted as i32,
                        old_path: file_event.paths[0].display().to_string(),
                        new_path: None,
                    };
                    if self
                        .client
                        .as_mut()
                        .unwrap()
                        .folder_event(tokio_stream::iter(vec![event]))
                        .await
                        .is_err()
                    {
                        warn!("Failed to send file event to gRPC server");
                        bail!(GrpcClientError::EventSendError());
                    }
                } else {
                    let event = FileEventRequest {
                        event_type: FileEventType::Deleted as i32,
                        pretty_path: file_event.paths[0].display().to_string(),
                        path: vec![file_event.paths[0].display().to_string()],
                        size: None,
                        hash: None,
                        last_accessed: None,
                        last_modified: None,
                    };
                    if self
                        .client
                        .as_mut()
                        .unwrap()
                        .file_event(tokio_stream::iter(vec![event]))
                        .await
                        .is_err()
                    {
                        warn!("Failed to send file event to gRPC server");
                        bail!(GrpcClientError::EventSendError());
                    }
                }
            }
            // In this case, the object was actually renamed, so we can use the Moved event type
            ModifyKind::Name(notify::event::RenameMode::Both) => {
                if file_event.paths[0].is_dir() {
                    let event = FolderEventRequest {
                        event_type: FileEventType::Moved as i32,
                        old_path: file_event.paths[0].display().to_string(),
                        new_path: Some(file_event.paths[1].display().to_string()),
                    };
                    if self
                        .client
                        .as_mut()
                        .unwrap()
                        .folder_event(tokio_stream::iter(vec![event]))
                        .await
                        .is_err()
                    {
                        warn!("Failed to send file event to gRPC server");
                        bail!(GrpcClientError::EventSendError());
                    }
                } else {
                    let info = match file_info::create_file_info(&file_event.paths[0].clone()) {
                        Some(info) => info,
                        None => bail!(GrpcClientError::FileInfoError()),
                    };
                    let event = FileEventRequest {
                        event_type: FileEventType::Created as i32,
                        pretty_path: info.pretty_path.display().to_string(),
                        path: vec![info.path.display().to_string()],
                        size: Some(info.size),
                        hash: info.hash,
                        last_accessed: Some(info.last_accessed.into()),
                        last_modified: Some(info.last_modified.into()),
                    };
                    if self
                        .client
                        .as_mut()
                        .unwrap()
                        .file_event(tokio_stream::iter(vec![event]))
                        .await
                        .is_err()
                    {
                        warn!("Failed to send file event to gRPC server");
                        bail!(GrpcClientError::EventSendError());
                    }
                }
            }
            _ => (),
        };
        Ok(())
    }

    async fn handle_remove_events(
        &mut self,
        remove_kind: notify::event::RemoveKind,
        file_event: DebouncedEvent,
    ) -> Result<(), Error> {
        match remove_kind {
            notify::event::RemoveKind::File => {
                let event = FileEventRequest {
                    event_type: FileEventType::Deleted as i32,
                    pretty_path: file_event.paths[0].display().to_string(),
                    path: vec![file_event.paths[0].display().to_string()],
                    size: None,
                    hash: None,
                    last_accessed: None,
                    last_modified: None,
                };
                if self
                    .client
                    .as_mut()
                    .unwrap()
                    .file_event(tokio_stream::iter(vec![event]))
                    .await
                    .is_err()
                {
                    warn!("Failed to send file event to gRPC server");
                    bail!(GrpcClientError::EventSendError());
                }

                Ok(())
            }
            notify::event::RemoveKind::Folder => {
                let event = FolderEventRequest {
                    event_type: FileEventType::Deleted as i32,
                    old_path: file_event.paths[0].display().to_string(),
                    new_path: None,
                };
                if self
                    .client
                    .as_mut()
                    .unwrap()
                    .folder_event(tokio_stream::iter(vec![event]))
                    .await
                    .is_err()
                {
                    warn!("Failed to send file event to gRPC server");
                    bail!(GrpcClientError::EventSendError());
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    // endregion: --- event handlers
}
