use config::{Config, ConfigError};

#[derive(Debug, Clone, Default)]
pub struct ConfigurationWrapper {
    config: Config,
}

impl ConfigurationWrapper {
    pub fn bind<T>(&self, key: &str) -> Result<T, ConfigError>
    where
        T: serde::de::DeserializeOwned,
    {
        self.config.get::<T>(key)
    }
}
