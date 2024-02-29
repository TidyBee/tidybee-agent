use crate::configuration::HubConfig;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use std::env;
use tracing::{debug, info, warn};
use anyhow::Error;

const MAX_RETRIES: u32 = 30;

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

    pub async fn connect(&self) -> Result<(), Error> {
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

        let mut tries = 0;
        while tries < MAX_RETRIES {
            let response = self
                .http_client
                .post(&url)
                .header(CONTENT_TYPE, "application/json")
                .json("{}")
                .send()
                .await;
            match response {
                Ok(response) => {
                    if response.status().is_success() {
                        return match response.text().await {
                            Ok(text) => {
                                info!("Successfully connected the agent to the Hub with id: {}", text);
                                env::set_var("AGENT_UUID", text);
                                Ok(())
                            }
                            Err(err) => {
                                warn!("Parsing error : {}", err);
                                Err(Error::msg("Failed to parse response from Hub when authenticating."))
                            }
                        }

                    }
                }
                Err(e) => {
                    warn!("Error connecting to hub: {:?}", e);
                }
            }
            tries += 1;
        }
        Err(Error::msg("Maximum number of retries reached without success."))
    }
}
