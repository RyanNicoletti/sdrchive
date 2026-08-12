use crate::{resolve::ResolvedJob, sdr::SdrDevice};
use std::path::Path;

pub fn run_job(
    job: &ResolvedJob,
    duration: u32,
    sdr: &mut dyn SdrDevice,
    outpath: &Path,
) -> anyhow::Result<()> {
    sdr.configure(&job.hw_config)?;
    let samples_needed: u64 = duration as u64 * job.hw_config.sample_rate_hz as u64;
    let mut current_total: u64 = 0;
    while current_total < samples_needed {
        let chunk = sdr.read_iq()?;
        current_total += chunk.len() as u64;
    }
    println!("Collected {} samples", current_total);
    Ok(())
}
