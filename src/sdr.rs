use num_complex::Complex;
use rtl_sdr_rs::RtlSdr;

trait IqSource {
    fn read_iq(&mut self, iq: &mut [Complex<f32>]) -> anyhow::Result<usize>;
    fn configure(&mut self, frequency: u32, sample_rate: u32) -> anyhow::Result<()>;
}

struct RtlSdrSource {
    device: RtlSdr,
}

impl RtlSdrSource {
    fn new() -> anyhow::Result<Self> {
        let device = RtlSdr::open_first_available()?;
        Ok(Self { device })
    }
}

impl IqSource for RtlSdrSource {
    fn read_iq(&mut self, iq_buff: &mut [Complex<f32>]) -> anyhow::Result<usize> {
        let mut raw = vec![0_u8; iq_buff.len() * 2];
        let bytes_read = self.device.read_sync(&mut raw)?;
        let samples_read = bytes_read / 2;
        for (sample, bytes) in iq_buff[..samples_read]
            .iter_mut()
            .zip(raw[..samples_read * 2].chunks_exact(2))
        {
            let i = (bytes[0] as f32 - 127.5) / 127.5;
            let q = (bytes[1] as f32 - 127.5) / 127.5;
            *sample = Complex::new(i, q);
        }
        Ok(samples_read)
    }

    fn configure(&mut self, frequency: u32, sample_rate: u32) -> anyhow::Result<()> {
        self.device.set_center_freq(frequency)?;
        self.device.set_sample_rate(sample_rate)?;
        self.device.reset_buffer()?;
        Ok(())
    }
}
