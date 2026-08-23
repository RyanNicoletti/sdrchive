use futuredsp::{DecimatingFirFilter, Filter};
pub struct StreamingFir<In, Out, TA> {
    pub kernel: DecimatingFirFilter<In, Out, TA>,
    decimation: usize,
    in_buf: Vec<In>,
    out: Vec<Out>,
}

impl<In, Out, TA> StreamingFir<In, Out, TA>
where
    DecimatingFirFilter<In, Out, TA>: Filter<In, Out, f32>,
    Out: Default + Copy,
    In: Copy,
{
    pub fn new(decimation: usize, taps: TA) -> Self {
        let filter = DecimatingFirFilter::new(decimation, taps);
        Self {
            kernel: filter,
            decimation,
            in_buf: Vec::new(),
            out: Vec::new(),
        }
    }
    pub fn run_filter(&mut self, chunk: &[In]) -> &[Out] {
        self.in_buf.extend_from_slice(chunk);
        let num_taps = self.kernel.length();
        let max_out_len = (self.in_buf.len() + 1).saturating_sub(num_taps) / self.decimation;
        self.out.resize(max_out_len, Out::default());
        let (consumed, produced, _status) = self.kernel.filter(&self.in_buf, &mut self.out);
        self.in_buf.drain(..consumed);
        &self.out[..produced]
    }
}
