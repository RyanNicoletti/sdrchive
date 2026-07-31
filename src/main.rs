mod config;
mod scheduler;
use anyhow::Result;
use std::path::Path;

use crate::scheduler::Scheduler;

fn main() -> Result<()> {
    let config_str = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sdrchive_config.json".to_string());
    let config_path: &Path = Path::new(&config_str);
    let cfg = config::Config::load(config_path)?;
    let mut scheduler = Scheduler::new(cfg)?;
    scheduler.run();
    Ok(())
}
