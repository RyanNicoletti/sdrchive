use crate::config::DemodType;

pub struct DemodParams {
    pub channel_fs_hz: u32,
    pub channel_cutoff_hz: f32,
    pub channel_taps: usize,
    pub audio_fs_hz: u32,
    pub audio_cutoff_hz: f32,
    pub audio_taps: usize,
    pub dc_cutoff_hz: Option<f32>,
}

pub fn params_for(demod: DemodType) -> DemodParams {
    match demod {
        DemodType::Nfm => DemodParams {
            channel_fs_hz: 48000,
            channel_cutoff_hz: 8000.0,
            channel_taps: 101,
            audio_fs_hz: 24000,
            audio_cutoff_hz: 5000.0,
            audio_taps: 41,
            dc_cutoff_hz: None,
        },
        DemodType::Wfm => DemodParams {
            channel_fs_hz: 240000,
            channel_cutoff_hz: 100000.0,
            channel_taps: 81,
            audio_fs_hz: 48000,
            audio_cutoff_hz: 15000.0,
            audio_taps: 81,
            dc_cutoff_hz: None,
        },
        DemodType::Am => DemodParams {
            channel_fs_hz: 48000,
            channel_cutoff_hz: 5000.0,
            channel_taps: 101,
            audio_fs_hz: 24000,
            audio_cutoff_hz: 4000.0,
            audio_taps: 41,
            dc_cutoff_hz: Some(20.0),
        },
    }
}
