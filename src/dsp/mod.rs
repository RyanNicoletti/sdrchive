use std::num::NonZeroUsize;

use num_complex::Complex;

pub fn measure_phase_change(input_samples: &[Complex<f32>]) -> Vec<f32> {
    // measures phase change per sample
    input_samples
        .windows(2)
        .map(|pair| (pair[1] * pair[0].conj()).arg())
        .collect()
}

pub fn decimate<T: Copy>(input_samples: &[T], factor: NonZeroUsize) -> Vec<T> {
    input_samples
        .iter()
        .step_by(factor.get())
        .copied()
        .collect()
}

pub fn filter(input_samples: &[f32], taps: &[f32]) -> Vec<f32> {
    input_samples
        .windows(taps.len())
        .map(|win| {
            win.iter()
                .zip(taps.iter().rev())
                .map(|(&sample, &tap)| sample * tap)
                .sum::<f32>()
        })
        .collect()
}
