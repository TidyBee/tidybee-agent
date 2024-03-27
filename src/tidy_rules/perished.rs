use chrono::{DateTime, Duration, Utc};
use config::Value;
use std::collections::HashMap;
use tracing::warn;

use crate::{
    file_info::{FileInfo, TidyScore},
    my_files::MyFiles,
};

fn parse_duration(duration_str: String) -> Result<Duration, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = duration_str.split_whitespace().collect();
    match parts.len() {
        len if len >= 2 => {
            let duration: i64 = parts[0].parse()?;
            let unit = parts[1];
            let duration = match unit {
                "days" => Duration::days(duration * 24 * 60 * 60),
                "weeks" => Duration::weeks(duration * 7 * 24 * 60 * 60),
                "months" => Duration::days(duration * 30 * 24 * 60 * 60), // Approximation of 30 days per month
                "years" => Duration::days(duration * 365 * 24 * 60 * 60), // Approximation of 365 days per year
                _ => return Err("Unsupported time unit".into()),
            };
            Ok(duration)
        }
        _ => Err("No duration found".into()),
    }
}

fn calculate_expiration_date(
    expiration_duration: String,
) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    let expiration_duration = parse_duration(expiration_duration)?;
    let now = Utc::now();
    let expiration_date = now + expiration_duration;
    Ok(expiration_date)
}

pub fn apply_perished(
    file_info: &FileInfo,
    _my_files: &MyFiles,
    raw_params: HashMap<String, Value>,
) -> TidyScore {
    let mut tidy_score = file_info.tidy_score.clone();
    let expiration_duration = match raw_params.get("expiration_duration") {
        Some(s) => match s.clone().into_string() {
            Ok(s) => s,
            Err(e) => {
                warn!("Error parsing max date: {}", e);
                return TidyScore::new(false, false, None);
            }
        },
        _ => {
            warn!("No max date provided");
            return TidyScore::new(false, false, None);
        }
    };
    let expiration_date = match calculate_expiration_date(expiration_duration) {
        Ok(d) => d,
        Err(e) => {
            warn!("Error while computing date time : {}", e);
            return TidyScore::new(false, false, None);
        }
    };
    let last_accessed: DateTime<Utc> = file_info.last_accessed.into();
    let perished: bool = last_accessed < expiration_date;

    if perished {
        tracing::debug!(
            "Found a new perished file {:?} with hash {}",
            file_info.path.clone(),
            file_info.hash.clone().unwrap(),
        );
    }
    match tidy_score {
        Some(mut score) => {
            score.unused = perished;
            tidy_score = Some(score);
            tidy_score.unwrap()
        }
        None => {
            if last_accessed < expiration_date {
                TidyScore::new(false, false, None)
            } else {
                TidyScore::new(false, true, None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_day_duration() {
        assert!(parse_duration("+1 days".to_owned()).is_ok());
    }

    #[test]
    fn one_day_duration_invalid() {
        assert!(parse_duration("+1 day".to_owned()).is_err());
    }

    #[test]
    fn one_week_duration() {
        assert!(parse_duration("+1 weeks".to_owned()).is_ok());
    }

    #[test]
    fn one_month_duration() {
        assert!(parse_duration("+1 months".to_owned()).is_ok());
    }

    #[test]
    fn one_year_duration() {
        assert!(parse_duration("+1 years".to_owned()).is_ok());
    }

    #[test]
    fn invalid_unit() {
        assert!(parse_duration("+1 invalid".to_owned()).is_err());
    }

    #[test]
    fn missing_unit() {
        assert!(parse_duration("+1".to_owned()).is_err());
    }

    #[test]
    fn missing_number() {
        assert!(parse_duration("days".to_owned()).is_err());
    }

    #[test]
    fn missing_everything() {
        assert!(parse_duration("".to_owned()).is_err());
    }
}
