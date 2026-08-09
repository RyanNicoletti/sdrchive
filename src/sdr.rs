use std::ops::RangeInclusive;

use num_complex::Complex;
use rtl_sdr_rs::{RtlSdr, TunerGain};

const RAW_BYTES: usize = 16 * 16384;

pub struct Capabilities {
    pub frequency_range_hz: RangeInclusive<u64>,
    pub sample_rates_hz: Vec<RangeInclusive<u32>>,
    pub gain_steps_db: Vec<f32>,
}

impl Capabilities {
    pub fn supports_fs(&self, rate: u32) -> bool {
        self.sample_rates_hz.iter().any(|r| r.contains(&rate))
    }
    pub fn supports_freq(&self, freq: u64) -> bool {
        self.frequency_range_hz.contains(&freq)
    }
    pub fn supports_gain(&self, gain: f32) -> bool {
        self.gain_steps_db.iter().any(|&g| g == gain)
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

#[derive(Debug)]

pub struct HardwareConfig {
    pub center_freq_hz: u64,
    pub sample_rate_hz: u32,
    pub gain: Gain,
}

pub trait SdrDevice {
    fn capabilities(&self) -> &Capabilities;
    fn read_iq(&mut self) -> anyhow::Result<&[Complex<f32>]>;
    fn configure(&mut self, cfg: &HardwareConfig) -> anyhow::Result<()>;
    fn name(&self) -> &'static str;
}

pub struct RtlSdrDev {
    device: RtlSdr,
    capabilities: Capabilities,
    raw: Vec<u8>,
    iq: Vec<Complex<f32>>,
}

impl RtlSdrDev {
    pub fn new() -> anyhow::Result<Self> {
        let device = RtlSdr::open_first_available()?;
        let capabilities = Capabilities {
            frequency_range_hz: 24_000_000..=1_766_000_000,
            sample_rates_hz: vec![225_001..=300_000, 900_001..=3_200_000],
            // source: https://github.com/osmocom/rtl-sdr/blob/master/src/tuner_r82xx.c
            gain_steps_db: vec![
                -1.0, 1.5, 4.0, 6.5, 9.0, 11.5, 14.0, 16.5, 19.0, 21.5, 24.0, 29.0, 34.0, 42.0,
                43.0, 45.0, 47.0, 49.0,
            ],
        };
        let raw = vec![0; RAW_BYTES];
        let iq = vec![Complex::new(0.0, 0.0); RAW_BYTES / 2];
        Ok(Self {
            device,
            capabilities,
            raw,
            iq,
        })
    }
}

impl SdrDevice for RtlSdrDev {
    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
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

    fn configure(&mut self, cfg: &HardwareConfig) -> anyhow::Result<()> {
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

    fn name(&self) -> &'static str {
        "RTL-SDR"
    }
}

pub fn detect() -> anyhow::Result<Box<dyn SdrDevice>> {
    let mut tried: Vec<String> = Vec::new();
    match RtlSdrDev::new() {
        Ok(rtl) => {
            println!("Using device: {}", rtl.name());
            return Ok(Box::new(rtl));
        }
        Err(e) => tried.push(format!("RTLSDR: ({e})")),
    };
    anyhow::bail!("No SDR device found, tried: {}", tried.join(", "))
}
