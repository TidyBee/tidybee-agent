use crate::configuration_wrapper::ConfigurationWrapper;
use gethostname::gethostname;
use log::info;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sysinfo::{PidExt, RefreshKind, System, SystemExt as SysInfoSystemExt};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AgentVersion {
    latest_version: String,
    minimal_version: String,
}

impl Default for AgentVersion {
    fn default() -> Self {
        let latest_version = "0.1.0".to_string();
        let minimal_version = "0.1.0".to_string();

        AgentVersion {
            latest_version,
            minimal_version,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct AgentData {
    agent_version: AgentVersion,
    machine_name: String,
    process_id: u32,
    uptime: u64,
    watched_directories: Vec<PathBuf>,
}

#[derive(Default)]
pub struct AgentDataBuilder {
    configuration_wrapper: ConfigurationWrapper,
}

impl AgentDataBuilder {
    pub fn new() -> Self {
        AgentDataBuilder::default()
    }

    pub fn configuration_wrapper(
        mut self,
        configuration_wrapper: impl Into<ConfigurationWrapper>,
    ) -> Self {
        self.configuration_wrapper = configuration_wrapper.into();
        self
    }

    pub fn build(self, directories_watch_args: Vec<PathBuf>) -> AgentData {
        let agent_version: AgentVersion = self
            .configuration_wrapper
            .bind::<AgentVersion>("agent_config")
            .unwrap_or_default();

        AgentData {
            agent_version,
            machine_name: gethostname().to_str().unwrap().to_string(),
            process_id: sysinfo::get_current_pid().unwrap().as_u32(),
            uptime: System::new_with_specifics(RefreshKind::new()).uptime(),
            watched_directories: directories_watch_args,
        }
    }
}

#[allow(dead_code)]
impl AgentData {
    pub fn dump(&self) {
        info!(
            "Agent's status and his configuration below :\n
        Latest version : {}
        Minimal version : {}
        Machine name : {:?}
        Pid : {:?}
        Up time : {:?}
        Watched directories : {:?}
        ",
            self.agent_version.latest_version,
            self.agent_version.minimal_version,
            self.machine_name,
            self.process_id,
            self.uptime,
            self.watched_directories
        );
    }

    pub fn update(&mut self) {
        self.uptime = System::new_with_specifics(RefreshKind::new()).uptime();
    }
}