/// The engine's values, in the units the controls show. The plugin builds
/// them from its params once per block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Linear gain.
    pub gain: f64,
    /// The footswitch. Off fades the effect out and passes the input.
    pub stomp: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { gain: 1.0, stomp: true }
    }
}

/// The coefficient for a one-pole smoother with a time constant of
/// `seconds`: `x += (target - x) * coef`.
fn coef(sample_rate: f64, seconds: f64) -> f64 {
    1.0 - (-1.0 / (sample_rate * seconds)).exp()
}

/// Zeroes values too small to hear, so anything decaying toward zero never
/// reaches subnormals, which are slow on x86 and which hosts reject in
/// output.
fn flush(x: f64) -> f64 {
    if x.abs() < 1e-30 {
        0.0
    } else {
        x
    }
}

/// The plugin's DSP, apart from the host plumbing so tests can run it
/// offline.
pub struct Engine {
    sample_rate: f32,
    glide: f64,
    settings: Settings,
    gain: f64,
    /// How far the effect is engaged, gliding between 0 and 1 with the
    /// footswitch.
    wet: f64,
}

impl Engine {
    pub fn new(sample_rate: f32) -> Self {
        let settings = Settings::default();
        Self { sample_rate, glide: coef(sample_rate as f64, 0.02), settings, gain: settings.gain, wet: 1.0 }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Jumps to the current settings instead of gliding to them.
    pub fn reset(&mut self) {
        self.gain = self.settings.gain;
        self.wet = if self.settings.stomp { 1.0 } else { 0.0 };
    }

    pub fn set(&mut self, settings: Settings) {
        self.settings = settings;
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let s = self.settings;
        let wet_target = if s.stomp { 1.0 } else { 0.0 };
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            self.gain += (s.gain - self.gain) * self.glide;
            self.wet = flush(self.wet + (wet_target - self.wet) * self.glide);
            let gain = 1.0 + (self.gain - 1.0) * self.wet;
            *l = (*l as f64 * gain) as f32;
            *r = (*r as f64 * gain) as f32;
        }
    }
}
