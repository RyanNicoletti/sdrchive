mod iq;

pub use iq::IqSink;
use num_complex::Complex;

pub trait Sink {
    fn write(&mut self, samples: &[Complex<f32>]) -> anyhow::Result<()>;
    fn finish(self: Box<Self>) -> anyhow::Result<()>;
}
