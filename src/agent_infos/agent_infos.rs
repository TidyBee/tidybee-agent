use std::ffi::OsString;
use std::fmt;
use gethostname::gethostname;
use log::info;
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
        let latest_version = "0.0.1".to_string();
        let minimal_version = "0.0.0".to_string();

        AgentVersion {
            latest_version, minimal_version
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct MachineName {
    name: OsString,
}

impl fmt::Display for MachineName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name.to_string_lossy())
    }
}

#[derive(Debug, Default)]
pub struct AgentInfos {
    agent_version: AgentVersion,
    machine_name: MachineName,
    process_id: u32,
    uptime: u64,
    watched_directories: Vec<String>,
}

#[derive(Default)]
pub struct AgentInfosBuilder {
    system: System,
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

    // fn agent_version(mut self, version: String) -> Self {
    //     self.agent_version = AgentVersion { version };
    //     self
    // }
    //
    // fn machine_name(mut self) -> Self {
    //     self.machine_name = MachineName { name: gethostname() };
    //     self
    // }
    //
    // fn process_id(mut self, pid: u32) -> Self {
    //     self.process_id = ProcessId { pid };
    //     self
    // }
    //
    // fn process_id_from_system(mut self) -> Self {
    //     if let Ok(process) = sysinfo::get_current_pid() {
    //         self.process_id = ProcessId { pid: process.as_u32() };
    //     }
    //     self
    // }
    //
    // fn uptime(mut self, time: u64) -> Self {
    //     self.uptime = Uptime { time };
    //     self
    // }
    //
    // fn uptime_from_system(mut self) -> Self {
    //     let system = System::new();
    //
    //     self.uptime = Uptime {
    //         time: system.uptime(),
    //     };
    //     self
    // }
    //
    // fn watched_directories(mut self, dirs: Vec<&str>) -> Self {
    //     self.watched_directories = WatchedDirectories {
    //         directories: dirs.iter().map(|s| s.to_string()).collect(),
    //     };
    //     self
    // }
    //



    //pub struct AgentInfos {
    //     agent_version: AgentVersion,
    //     machine_name: MachineName,
    //     process_id: ProcessId,
    //     uptime: Uptime,
    //     watched_directories: WatchedDirectories,
    //     configuration_directory: ConfigurationDirectory,
    // }
    pub fn build(self) -> AgentInfos {
        let agent_version: AgentVersion = self.configuration_wrapper
            .bind::<AgentVersion>("agent_config")
            .unwrap_or_default();
        let process = sysinfo::get_current_pid();
        let mut dirs: Vec<String> = vec![];

        dirs.push("src".to_string());
        AgentInfos {
            agent_version,
            machine_name: MachineName{
                name: gethostname()
            },
            process_id: process.unwrap().as_u32(),
            uptime: self.system.uptime(),
            watched_directories: dirs,
        }
    }
}

impl AgentInfos {
    pub async fn dump(self) {
        info!("Voici le status de l'agent et ses configurations :");
        info!("Latest version : {}", self.agent_version.latest_version);
        info!("Minimal version : {}", self.agent_version.minimal_version);
        info!("Machine name : {}", self.machine_name);
    }

    pub async fn refresh(self) {
    }
}
