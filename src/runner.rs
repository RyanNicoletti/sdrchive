use crate::sink::Sink;
use crate::{resolve::ResolvedJob, sdr::SdrDevice};

pub fn run_job(
    job: &ResolvedJob,
    job_duration: u32,
    sdr: &mut dyn SdrDevice,
    mut sink: Box<dyn Sink>,
) -> anyhow::Result<()> {
    sdr.configure(&job.hw_config)?;
    let samples_needed: u64 = job_duration as u64 * job.hw_config.sample_rate_hz as u64;
    let mut count: u64 = 0;
    while count < samples_needed {
        let samples = sdr.read_iq()?;
        sink.write(samples)?;
        count += samples.len() as u64;
    }
    sink.finish()?;
    println!("Collected {} samples", count);
    Ok(())
}
