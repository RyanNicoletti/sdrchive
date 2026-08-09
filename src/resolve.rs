use std::path::PathBuf;

use crate::{
    config::{Config, DemodType, GainSetting, Issues, Location, Schedule},
    sdr::{
        Capabilities,
        Gain::{self, Auto},
        HardwareConfig,
    },
};

struct ResolvedConfig {
    pub output_dir: PathBuf,
    pub location: Option<Location>,
    pub resolved_jobs: Vec<ResolvedJob>,
}

struct ResolvedJob {
    pub name: String,
    pub schedule: Schedule,
    pub demod_type: DemodType,
    pub retention_days: u32,
    pub hw_config: HardwareConfig,
}

fn resolve(cfg: &Config, caps: &Capabilities) -> Result<ResolvedConfig, Issues> {
    let mut resolved_jobs: Vec<ResolvedJob> = Vec::new();
    let mut issues: Issues = Issues::default();
    for j in &cfg.jobs {
        let beforelen = issues.items.len();
        issues.check(
            caps.supports_freq(j.frequency_hz),
            format!("{}", j.frequency_hz),
            "frequency is not supported",
        );
        issues.check(
            caps.supports_fs(j.sample_rate_hz),
            format!("{}", j.sample_rate_hz),
            "sample rate is not supported",
        );
        let gain = match j.gain {
            GainSetting::Auto => Auto,
            GainSetting::Db(db) => Gain::Db(get_nearest_gain(&db, &caps.gain_steps_db)),
        };
        let afterlen = issues.items.len();
        if afterlen == beforelen {
            resolved_jobs.push(ResolvedJob {
                name: j.name.clone(),
                schedule: j.schedule.clone(),
                demod_type: j.demod_type.clone(),
                retention_days: j.retention_days,
                hw_config: HardwareConfig {
                    center_freq_hz: j.frequency_hz,
                    sample_rate_hz: j.sample_rate_hz,
                    gain,
                },
            })
        }
    }
    if issues.items.is_empty() {
        Ok(ResolvedConfig {
            output_dir: cfg.output_dir.clone(),
            location: cfg.location,
            resolved_jobs,
        })
    } else {
        Err(issues)
    }
}

fn get_nearest_gain(db: &f32, gains: &Vec<f32>) -> f32 {
    let mut closest = gains[0];
    for gain in &gains[1..] {
        if (db - *gain).abs() < (db - closest).abs() {
            closest = (db - *gain).abs();
        }
    }
    closest
}
