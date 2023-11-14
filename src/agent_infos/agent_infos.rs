use std::ffi::OsString;
use std::path::PathBuf;
use gethostname::gethostname;
use log::{info};
use serde::Deserialize;
use sysinfo::{PidExt, System, SystemExt as SysInfoSystemExt};
use crate::configuration_wrapper::ConfigurationWrapper;

#[derive(Debug, Deserialize)]
struct AgentVersion {
    latest_version: String,
    minimal_version: String
}

impl Default for AgentVersion {
    fn default() -> Self {
        let latest_version = "0.0.0".to_string();
        let minimal_version = "0.0.0".to_string();

        AgentVersion {
            latest_version,
            minimal_version
        }
    }
}

#[derive(Debug, Default)]
pub struct AgentInfos {
    agent_version: AgentVersion,
    machine_name: OsString,
    process_id: u32,
    uptime: u64,
    watched_directories: Vec<PathBuf>,
}

#[derive(Default)]
pub struct AgentInfosBuilder {
    system: System,
    configuration_wrapper: ConfigurationWrapper
}

impl AgentInfosBuilder {
    pub(crate) fn new() -> Self {
        AgentInfosBuilder::default()
    }

    pub fn configuration_wrapper(
        mut self,
        configuration_wrapper: impl Into<ConfigurationWrapper>
    ) -> Self {
        self.configuration_wrapper = configuration_wrapper.into();
        self
    }

    pub fn build(self, directories_watch_args: Vec<PathBuf>) -> AgentInfos {
        let agent_version: AgentVersion = self.configuration_wrapper
            .bind::<AgentVersion>("agent_config")
            .unwrap_or_default();

        AgentInfos {
            agent_version,
            machine_name: gethostname(),
            process_id: sysinfo::get_current_pid().unwrap().as_u32(),
            uptime: self.system.uptime(),
            watched_directories: directories_watch_args,
        }
    }
}

impl AgentInfos {
    pub async fn dump(&self) {
        info!("Voici le status de l'agent et ses configurations :");
        info!("Latest version : {}", self.agent_version.latest_version);
        info!("Minimal version : {}", self.agent_version.minimal_version);
        info!("Machine name : {:?}", self.machine_name);
        info!("Pid : {:?}", self.process_id);
        info!("Up time : {:?}", self.uptime);
        info!("Les dossiers pris en compte : {:?}", self.watched_directories);
    }
}
