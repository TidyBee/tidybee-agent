use crate::{configuration::HubConfig, error::HubError::*, http::grpc::GrpcClient};
use anyhow::{bail, Error};
use gethostname::gethostname;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use std::env;
use tracing::info;

pub struct Hub {
    config: HubConfig,
    http_client: Client,
    pub grpc_client: GrpcClient,
}

impl Hub {
    pub fn new(hub_config: HubConfig) -> Result<Self, Error> {
        let http_client: Client = Client::new();
        let grpc_client = match GrpcClient::new(&hub_config.grpc_server) {
            Ok(client) => client,
            Err(e) => {
                bail!(HubClientCreationFailed(e.to_string()))
            }
        };
        Ok(Self {
            config: hub_config,
            http_client,
            grpc_client,
        })
    }

    pub async fn connect(&mut self) -> Result<String, Error> {
        let agent_uuid = env::var("AGENT_UUID");
        let base_url = format!(
            "{}://{}:{}",
            self.config.protocol, self.config.host, self.config.port
        );

        let url = match agent_uuid {
            Ok(uuid) => {
                format!("{}{}/{}", base_url, self.config.auth_path, uuid)
            }
            Err(_) => {
                format!("{}{}", base_url, self.config.auth_path)
            }
        };

        let agent_connection_data = format!(
            r#"
            {{
                "Metadata": {{}},
                "ConnectionModel": {{
                    "address": "{}",
                    "port": "8111"
                }}
            }}"#,
            gethostname().to_str().unwrap(),
        );

        let mut tries = 0;
        while tries < self.config.connection_attempt_limit {
            let response = self
                .http_client
                .post(&url)
                .header(CONTENT_TYPE, "application/json")
                .json(&agent_connection_data)
                .send()
                .await;
            match response {
                Ok(response) => {
                    if response.status().is_success() {
                        return match response.text().await {
                            Ok(mut text) => {
                                text = text.trim_matches('"').to_string();
                                info!(
                                    "Successfully connected the agent to the Hub with id: {}",
                                    text
                                );
                                env::set_var("AGENT_UUID", &text);

                                self.grpc_client.set_agent_uuid(&text);
                                self.grpc_client.connect().await;
                                Ok(text)
                            }
                            Err(err) => {
                                bail!(HttpError(err))
                            }
                        };
                    }
                }
                Err(e) => {
                    bail!(UnExpectedError(e.to_string()))
                }
            }
            tries += 1;
        }
        bail!(MaximumAttemptsReached())
    }
}
