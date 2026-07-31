mod config;
mod dsp;
mod scheduler;
mod sdr;
use anyhow::Result;
use dsp::FmDemodulator;
use num_complex::Complex;
use sdr::{FileSdr, SdrSource};
use std::path::Path;

const SIGMF_META: &str = "data/test_1khz_fm.sigmf-meta";

fn main() -> Result<()> {
    let config_str = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sdrchive_config.json".to_string());
    let config_path: &Path = Path::new(&config_str);
    let config_parsed = config::Config::load(config_path)?;
    println!("{:?}", config_parsed);

    let mut buf = vec![Complex::new(0.0, 0.0); 512];
    let mut sdr = FileSdr::new(SIGMF_META, buf.len())?;
    let mut fm_demod = FmDemodulator::new();
    let mut audio_buf: Vec<f32> = Vec::new();
    loop {
        let n = sdr.read_samples(&mut buf)?;
        println!("{}", n);
        if n == 0 {
            println!("brodie");
            break;
        }
        fm_demod.demodulate(&buf[..n], &mut audio_buf);
    }
    Ok(())
}
