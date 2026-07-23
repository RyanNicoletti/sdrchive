use anyhow::Result;
use num_complex::Complex;
use std::fs::File;
use std::io::Read;

trait SdrSource {
    fn set_frequency(&mut self, freq: u64) -> Result<()>;
    fn set_sample_rate(&mut self, fs: u64) -> Result<()>;
    fn read_samples(&mut self, buf: &mut [Complex<f32>]) -> Result<usize>;
}

struct FileSdr {
    f: File,
    file_data: Vec<u8>,
}

impl FileSdr {
    fn new(path: &str, buflen: usize) -> Result<Self> {
        let f = File::open(path)?;
        let buf = vec![0u8; buflen * 2];
        Ok(FileSdr { f, file_data: buf })
    }
}

impl SdrSource for FileSdr {
    fn set_frequency(&mut self, freq: u64) -> Result<()> {
        todo!()
    }

    fn set_sample_rate(&mut self, fs: u64) -> Result<()> {
        todo!()
    }

    fn read_samples(&mut self, buf: &mut [Complex<f32>]) -> Result<usize> {
        let n = self.f.read(&mut self.file_data)?;
        let mut count = 0;
        for (slot, iq) in buf.iter_mut().zip(self.file_data[..n].chunks_exact(2)) {
            let i = (iq[0] as f32 - 127.5) / 127.5;
            let q = (iq[1] as f32 - 127.5) / 127.5;
            *slot = Complex::new(i, q);
            count += 1;
        }
        Ok(count)
    }
}

fn main() -> Result<()> {
    let mut buf = vec![Complex::new(0.0, 0.0); 512];
    let mut sdr = FileSdr::new("src/test_1khz_fm.iq", buf.len())?;
    let n = sdr.read_samples(&mut buf)?;
    println!("read {n} samples; first = {:?}", buf[0]);
    Ok(())
}
