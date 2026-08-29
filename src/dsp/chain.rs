use futuredsp::{firdes, windows};
use num_complex::Complex;

use crate::{
    config::DemodType,
    dsp::{
        agc::Agc,
        dc_blocker::DcBlocker,
        demod_params::params_for,
        discriminator::FmDiscriminator,
        envelope::AmEnvelope,
        fir::StreamingFir,
        squelch::{self, Squelch},
    },
};

pub enum DemodChainType {
    Fm(FmDemodChain),
    Am(AmDemodChain),
}

impl DemodChainType {
    pub fn new(demod_type: DemodType, sdr_fs: u32, squelch_thresh_dbfs: Option<f32>) -> Self {
        match demod_type {
            DemodType::Nfm | DemodType::Wfm => {
                DemodChainType::Fm(FmDemodChain::new(demod_type, sdr_fs))
            }
            DemodType::Am => {
                DemodChainType::Am(AmDemodChain::new(demod_type, sdr_fs, squelch_thresh_dbfs))
            }
        }
    }
    pub fn process(&mut self, iq: &[Complex<f32>]) -> &[f32] {
        match self {
            DemodChainType::Fm(fm) => fm.process(iq),
            DemodChainType::Am(am) => am.process(iq),
        }
    }
    pub fn get_audio_fs(&self) -> u32 {
        match self {
            DemodChainType::Fm(fm) => fm.audio_sample_rate_hz(),
            DemodChainType::Am(am) => am.audio_sample_rate_hz(),
        }
    }
}

pub struct FmDemodChain {
    channel_fir: StreamingFir<Complex<f32>, Complex<f32>, Vec<f32>>,
    disc: FmDiscriminator,
    audio_fir: StreamingFir<f32, f32, Vec<f32>>,
    audio_fs: u32,
}

impl FmDemodChain {
    pub fn new(demod_type: DemodType, sdr_fs_hz: u32) -> Self {
        let demod_params = params_for(demod_type);
        let dec1 = sdr_fs_hz / demod_params.channel_fs_hz;
        let dec2 = demod_params.channel_fs_hz / demod_params.audio_fs_hz;
        let hamming_win1 = windows::hamming(demod_params.channel_taps, false);
        let hamming_win2 = windows::hamming(demod_params.audio_taps, false);
        let cutoff_normalized1 = demod_params.channel_cutoff_hz as f64 / sdr_fs_hz as f64;
        let cutoff_normalized2 =
            demod_params.audio_cutoff_hz as f64 / demod_params.channel_fs_hz as f64;
        let taps1 = firdes::lowpass::<f32>(cutoff_normalized1, hamming_win1.as_slice());
        let taps2 = firdes::lowpass::<f32>(cutoff_normalized2, hamming_win2.as_slice());
        FmDemodChain {
            channel_fir: StreamingFir::new(dec1 as usize, taps1),
            disc: FmDiscriminator::default(),
            audio_fir: StreamingFir::new(dec2 as usize, taps2),
            audio_fs: demod_params.audio_fs_hz,
        }
    }
    fn process(&mut self, iq: &[Complex<f32>]) -> &[f32] {
        let filtered = self.channel_fir.run_filter(iq);
        let audio = self.disc.run_discriminator(filtered);
        let filt_audio = self.audio_fir.run_filter(audio);
        filt_audio
    }
    fn audio_sample_rate_hz(&self) -> u32 {
        self.audio_fs
    }
}

pub struct AmDemodChain {
    channel_fir: StreamingFir<Complex<f32>, Complex<f32>, Vec<f32>>,
    envelope: AmEnvelope,
    audio_fir: StreamingFir<f32, f32, Vec<f32>>,
    dc_blocker: DcBlocker,
    audio_fs: u32,
    agc: Agc,
    squelch: Option<Squelch>,
    muted: Vec<f32>,
}

impl AmDemodChain {
    pub fn new(demod_type: DemodType, sdr_fs_hz: u32, squelch_threshold: Option<f32>) -> Self {
        let demod_params = params_for(demod_type);
        let dec1 = sdr_fs_hz / demod_params.channel_fs_hz;
        let dec2 = demod_params.channel_fs_hz / demod_params.audio_fs_hz;
        let hamming_win1 = windows::hamming(demod_params.channel_taps, false);
        let hamming_win2 = windows::hamming(demod_params.audio_taps, false);
        let cutoff_normalized1 = demod_params.channel_cutoff_hz as f64 / sdr_fs_hz as f64;
        let cutoff_normalized2 =
            demod_params.audio_cutoff_hz as f64 / demod_params.channel_fs_hz as f64;
        let taps1 = firdes::lowpass::<f32>(cutoff_normalized1, hamming_win1.as_slice());
        let taps2 = firdes::lowpass::<f32>(cutoff_normalized2, hamming_win2.as_slice());
        AmDemodChain {
            channel_fir: StreamingFir::new(dec1 as usize, taps1),
            envelope: AmEnvelope::default(),
            dc_blocker: DcBlocker::new(
                demod_params.audio_fs_hz,
                demod_params
                    .dc_cutoff_hz
                    .expect("AM params must have dc_cutoff_hz"),
            ),
            audio_fir: StreamingFir::new(dec2 as usize, taps2),
            audio_fs: demod_params.audio_fs_hz,
            agc: Agc::new(0.3, 1e-2, 1e-4, 100.0),
            squelch: squelch_threshold.map(Squelch::new),
            muted: Vec::new(),
        }
    }
    fn process(&mut self, iq: &[Complex<f32>]) -> &[f32] {
        let filtered = self.channel_fir.run_filter(iq);
        let mean_mag = filtered.iter().map(|s| s.norm()).sum::<f32>() / filtered.len() as f32;
        let is_squelch_open = if let Some(s) = &mut self.squelch {
            s.update(mean_mag)
        } else {
            true
        };
        let audio = self.envelope.run(filtered);
        let filt_audio = self.audio_fir.run_filter(audio);
        let cleaned = self.dc_blocker.run(filt_audio);
        let boosted = self.agc.run(cleaned);
        if is_squelch_open {
            boosted
        } else {
            self.muted.resize(boosted.len(), 0.0);
            self.muted.fill(0.0);
            &self.muted
        }
    }
    fn audio_sample_rate_hz(&self) -> u32 {
        self.audio_fs
    }
}
