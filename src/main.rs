mod dsp;
mod sdr;
use anyhow::Result;
use dsp::FmDemodulator;
use num_complex::Complex;
use sdr::{FileSdr, SdrSource};
use std::println;

const SIGMF_META: &str = "data/test_1khz_fm.sigmf-meta";

fn main() -> Result<()> {
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
        // println!("{} {} {}", audio_buf[1], audio_buf[121], audio_buf[241]);
    }
    Ok(())
}
