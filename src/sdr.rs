use std::ops::RangeInclusive;

use num_complex::Complex;
use rtl_sdr_rs::{RtlSdr, TunerGain};

const RAW_BYTES: usize = 16 * 16384;

pub struct Specifications {
    pub frequency_range_hz: RangeInclusive<u64>,
    pub sample_rates_hz: Vec<RangeInclusive<u32>>,
}

impl Specifications {
    pub fn supports_fs(&self, rate: u32) -> bool {
        self.sample_rates_hz.iter().any(|r| r.contains(&rate))
    }
    pub fn supports_freq(&self, freq: u64) -> bool {
        self.frequency_range_hz.contains(&freq)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gain {
    Auto,
    Db(f32),
}

impl Default for Gain {
    fn default() -> Self {
        Gain::Auto
    }
}

pub struct SourceConfig {
    pub center_freq_hz: u64,
    pub sample_rate_hz: u32,
    pub gain: Gain,
}

pub trait IqSource {
    fn specifications(&self) -> Specifications;
    fn read_iq(&mut self) -> anyhow::Result<&[Complex<f32>]>;
    fn configure(&mut self, cfg: &SourceConfig) -> anyhow::Result<()>;
}

pub struct RtlSdrSource {
    device: RtlSdr,
    raw: Vec<u8>,
    iq: Vec<Complex<f32>>,
}

impl RtlSdrSource {
    pub fn new() -> anyhow::Result<Self> {
        let device = RtlSdr::open_first_available()?;
        let raw = vec![0; RAW_BYTES];
        let iq = vec![Complex::new(0.0, 0.0); RAW_BYTES / 2];
        Ok(Self { device, raw, iq })
    }
}

impl IqSource for RtlSdrSource {
    fn specifications(&self) -> Specifications {
        Specifications {
            frequency_range_hz: 24000000..=1766000000,
            sample_rates_hz: vec![225001..=300000, 900001..=3200000],
        }
    }

    fn read_iq(&mut self) -> anyhow::Result<&[Complex<f32>]> {
        let bytes_read = self.device.read_sync(&mut self.raw)?;
        anyhow::ensure!(
            bytes_read % 2 == 0,
            "odd byte count from device: {bytes_read}"
        );
        let iq_pairs = bytes_read / 2;
        for (sample, bytes) in self.iq[..iq_pairs]
            .iter_mut()
            .zip(self.raw[..iq_pairs * 2].chunks_exact(2))
        {
            let i = (bytes[0] as f32 - 127.5) / 127.5;
            let q = (bytes[1] as f32 - 127.5) / 127.5;
            *sample = Complex::new(i, q);
        }
        Ok(&self.iq[..iq_pairs])
    }

    fn configure(&mut self, cfg: &SourceConfig) -> anyhow::Result<()> {
        self.device
            .set_center_freq(u32::try_from(cfg.center_freq_hz)?)?;
        self.device.set_sample_rate(cfg.sample_rate_hz)?;
        match cfg.gain {
            Gain::Auto => self.device.set_tuner_gain(TunerGain::Auto)?,
            Gain::Db(db) => self
                .device
                .set_tuner_gain(TunerGain::Manual((db * 10.0).round() as i32))?,
        }
        self.device.reset_buffer()?;
        Ok(())
    }
}
