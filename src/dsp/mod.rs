use std::num::NonZeroUsize;

use num_complex::Complex;

struct Discriminator {
    prev: Complex<f32>,
    data: Vec<f32>,
}

impl Discriminator {
    pub fn process(&mut self, input: &[Complex<f32>]) -> &[f32] {}
}
