use crate::configuration::HubConfig;
use anyhow::{bail, Error};
use gethostname::gethostname;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use std::env;
use tracing::{info, warn};

pub struct Hub {
    config: HubConfig,
    http_client: Client,
}

impl Hub {
    pub fn new(hub_config: HubConfig) -> Self {
        let http_client: Client = Client::new();

        Self {
            config: hub_config,
            http_client,
        }
    }

    pub async fn connect(&self) -> Result<String, Error> {
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
                            Ok(text) => {
                                info!(
                                    "Successfully connected the agent to the Hub with id: {}",
                                    text
                                );
                                env::set_var("AGENT_UUID", &text);
                                Ok(text)
                            }
                            Err(err) => {
                                warn!("Parsing error : {}", err);
                                bail!("Failed to parse response from Hub when authenticating",)
                            }
                        };
                    }
                }
                Err(e) => {
                    warn!("Error connecting to hub: {:?}", e);
                }
            }
            tries += 1;
        }
        bail!("Maximum number of retries reached without success")
    }
}
