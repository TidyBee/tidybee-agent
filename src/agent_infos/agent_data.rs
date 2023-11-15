use std::ffi::OsString;
use std::path::PathBuf;
use gethostname::gethostname;
use log::{info};
use serde::{Deserialize, Serialize};
use sysinfo::{PidExt, RefreshKind, System, SystemExt as SysInfoSystemExt};
use crate::configuration_wrapper::ConfigurationWrapper;

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Default, Serialize, Clone, Copy)]
pub struct AgentData {
    // agent_version: AgentVersion,
    // pub(crate) machine_name: String,
    pub(crate) process_id: u32,
    // uptime: u64,
    // watched_directories: Vec<PathBuf>,
}

// #[derive(Debug, Default)]
// pub struct AgentDataWrapper {
//     agent_data: AgentData,
//     system: System,
//     mutex: Mutex<AgentData>
// }

#[derive(Default)]
pub struct AgentDataWrapperBuilder {
    system: System,
    configuration_wrapper: ConfigurationWrapper
}

impl AgentDataWrapperBuilder {
    pub(crate) fn new() -> Self {
        AgentDataWrapperBuilder::default()
    }

    pub fn configuration_wrapper(
        mut self,
        configuration_wrapper: impl Into<ConfigurationWrapper>
    ) -> Self {
        self.configuration_wrapper = configuration_wrapper.into();
        self
    }

    pub fn build(self, directories_watch_args: Vec<PathBuf>) -> AgentData {
        let agent_version: AgentVersion = self.configuration_wrapper
            .bind::<AgentVersion>("agent_config")
            .unwrap_or_default();

        AgentData {
            // agent_version,
            // machine_name: gethostname().to_str().unwrap().to_string(),
            process_id: sysinfo::get_current_pid().unwrap().as_u32(),
            // uptime: System::new_with_specifics(RefreshKind::new()).uptime(),
            // watched_directories: directories_watch_args,
        }
    }
}

impl AgentData {
    pub fn dump(&self) {
        // info!("Voici le status de l'agent et ses configurations :");
        // info!("Latest version : {}", self.agent_version.latest_version);
        // info!("Minimal version : {}", self.agent_version.minimal_version);
        // info!("Machine name : {:?}", self.machine_name);
        // info!("Pid : {:?}", self.process_id);
        // info!("Up time : {:?}", self.uptime);
        // info!("Les dossiers pris en compte : {:?}", self.watched_directories);
    }

    // pub fn update(&mut self) {
    //     self.uptime = System::new_with_specifics(RefreshKind::new()).uptime();
    // }
    //
    pub fn get_pid(self) -> u32 {
        self.process_id.clone()
    }

    // pub fn get_machine(self) -> String {
    //     self.machine_name.clone()
    // }
}
