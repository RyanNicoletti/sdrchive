use crate::{resolve::ResolvedJob, sdr::SdrDevice};
use jiff::Zoned;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn run_job(
    job: &ResolvedJob,
    duration: u32,
    sdr: &mut dyn SdrDevice,
    outpath: &Path,
) -> anyhow::Result<()> {
    sdr.configure(&job.hw_config)?;
    let timestamp = Zoned::now().strftime("%Y%m%dT%H%M%S").to_string();
    let path = outpath.join(format!("{}_{}.iq", &job.name, timestamp));
    let file = File::create(&path)?;
    let mut file_writer = BufWriter::new(file);
    let samples_needed: u64 = duration as u64 * job.hw_config.sample_rate_hz as u64;
    let mut current_total: u64 = 0;
    while current_total < samples_needed {
        let samples = sdr.read_iq()?;
        for s in samples {
            file_writer.write_all(&s.re.to_le_bytes())?;
            file_writer.write_all(&s.im.to_le_bytes())?;
        }
        current_total += samples.len() as u64;
    }
    file_writer.flush()?;
    println!("Collected {} samples", current_total);
    Ok(())
}
