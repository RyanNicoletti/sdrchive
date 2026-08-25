use num_complex::Complex;

#[derive(Default)]
pub struct AmEnvelope {
    output: Vec<f32>,
}

impl AmEnvelope {
    pub fn run(&mut self, input: &[Complex<f32>]) -> &[f32] {
        self.output.clear();
        self.output.reserve(input.len());
        for &sample in input {
            self.output.push(sample.norm());
        }
        &self.output
    }
}
