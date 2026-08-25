pub(crate) mod issues;

pub use issues::Issues;

use anyhow::{Context, ensure};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn default_output_dir() -> PathBuf {
    PathBuf::from("./recordings")
}
fn default_retention() -> u32 {
    30
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
    pub location: Option<Location>,
    pub jobs: Vec<Job>,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("could not read config file {path:?}"))?;
        Self::from_json(&text).with_context(|| format!("invalid config file {path:?}"))
    }

    pub fn from_json(cfg_json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(cfg_json)?)
    }

    pub fn validate(&self, issues: &mut Issues) {
        issues.check(
            !self.jobs.is_empty(),
            "jobs",
            "config must contain at least one job",
        );
        if let Some(l) = self.location.as_ref() {
            issues.check(
                (-90.0..=90.0).contains(&l.latitude),
                "location",
                "Latitude is out of range",
            );
            issues.check(
                (-180.0..=180.0).contains(&l.longitude),
                "location",
                "Longitude is out of range",
            );
        }

        let mut seen = HashSet::new();
        for (i, job) in self.jobs.iter().enumerate() {
            issues.check(
                !job.name.is_empty(),
                format!("jobs[{i}].name"),
                "must not be empty",
            );
            let is_unique = seen.insert(job.name.as_str());
            issues.check(
                is_unique,
                format!("jobs[{i}].name"),
                "job names must be unique",
            );
            issues.check(
                job.retention_days > 0,
                format!("jobs[{i}].retention_days"),
                "must be greater than 0",
            );
            issues.check(
                job.sample_rate_hz > 0,
                format!("jobs[{i}].sample_rate_hz"),
                "must be greater than 0",
            );
            issues.check(
                job.max_runs.is_none_or(|m| m > 0),
                format!("jobs[{i}].max_runs"),
                "must be greater than 0 omitted from config to run indefinitely",
            );

            if let GainSetting::Db(db) = job.gain {
                issues.check(
                    db.is_finite(),
                    format!("jobs[{i}].gain"),
                    "must be a finite number",
                )
            }

            match &job.schedule {
                Schedule::Daily {
                    duration_minutes, ..
                } => {
                    issues.check(
                        *duration_minutes > 0,
                        format!("jobs[{i}].schedule.duration_minutes"),
                        "must be greater than 0",
                    );
                    issues.check(
                        *duration_minutes < 1440,
                        format!("jobs[{i}].schedule.duration_minutes"),
                        "must be less than 1440",
                    );
                }
                Schedule::Every {
                    interval_minutes,
                    duration_minutes,
                } => {
                    issues.check(
                        *interval_minutes > 0,
                        format!("jobs[{i}].schedule.interval_minutes"),
                        "must be greater than 0",
                    );
                    issues.check(
                        *duration_minutes > 0,
                        format!("jobs[{i}].schedule.duration_minutes"),
                        "must be greater than 0",
                    );
                    issues.check(
                        *duration_minutes < *interval_minutes,
                        format!("jobs[{i}].schedule.duration_minutes"),
                        "must be less than 'interval_minutes'",
                    );
                }
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Job {
    pub name: String,
    pub frequency_hz: u64,
    pub schedule: Schedule,
    pub demod_type: DemodType,
    #[serde(default = "default_retention")]
    pub retention_days: u32,
    #[serde(default)]
    pub gain: GainSetting,
    pub sample_rate_hz: u32,
    pub max_runs: Option<u32>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum DemodType {
    Nfm,
    Wfm,
    Am,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawGain {
    Keyword(String),
    Db(f32),
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(try_from = "RawGain")]
pub enum GainSetting {
    #[default]
    Auto,
    Db(f32),
}

impl TryFrom<RawGain> for GainSetting {
    type Error = anyhow::Error;
    fn try_from(raw: RawGain) -> Result<Self, Self::Error> {
        match raw {
            RawGain::Keyword(s) if s.eq_ignore_ascii_case("auto") => Ok(GainSetting::Auto),
            RawGain::Keyword(s) => anyhow::bail!("invalid gain {s:?}: expected \"auto\" or dB"),
            RawGain::Db(db) => Ok(GainSetting::Db(db)),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
#[serde(deny_unknown_fields)]
pub enum Schedule {
    Daily {
        start: StartTime,
        duration_minutes: u32,
    },
    Every {
        interval_minutes: u32,
        duration_minutes: u32,
    },
}
impl Schedule {
    pub fn duration_minutes(&self) -> u32 {
        match self {
            Self::Daily {
                duration_minutes, ..
            } => *duration_minutes,
            Self::Every {
                duration_minutes, ..
            } => *duration_minutes,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct StartTime {
    pub hour: u8,
    pub minute: u8,
}

impl TryFrom<String> for StartTime {
    type Error = anyhow::Error;
    fn try_from(start_str: String) -> Result<Self, Self::Error> {
        let (hour, min) = start_str
            .split_once(':')
            .context("expected HH:MM, missing ':'")?;
        let hour_int = hour.parse().with_context(|| {
            format!("invalid start time {start_str:?}: could not parse hour {hour:?}")
        })?;
        let min_int = min.parse().with_context(|| {
            format!("invalid start time {start_str:?}: could not parse minute {min:?}")
        })?;
        ensure!(
            hour_int < 24,
            "invalid start time {start_str:?}: hour must be less than 24"
        );
        ensure!(
            min_int < 60,
            "invalid start time {start_str:?}: minute must be less than 60"
        );
        Ok(StartTime {
            minute: min_int,
            hour: hour_int,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_success() {
        let config_str = r#"{
        "output_dir": "./recordings",
        "jobs": [
            {
            "name": "noaa_wx",
            "frequency_hz": 162550000,
            "sample_rate_hz": 2400000,
            "schedule": { "type": "daily", "start": "06:00", "duration_minutes": 60 },
            "demod_type": "nfm",
            "retention_days": 30
            }
        ]
        }
        "#;
        let config = Config::from_json(config_str).unwrap();
        assert_eq!(config.jobs[0].frequency_hz, 162550000);
        assert_eq!(config.jobs[0].retention_days, 30);
    }

    #[test]
    fn test_config_defaults() {
        let config_str = r#"{
        "jobs": [
            {
            "name": "noaa_wx",
            "frequency_hz": 162550000,
            "sample_rate_hz": 2400000,
            "schedule": { "type": "daily", "start": "06:00", "duration_minutes": 60 },
            "demod_type": "nfm"
            }
        ]
        }
        "#;
        let config = Config::from_json(config_str).unwrap();
        assert_eq!(config.jobs[0].retention_days, 30);
        assert_eq!(config.output_dir, Path::new("./recordings"));
        assert_eq!(config.jobs[0].gain, GainSetting::Auto);
    }

    #[test]
    fn test_error_on_unknown_fields() {
        let config_str = r#"{
        "jobs": [
            {
            "name": "noaa_wx",
            "frequency_typo": 162550000,
            "sample_rate_hz": 2400000,
            "schedule": { "type": "daily", "start": "06:00", "duration_minutes": 60 },
            "demod_type": "nfm"
            }
        ]
        }
        "#;
        assert!(Config::from_json(config_str).is_err());
    }

    #[test]
    fn test_invalid_start() {
        let config_str = r#"{
        "output_dir": "./recordings",
        "jobs": [
            {
            "name": "noaa_wx",
            "frequency_hz": 162550000,
            "sample_rate_hz": 2400000,
            "schedule": { "type": "daily", "start": "26:00", "duration_minutes": 60 },
            "demod_type": "nfm",
            "retention_days": 30
            }
        ]
        }
        "#;
        assert!(Config::from_json(config_str).is_err());
    }

    #[test]
    fn test_duplicate_job_names() {
        let config_str = r#"{
    "jobs": [
        { "name": "noaa_wx", "frequency_hz": 162550000, "sample_rate_hz": 2400000,
          "schedule": { "type": "daily", "start": "06:00", "duration_minutes": 60 },
          "demod_type": "nfm" },
        { "name": "noaa_wx", "frequency_hz": 162400000, "sample_rate_hz": 2400000,
          "schedule": { "type": "daily", "start": "07:00", "duration_minutes": 60 },
          "demod_type": "nfm" }
    ]
    }"#;
        let config = Config::from_json(config_str).unwrap();
        let mut issues = Issues::default();
        config.validate(&mut issues);
        let err = issues.into_result().unwrap_err();
        assert!(err.to_string().contains("unique"));
    }

    #[test]
    fn test_zero_retention() {
        let config_str = r#"{
        "jobs": [
            {
            "name": "noaa_wx",
            "frequency_hz": 162550000,
            "sample_rate_hz": 2400000,
            "schedule": { "type": "daily", "start": "06:00", "duration_minutes": 60 },
            "demod_type": "nfm",
            "retention_days": 0
            }
        ]
        }
        "#;
        let config = Config::from_json(config_str).unwrap();
        let mut issues = Issues::default();
        config.validate(&mut issues);
        let err = issues.into_result().unwrap_err();
        assert!(err.to_string().contains("retention_days"));
    }

    #[test]
    fn test_zero_daily_duration() {
        let config_str = r#"{
        "jobs": [
            {
            "name": "noaa_wx",
            "frequency_hz": 162550000,
            "sample_rate_hz": 2400000,
            "schedule": { "type": "daily", "start": "06:00", "duration_minutes": 0 },
            "demod_type": "nfm"
            }
        ]
        }
        "#;
        let config = Config::from_json(config_str).unwrap();
        let mut issues = Issues::default();
        config.validate(&mut issues);
        let err = issues.into_result().unwrap_err();
        assert!(err.to_string().contains("greater than 0"));
    }

    #[test]
    fn test_daily_duration_full_day() {
        let config_str = r#"{
        "jobs": [
            {
            "name": "noaa_wx",
            "frequency_hz": 162550000,
            "sample_rate_hz": 2400000,
            "schedule": { "type": "daily", "start": "06:00", "duration_minutes": 1440 },
            "demod_type": "nfm"
            }
        ]
        }
        "#;
        let config = Config::from_json(config_str).unwrap();
        let mut issues = Issues::default();
        config.validate(&mut issues);
        let err = issues.into_result().unwrap_err();
        assert!(err.to_string().contains("less than 1440"));
    }

    #[test]
    fn test_zero_interval() {
        let config_str = r#"{
        "jobs": [
            {
            "name": "airband",
            "frequency_hz": 121500000,
            "sample_rate_hz": 2400000,
            "schedule": { "type": "every", "interval_minutes": 0, "duration_minutes": 30 },
            "demod_type": "am"
            }
        ]
        }
        "#;
        let config = Config::from_json(config_str).unwrap();
        let mut issues = Issues::default();
        config.validate(&mut issues);
        let err = issues.into_result().unwrap_err();
        assert!(err.to_string().contains("interval_minutes"));
    }

    #[test]
    fn test_zero_interval_duration() {
        let config_str = r#"{
        "jobs": [
            {
            "name": "airband",
            "frequency_hz": 121500000,
            "sample_rate_hz": 2400000,
            "schedule": { "type": "every", "interval_minutes": 240, "duration_minutes": 0 },
            "demod_type": "am"
            }
        ]
        }
        "#;
        let config = Config::from_json(config_str).unwrap();
        let mut issues = Issues::default();
        config.validate(&mut issues);
        let err = issues.into_result().unwrap_err();
        assert!(err.to_string().contains("greater than 0"));
    }

    #[test]
    fn test_duration_not_less_than_interval() {
        let config_str = r#"{
        "jobs": [
            {
            "name": "airband",
            "frequency_hz": 121500000,
            "sample_rate_hz": 2400000,
            "schedule": { "type": "every", "interval_minutes": 60, "duration_minutes": 60 },
            "demod_type": "am"
            }
        ]
        }
        "#;
        let config = Config::from_json(config_str).unwrap();
        let mut issues = Issues::default();
        config.validate(&mut issues);
        let err = issues.into_result().unwrap_err();
        assert!(err.to_string().contains("less than 'interval_minutes'"));
    }

    #[test]
    fn test_valid_start_times() {
        assert_eq!(
            StartTime::try_from("06:00".to_string()).unwrap(),
            StartTime { hour: 6, minute: 0 }
        );
        assert_eq!(
            StartTime::try_from("00:00".to_string()).unwrap(),
            StartTime { hour: 0, minute: 0 }
        );
        assert_eq!(
            StartTime::try_from("23:59".to_string()).unwrap(),
            StartTime {
                hour: 23,
                minute: 59
            }
        );
        assert_eq!(
            StartTime::try_from("6:5".to_string()).unwrap(),
            StartTime { hour: 6, minute: 5 }
        );
    }

    #[test]
    fn test_malformed_start_times() {
        for bad in [
            "", "0600", "06", "06:", ":00", "ab:cd", "06:00:00", "-1:00", "06:00 ",
        ] {
            assert!(
                StartTime::try_from(bad.to_string()).is_err(),
                "expected {bad:?} to be rejected"
            );
        }
    }

    #[test]
    fn test_out_of_range_start_times() {
        assert!(StartTime::try_from("24:00".to_string()).is_err());
        assert!(StartTime::try_from("26:00".to_string()).is_err());
        assert!(StartTime::try_from("12:60".to_string()).is_err());
        assert!(StartTime::try_from("300:00".to_string()).is_err());
    }

    #[test]
    fn test_unknown_field_inside_schedule() {
        let config_str = r#"{
    "jobs": [
        { "name": "noaa_wx", "frequency_hz": 162550000, "sample_rate_hz": 2400000,
          "schedule": { "type": "daily", "start": "06:00", "duration_minutes": 60, "bruh": 3 },
          "demod_type": "nfm" }
    ]
    }"#;
        assert!(Config::from_json(config_str).is_err());
    }
}
