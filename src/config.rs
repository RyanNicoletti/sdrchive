use anyhow::Context;
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

pub fn load_config(path: &Path) -> anyhow::Result<Config> {
    let file: File =
        File::open(path).with_context(|| format!("Could not open config file {path:?}"))?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)
        .with_context(|| format!("Unable to parse config file {path:?}"))?;
    Ok(config)
}
