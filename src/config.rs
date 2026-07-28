use anyhow::{Context, ensure};
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
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
    pub jobs: Vec<Job>,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let file: File =
            File::open(path).with_context(|| format!("Could not open config file {path:?}"))?;
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)
            .with_context(|| format!("Unable to parse config file {path:?}"))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            !self.jobs.is_empty(),
            "Config must contain at least one job."
        );
        let mut seen = std::collections::HashSet::new();
        for (i, job) in self.jobs.iter().enumerate() {
            ensure!(
                !job.name.is_empty(),
                "Job #{i}: the field 'name' must not be empty."
            );
            ensure!(
                seen.insert(job.name.as_str()),
                "Job {:?}: job names must be unique.",
                job.name
            );
            ensure!(
                job.retention_days > 0,
                "Job {:?}: the field 'retention_days' must be greater than 0.",
                job.name
            );
            match &job.schedule {
                Schedule::Daily {
                    duration_minutes, ..
                } => {
                    ensure!(
                        *duration_minutes > 0,
                        "Job {:?}: the field 'duration_minutes' must be greater than 0 in job schedule",
                        job.name
                    );
                }
                Schedule::Every {
                    interval_minutes,
                    duration_minutes,
                } => {
                    ensure!(
                        *interval_minutes > 0,
                        "Job {:?}: the field 'interval_minutes' must be greater than 0 in job schedule",
                        job.name
                    );
                    ensure!(
                        *duration_minutes > 0,
                        "Job {:?}: the field 'duration_minutes' must be greater than 0 in job schedule",
                        job.name
                    );
                }
            }
        }
        Ok(())
    }
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
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DemodType {
    Nfm,
    Wfm,
    Am,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Schedule {
    Daily {
        start: String,
        duration_minutes: u32,
    },
    Every {
        interval_minutes: u32,
        duration_minutes: u32,
    },
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
            "schedule": { "type": "daily", "start": "06:00", "duration_minutes": 60 },
            "demod_type": "nfm",
            "retention_days": 30
            }
        ]
        }
        "#;
        let config: Config = serde_json::from_str(config_str).unwrap();
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
            "schedule": { "type": "daily", "start": "06:00", "duration_minutes": 60 },
            "demod_type": "nfm"
            }
        ]
        }
        "#;
        let config: Config = serde_json::from_str(config_str).unwrap();
        assert_eq!(config.jobs[0].retention_days, 30);
        assert_eq!(config.output_dir, Path::new("./recordings"));
    }

    #[test]
    fn test_error_on_unknown_fields() {
        let config_str = r#"{
        "jobs": [
            {
            "name": "noaa_wx",
            "frequency_typo": 162550000,
            "schedule": { "type": "daily", "start": "06:00", "duration_minutes": 60 },
            "demod_type": "nfm"
            }
        ]
        }
        "#;
        let config: Result<Config, _> = serde_json::from_str(config_str);
        assert!(config.is_err());
    }
}
