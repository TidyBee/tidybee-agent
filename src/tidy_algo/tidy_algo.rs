use crate::file_info::{FileInfo, TidyScore};
use crate::my_files::MyFiles;
use crate::tidy_algo::tidy_rules::duplicated::duplicated;
use crate::tidy_algo::tidy_rules::misnamed::misnamed;
use crate::tidy_algo::tidy_rules::perished::perished;
use config::{Config, ConfigError, File, Value};
use std::collections::HashMap;
use std::path;
use tracing::debug;

/// Represents a rule that can be applied to a file
#[allow(dead_code)]
pub struct TidyRule {
    name: String,
    log: String,
    scope: String,
    pub params: HashMap<String, Value>,
    apply: fn(&FileInfo, &MyFiles, HashMap<String, Value>) -> TidyScore,
}

impl TidyRule {
    pub fn new(
        name: String,
        log: String,
        scope: String,
        params: HashMap<String, Value>,
        apply: fn(&FileInfo, &MyFiles, HashMap<String, Value>) -> TidyScore,
    ) -> Self {
        Self {
            name,
            log,
            scope,
            params,
            apply,
        }
    }
}

impl PartialEq for TidyRule {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct TidyAlgo {
    rules: Vec<Box<TidyRule>>,
}

impl TidyAlgo {
    pub const fn new() -> Self {
        Self { rules: Vec::new() }
    }

    fn add_rule(&mut self, rule: TidyRule) {
        self.rules.push(Box::new(rule));
    }

    pub fn load_rules_from_file(&mut self, _my_files: &MyFiles, path: path::PathBuf) {
        let rules_config: Result<Config, ConfigError> =
            Config::builder().add_source(File::from(path)).build();

        let rules = match rules_config {
            Ok(config) => config.get_array("rules").unwrap(),
            Err(error) => panic!("Error while loading rules: {error}"),
        };

        for rule in rules {
            let rule = rule.into_table().unwrap();
            let name = rule.get("name").unwrap().clone().into_string().unwrap();
            let log = rule.get("log").unwrap().clone().into_string().unwrap();
            let scope = rule.get("scope").unwrap().clone().into_string().unwrap();
            let apply_type = rule.get("type").unwrap().clone().into_string().unwrap();
            let apply = match apply_type.as_str() {
                "duplicated" => duplicated,
                "misnamed" => misnamed,
                "perished" => perished,
                _ => panic!("Unknown rule"),
            };
            debug!(
                "Adding rule {} of type {} that will be logged as {}",
                name, apply_type, log
            );
            self.add_rule(TidyRule::new(name, log, scope, rule, apply));
        }
    }
}
