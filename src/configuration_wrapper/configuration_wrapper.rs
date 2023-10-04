use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Clone)]
pub struct ConfigurationWrapper {
    config: Config,
}

impl ConfigurationWrapper {
    pub fn new() -> Result<Self, ConfigError> {
        let env_name = std::env::var("ENV_NAME").unwrap_or_else(|_| "development".into());

        let config: Result<Config, ConfigError> = Config::builder()
            .add_source(File::with_name("config/default.json"))
            .add_source(File::with_name(&format!("config/{}.json", env_name)).required(false))
            .add_source(Environment::with_prefix("TIDYBEE"))
            .build();

        match config {
            Ok(config) => Ok(ConfigurationWrapper { config }),
            Err(error) => Err(error),
        }
    }

    pub fn bind<T>(&self, key: &str) -> Result<T, ConfigError>
    where
        T: serde::de::DeserializeOwned,
    {
        self.config.get::<T>(key)
    }
}
