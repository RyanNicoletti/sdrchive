use std::f32::consts::PI;

pub struct DcBlocker {
    prev_input: f32,
    prev_output: f32,
    r: f32,
    output: Vec<f32>,
}

impl DcBlocker {
    pub fn new(sample_rate_hz: u32, cutoff_hz: f32) -> Self {
        let r = (-2.0 * PI * cutoff_hz / sample_rate_hz as f32).exp();
        Self {
            prev_input: 0.0,
            prev_output: 0.0,
            r,
            output: Vec::new(),
        }
    }
    pub fn run(&mut self, input: &[f32]) -> &[f32] {
        self.output.clear();
        self.output.reserve(input.len());

        for &sample in input {
            let out = sample - self.prev_input + self.r * self.prev_output;
            self.prev_input = sample;
            self.prev_output = out;
            self.output.push(out);
        }
        &self.output
    }
}
