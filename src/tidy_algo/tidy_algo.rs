use std::path;
use config::{Config, ConfigError, File};
use log::debug;
use crate::file_info::{FileInfo, TidyScore};
use crate::tidy_algo::tidy_rules::duplicated::duplicated;
use crate::tidy_algo::tidy_rules::missnamed::missnamed;
use crate::tidy_algo::tidy_rules::perished::perished;

/// Represents a rule that can be applied to a file
pub struct TidyRule {
    name: String,
    log: String,
    scope: String,
    apply: fn(&FileInfo) -> TidyScore,
}

impl TidyRule {
    pub fn new(name: String, log: String, scope: String, apply: fn(&FileInfo) -> TidyScore) -> Self {
        Self {
            name,
            log,
            scope,
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
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    fn add_rule(&mut self, rule: TidyRule) {
        self.rules.push(Box::new(rule));
    }

    pub fn load_rules_from_file(&mut self, path: path::PathBuf) {
        let rules_config: Result<Config, ConfigError> = Config::builder()
            .add_source(File::from(path))
            .build();

        let rules = match rules_config {
            Ok(config) => config.get_array("rules").unwrap(),
            Err(error) => panic!("Error while loading rules: {}", error),
        };

        for rule in rules {
            let rule = rule.into_table().unwrap();
            let name = rule.get("name").unwrap().clone().into_string().unwrap();
            let log = rule.get("log").unwrap().clone().into_string().unwrap();
            let scope = rule.get("scope").unwrap().clone().into_string().unwrap();
            let apply_type = rule.get("type").unwrap().clone().into_string().unwrap();
            let apply = match apply_type.as_str() {
                "duplicated" => duplicated,
                "misnamed" => missnamed,
                "perished" => perished,
                _ => panic!("Unknown rule"),
            };
            debug!("Adding rule {} of type {} that will be logged as {}", name, apply_type, log);
            self.add_rule(TidyRule::new(name, log, scope, apply));
        }
    }
}