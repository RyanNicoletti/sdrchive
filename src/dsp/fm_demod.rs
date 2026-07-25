use num_complex::Complex;

pub struct FmDemodulator {
    prev: Option<Complex<f32>>,
}

impl FmDemodulator {
    pub fn new() -> Self {
        FmDemodulator { prev: None }
    }
    pub fn demodulate(&mut self, iq: &[Complex<f32>], out: &mut Vec<f32>) {
        out.clear();
        let (mut prev, rest) = match self.prev {
            Some(p) => (p, iq),
            None => match iq.split_first() {
                Some((&first, rest)) => (first, rest),
                None => return,
            },
        };
        for &curr in rest {
            out.push((curr * prev.conj()).arg());
            prev = curr;
        }
        self.prev = Some(prev);
    }
}
