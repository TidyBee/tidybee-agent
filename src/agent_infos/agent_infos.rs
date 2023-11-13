use std::ffi::OsString;
use gethostname::gethostname;
use log::info;
use serde::Deserialize;
use sysinfo::{PidExt, System, SystemExt as SysInfoSystemExt};
use crate::configuration_wrapper::ConfigurationWrapper;

#[derive(Debug, Deserialize)]
struct AgentVersion {
    version: String,
}

impl Default for AgentVersion {
    fn default() -> Self {
        let version = "0.0.0".to_string();
        AgentVersion { version }
    }
}

#[derive(Debug, Default)]
struct MachineName {
    name: OsString,
}

#[derive(Debug, Default)]
struct ProcessId {
    pid: u32,
}

#[derive(Debug, Default)]
struct Uptime {
    time: u64,
}

#[derive(Debug, Default)]
struct WatchedDirectories {
    directories: Vec<String>,
}

#[derive(Debug, Default)]
struct ConfigurationDirectory {
    directory: String,
}

#[derive(Debug, Default)]
pub struct AgentInfos {
    agent_version: AgentVersion,
    machine_name: MachineName,
    process_id: ProcessId,
    uptime: Uptime,
    watched_directories: WatchedDirectories,
    configuration_directory: ConfigurationDirectory,
}

#[derive(Default)]
pub struct AgentInfosBuilder {
    agent_version: AgentVersion,
    machine_name: MachineName,
    process_id: ProcessId,
    uptime: Uptime,
    watched_directories: WatchedDirectories,
    configuration_directory: ConfigurationDirectory,
    configuration_wrapper: ConfigurationWrapper
}

// TODO add watched directories logic
// TODO add configuration_directory logic & thinking
impl AgentInfosBuilder {
    pub(crate) fn new() -> Self {
        AgentInfosBuilder::default()
    }

    pub fn configuration_wrapper(
        mut self,
        configuration_wrapper: impl Into<ConfigurationWrapper>,
    ) -> Self {
        self.configuration_wrapper = configuration_wrapper.into();
        self
    }

    fn agent_version(mut self, version: String) -> Self {
        self.agent_version = AgentVersion { version };
        self
    }

    fn machine_name(mut self) -> Self {
        self.machine_name = MachineName { name: gethostname() };
        self
    }

    fn process_id(mut self, pid: u32) -> Self {
        self.process_id = ProcessId { pid };
        self
    }

    fn process_id_from_system(mut self) -> Self {
        if let Ok(process) = sysinfo::get_current_pid() {
            self.process_id = ProcessId { pid: process.as_u32() };
        }
        self
    }

    fn uptime(mut self, time: u64) -> Self {
        self.uptime = Uptime { time };
        self
    }

    fn uptime_from_system(mut self) -> Self {
        let system = System::new();

        self.uptime = Uptime {
            time: system.uptime(),
        };
        self
    }

    fn watched_directories(mut self, dirs: Vec<&str>) -> Self {
        self.watched_directories = WatchedDirectories {
            directories: dirs.iter().map(|s| s.to_string()).collect(),
        };
        self
    }

    fn configuration_directory(mut self, config_dir: &str) -> Self {
        self.configuration_directory = ConfigurationDirectory {
            directory: config_dir.to_string(),
        };
        self
    }

    pub(crate) fn build(self) -> Result<AgentInfos, &'static str> {
        info!("This is the current configuration of the agent : ");
        info!("Agent version : {:?}", self.agent_version);
        info!("Machine name : {:?}", self.machine_name);
        Ok(AgentInfos {
            agent_version: self.configuration_wrapper.bind::<AgentVersion>("agent_config").unwrap_or_default(),
            machine_name: self.machine_name,
            process_id: self.process_id,
            uptime: self.uptime,
            watched_directories: self.watched_directories,
            configuration_directory: self.configuration_directory,
        })
    }
}
