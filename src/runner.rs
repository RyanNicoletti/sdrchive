use std::path::PathBuf;

use jiff::Zoned;

use crate::{dsp::chain::DemodChainType, resolve::ResolvedJob, sdr::SdrDevice};

pub fn run_job(
    job: &ResolvedJob,
    job_duration: u32,
    sdr: &mut dyn SdrDevice,
    out_dir: &PathBuf,
) -> anyhow::Result<()> {
    sdr.configure(&job.hw_config)?;
    let mut filter_chain = DemodChainType::new(job.demod_type, job.hw_config.sample_rate_hz);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: filter_chain.get_audio_fs(),
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let out_path = make_out_path(out_dir, job.name.as_str())?;
    let mut wav_writer = hound::WavWriter::create(out_path, spec)?;
    let samples_needed: u64 = job_duration as u64 * job.hw_config.sample_rate_hz as u64;
    let mut count: u64 = 0;
    while count < samples_needed {
        let samples = sdr.read_iq()?;
        let demoded_samples = filter_chain.process(samples);
        for &s in demoded_samples {
            let audio_sample = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            wav_writer.write_sample(audio_sample)?;
        }
        count += samples.len() as u64;
    }
    wav_writer.finalize()?;
    Ok(())
}

fn make_out_path(out_dir: &PathBuf, file_name: &str) -> anyhow::Result<PathBuf> {
    let date = Zoned::now().strftime("%Y%m%dT%H%M%S").to_string();
    let subdir = out_dir.join(file_name);
    std::fs::create_dir_all(&subdir)?;
    Ok(subdir.join(format!("{date}.wav")))
}
