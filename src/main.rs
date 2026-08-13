mod config;
mod resolve;
mod runner;
mod scheduler;
mod sdr;
use crate::resolve::resolve;
use crate::{config::Config, resolve::ResolvedConfig, scheduler::Scheduler, sdr::detect};
use anyhow::Result;
use std::fs::create_dir_all;
use std::path::Path;

fn main() -> Result<()> {
    let config_str = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sdrchive_config.json".to_string());
    let config_path: &Path = Path::new(&config_str);
    let cfg: Config = config::Config::load(config_path)?;
    create_dir_all(&cfg.output_dir)?;
    let mut sdr = detect()?;
    let caps = sdr.capabilities();
    let resolved_cfg: ResolvedConfig = resolve(&cfg, caps)?;
    let mut scheduler = Scheduler::new(resolved_cfg)?;
    // scheduler.run(&sdr)?;
    Ok(())
}
