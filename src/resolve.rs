use std::path::PathBuf;

use crate::{
    config::{Config, DemodType, GainSetting, Issues, Location, Schedule},
    sdr::{
        Capabilities,
        Gain::{self, Auto},
        HardwareConfig,
    },
};

#[derive(Debug)]

pub struct ResolvedConfig {
    pub output_dir: PathBuf,
    pub location: Option<Location>,
    pub resolved_jobs: Vec<ResolvedJob>,
}

#[derive(Debug)]

pub struct ResolvedJob {
    pub name: String,
    pub schedule: Schedule,
    pub demod_type: DemodType,
    pub retention_days: u32,
    pub hw_config: HardwareConfig,
}

pub fn resolve(cfg: &Config, caps: &Capabilities, issues: &mut Issues) -> ResolvedConfig {
    let mut resolved_jobs: Vec<ResolvedJob> = Vec::new();
    let supported_rates = caps
        .sample_rates_hz
        .iter()
        .map(|r| format!("{}-{}", r.start(), r.end()))
        .collect::<Vec<_>>()
        .join(", ");
    for (i, j) in cfg.jobs.iter().enumerate() {
        issues.check(
            caps.supports_freq(j.frequency_hz),
            format!("jobs[{i}].frequency_hz"),
            format!(
                "{} is outside the device range {}-{}",
                j.frequency_hz,
                caps.frequency_range_hz.start(),
                caps.frequency_range_hz.end()
            ),
        );

        issues.check(
            caps.supports_fs(j.sample_rate_hz),
            format!("jobs[{i}].sample_rate_hz"),
            format!(
                "{} is not supported; device supports {supported_rates}",
                j.sample_rate_hz
            ),
        );
        let gain = match j.gain {
            GainSetting::Auto => Auto,
            GainSetting::Db(db) => {
                let nearest = get_nearest_gain(&db, &caps.gain_steps_db);
                if nearest != db {
                    eprintln!(
                        "jobs[{i}].gain converted to nearest gain compatible with device: {nearest} dB"
                    );
                }
                Gain::Db(nearest)
            }
        };

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
    ResolvedConfig {
        output_dir: cfg.output_dir.clone(),
        location: cfg.location,
        resolved_jobs,
    }
}

fn get_nearest_gain(db: &f32, gains: &[f32]) -> f32 {
    let mut closest = gains[0];
    for gain in &gains[1..] {
        if (db - *gain).abs() < (db - closest).abs() {
            closest = *gain;
        }
    }
    closest
}
