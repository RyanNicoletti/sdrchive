use num_complex::Complex;

#[derive(Default)]
pub struct FmDiscriminator {
    prev: Option<Complex<f32>>,
    out: Vec<f32>,
}

impl FmDiscriminator {
    pub fn run_discriminator(&mut self, input: &[Complex<f32>]) -> &[f32] {
        if input.is_empty() {
            return &[];
        }
        let mut p = self.prev.unwrap_or(input[0]);
        self.out.clear();
        self.out.reserve(input.len());
        for x in input.iter() {
            let angle = (p * x.conj()).arg();
            self.out.push(angle);
            p = *x;
        }
        self.prev = Some(p);
        &self.out
    }
}
