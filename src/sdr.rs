use num_complex::Complex;
use rtl_sdr_rs::RtlSdr;

fn detect_source() -> Result<Box<dyn IqSource>, anyhow::Error> {
    let device = RtlSdr::open_first_available()?;
    Box::new(RtlSdrSource { device })?
}

trait IqSource {
    fn read_iq(&mut self, iq: &mut [Complex<f32>]) -> anyhow::Result<usize>;
    fn configure(&mut self, frequency: u64, sample_rate: u64) -> anyhow::Result<()>;
}

struct RtlSdrSource {
    device: RtlSdr,
}

impl RtlSdrSource {
    fn new(dev: RtlSdr) -> Self {
        todo!()
    }
}

impl IqSource for RtlSdrSource {
    fn read_iq(&mut self, iq: &mut [Complex<f32>]) -> anyhow::Result<usize> {
        todo!()
    }
    fn configure(&mut self, frequency: u64, sample_rate: u64) -> anyhow::Result<()> {
        todo!()
    }
}
