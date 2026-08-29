pub struct Squelch {
    open_thresh: f32,
    close_thresh: f32,
    is_open: bool,
}

impl Squelch {
    pub fn new(threshold_dbfs: f32) -> Self {
        // convert from logarithmic to linear scale
        let open = 10.0_f32.powf((threshold_dbfs + 0.5) / 20.0);
        let close = 10.0_f32.powf((threshold_dbfs - 0.5) / 20.0);
        Squelch {
            open_thresh: open,
            close_thresh: close,
            is_open: false,
        }
    }
    pub fn update(&mut self, magnitude: f32) -> bool {
        if self.is_open && magnitude < self.close_thresh {
            self.is_open = false;
        }
        if !self.is_open && magnitude > self.open_thresh {
            self.is_open = true;
        }
        self.is_open
    }
}
