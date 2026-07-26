use crate::sdr::sigmf_meta::SigmfMeta;
use anyhow::Result;
use num_complex::Complex;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

pub trait SdrSource {
    fn set_frequency(&mut self, freq: u64) -> Result<()>;
    fn set_sample_rate(&mut self, fs: u64) -> Result<()>;
    fn read_samples(&mut self, buf: &mut [Complex<f32>]) -> Result<usize>;
}

pub struct FileSdr {
    data_type: String,
    sample_rate: u64,
    raw_data: Vec<u8>,
    sigmf_data_file: File,
}

impl FileSdr {
    pub fn new(path: &str, data_size: usize) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let meta: SigmfMeta = serde_json::from_reader(reader)?;
        let data_file = File::open(PathBuf::from(path).with_extension("sigmf-data"))?;
        let buf = vec![0u8; data_size * 2];
        Ok(FileSdr {
            data_type: meta.global.datatype,
            sample_rate: meta.global.sample_rate,
            raw_data: buf,
            sigmf_data_file: data_file,
        })
    }
}

impl SdrSource for FileSdr {
    fn set_frequency(&mut self, freq: u64) -> Result<()> {
        Ok(())
    }

    fn set_sample_rate(&mut self, fs: u64) -> Result<()> {
        Ok(())
    }

    fn read_samples(&mut self, buf: &mut [Complex<f32>]) -> Result<usize> {
        let n = self.sigmf_data_file.read(&mut self.raw_data)?;
        println!("1024? {}", n);
        let mut count = 0;
        for (slot, iq) in buf.iter_mut().zip(self.raw_data[..n].chunks_exact(2)) {
            let i = (iq[0] as f32 - 127.5) / 127.5;
            let q = (iq[1] as f32 - 127.5) / 127.5;
            *slot = Complex::new(i, q);
            count += 1;
        }
        Ok(count)
    }
}
