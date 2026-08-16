use jiff::Zoned;
use num_complex::Complex;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use super::Sink;

pub struct IqSink {
    writer: BufWriter<File>,
}

impl IqSink {
    pub fn new(job_name: &str, output_dir: &Path) -> anyhow::Result<Self> {
        let timestamp = Zoned::now().strftime("%Y%m%dT%H%M%S").to_string();
        let path = output_dir.join(format!("{}_{}.iq", job_name, timestamp));
        let file = File::create(&path)?;
        Ok(IqSink {
            writer: BufWriter::new(file),
        })
    }
}

impl Sink for IqSink {
    fn write(&mut self, samples: &[Complex<f32>]) -> anyhow::Result<()> {
        for s in samples {
            self.writer.write_all(&s.re.to_le_bytes())?;
            self.writer.write_all(&s.im.to_le_bytes())?;
        }
        Ok(())
    }
    fn finish(mut self: Box<Self>) -> anyhow::Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}
