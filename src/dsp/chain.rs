use futuredsp::{firdes, windows};
use num_complex::Complex;

use crate::{
    config::DemodType,
    dsp::{demod_params::params_for, discriminator::FmDiscriminator, fir::StreamingFir},
};

pub enum DemodChainType {
    Fm(FmDemodChain),
    // Am(AmDemodChain), TODO: set up AmDemodChain, one variant enum for now
}

impl DemodChainType {
    pub fn new(demod_type: DemodType, sdr_fs: u32) -> Self {
        match demod_type {
            DemodType::Nfm | DemodType::Wfm => {
                DemodChainType::Fm(FmDemodChain::new(demod_type, sdr_fs))
            }
            DemodType::Am => todo!(),
        }
    }
    pub fn process(&mut self, iq: &[Complex<f32>]) -> &[f32] {
        match self {
            DemodChainType::Fm(fm) => fm.process(iq),
        }
    }
    pub fn get_audio_fs(&mut self) -> u32 {
        match self {
            DemodChainType::Fm(fm) => fm.audio_sample_rate_hz(),
        }
    }
}

pub struct FmDemodChain {
    channel_fir: StreamingFir<Complex<f32>, Complex<f32>, Vec<f32>>,
    disc: FmDiscriminator,
    audio_fir: StreamingFir<f32, f32, Vec<f32>>,
    pub audio_fs: u32,
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
    pub fn process(&mut self, iq: &[Complex<f32>]) -> &[f32] {
        let filtered = self.channel_fir.run_filter(iq);
        let audio = self.disc.run_discriminator(filtered);
        let filt_audio = self.audio_fir.run_filter(audio);
        filt_audio
    }
    pub fn audio_sample_rate_hz(&mut self) -> u32 {
        self.audio_fs
    }
}
