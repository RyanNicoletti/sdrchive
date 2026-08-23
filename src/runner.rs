use std::process::exit;

use crate::{dsp::chain::DemodChainType, resolve::ResolvedJob, sdr::SdrDevice};

pub fn run_job(
    job: &ResolvedJob,
    job_duration: u32,
    sdr: &mut dyn SdrDevice,
) -> anyhow::Result<()> {
    sdr.configure(&job.hw_config)?;
    let mut filter_chain = DemodChainType::new(job.demod_type, job.hw_config.sample_rate_hz);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: filter_chain.get_audio_fs(),
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut wav_writer = hound::WavWriter::create("test.wav", spec)?;
    let samples_needed: u64 = job_duration as u64 * job.hw_config.sample_rate_hz as u64;
    let mut count: u64 = 0;
    while count < samples_needed {
        let samples = sdr.read_iq()?;
        let demoded_samples = filter_chain.process(samples);
        for &s in demoded_samples {
            let audio_sample = ((s / std::f32::consts::PI) * i16::MAX as f32) as i16;
            wav_writer.write_sample(audio_sample)?;
        }
        count += samples.len() as u64;
    }
    wav_writer.finalize()?;
    println!("DONT WRITING FILE");
    exit(0);
    Ok(())
}
