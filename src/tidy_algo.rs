use crate::file_info::{FileInfo, TidyScore};
use crate::my_files::MyFiles;
use crate::tidy_rules::duplicated;
use crate::tidy_rules::misnamed;
use crate::tidy_rules::perished;
use config::{Config, ConfigError, File, Value};
use std::collections::HashMap;
use std::error::Error;
use std::path;
use tracing::debug;

/// Represents a rule that can be applied to a file
#[allow(dead_code)]
#[derive(Debug)]
pub struct TidyRule {
    name: String,
    log: String,
    scope: String,
    pub params: HashMap<String, Value>,
    apply: fn(&FileInfo, &MyFiles, HashMap<String, Value>) -> TidyScore,
}

impl TidyRule {
    pub const fn new(
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

    /// Load a rule into the ruleset
    ///
    /// Returns either the name of the loaded rule or an Error
    fn load_rule_from_hashmap(
        &mut self,
        table: HashMap<String, Value>,
    ) -> Result<String, Box<dyn Error>> {
        let name = get_string_from_table_safe(&table, "name")?;
        let log = get_string_from_table_safe(&table, "log")?;
        let scope = get_string_from_table_safe(&table, "scope")?;
        let apply_type = get_string_from_table_safe(&table, "type")?;
        let apply = match apply_type.as_str() {
            "duplicated" => duplicated::aply_duplicated,
            "misnamed" => misnamed::apply_misnamed,
            "perished" => perished::apply_perished,
            fallback => return Err(format!("Could not load rule with type {}", fallback).into()),
        };
        debug!(
            "Adding rule {} of type {} that will be logged as {}",
            name, apply_type, log
        );
        self.add_rule(TidyRule::new(name.clone(), log, scope, table, apply));
        Ok(name)
    }

    /// Load rules from a file
    ///
    /// Returns the number of rules loaded of an error
    pub fn load_rules_from_file(
        &mut self,
        _my_files: &MyFiles,
        path: path::PathBuf,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let rules_config: Result<Config, ConfigError> =
            Config::builder().add_source(File::from(path)).build();

        let rules = match rules_config {
            Ok(config) => config.get_array("rules").unwrap(),
            Err(error) => return Err(format!("Error while loading rules: {}", error).into()),
        };

        for rule in rules {
            let table = rule.into_table().unwrap();
            let _ = self.load_rule_from_hashmap(table);
        }
        Ok(self.rules.len())
    }
}

/// Helper function the get the key and clone the value or return an Err
fn get_string_from_table_safe(
    table: &HashMap<String, Value>,
    key: &'static str,
) -> Result<String, ConfigError> {
    match table.get(key) {
        Some(value) => Ok(value.to_string().clone()),
        None => Err(ConfigError::NotFound(format!(
            "Error while getting the key {}",
            key
        ))),
    }
}

// region: --- TidyAlgo tests
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{configuration::Configuration, my_files::MyFilesBuilder};

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        std::env::set_var("ENV_NAME", "test");
    }

    #[test]
    fn create_tidy_algo() {
        let tidy_algo = TidyAlgo::new();
        assert_eq!(tidy_algo.rules.len(), 0);
    }

    #[test]
    fn add_rule() {
        let mut tidy_algo = TidyAlgo::new();

        let new_rule = TidyRule::new(
            "dummy rule".to_string(),
            "dummy log".to_string(),
            "dummy scope".to_string(),
            HashMap::new(),
            |_, _, _| TidyScore::new(false, false, None),
        );
        tidy_algo.add_rule(new_rule);

        assert_eq!(tidy_algo.rules.len(), 1);
    }

    #[test]
    fn load_rules_from_file_basic() {
        let configuration_instance: Configuration = Configuration::init();

        let mut tidy_algo = TidyAlgo::new();
        let rules_path: PathBuf = [r"tests", r"assets", r"rules_folder", r"basic.yml"]
            .iter()
            .collect();
        let my_files = MyFilesBuilder::new()
            .configure(configuration_instance.my_files_config)
            .seal()
            .build()
            .unwrap();

        assert_eq!(
            tidy_algo
                .load_rules_from_file(&my_files, rules_path)
                .unwrap(),
            3
        );
    }

    #[test]
    fn load_rules_from_file_failing() {
        let configuration_instance: Configuration = Configuration::init();

        let mut tidy_algo = TidyAlgo::new();
        let rules_path: PathBuf = [r"tests", r"assets", r"rules_folder", r"invalid.yml"]
            .iter()
            .collect();
        let my_files = MyFilesBuilder::new()
            .configure(configuration_instance.my_files_config)
            .seal()
            .build()
            .unwrap();
        assert_eq!(
            tidy_algo
                .load_rules_from_file(&my_files, rules_path)
                .unwrap(),
            0
        );
    }
}

// endregion: --- TidyAlgo tests
