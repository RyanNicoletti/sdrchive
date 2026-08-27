pub struct Agc {
    gain: f32,
    target: f32,
    attack: f32,
    decay: f32,
    max_gain: f32,
    output: Vec<f32>,
}

impl Agc {
    pub fn new(target: f32, attack: f32, decay: f32, max_gain: f32) -> Self {
        Agc {
            gain: 1.0,
            target,
            attack,
            decay,
            max_gain,
            output: Vec::new(),
        }
    }
    pub fn run(&mut self, input: &[f32]) -> &[f32] {
        self.output.clear();
        self.output.reserve(input.len());
        for &x in input {
            let y = self.gain * x;
            let error = self.target - y.abs();
            let rate = if y.abs() > self.target {
                self.attack
            } else {
                self.decay
            };
            self.gain += rate * error;
            self.gain = self.gain.clamp(0.0, self.max_gain);
            self.output.push(y);
        }
        &self.output
    }
}
