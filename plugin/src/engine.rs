use std::f64::consts::PI;

/// The engine's values, in the units the controls show. The plugin builds
/// them from its params once per block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// How hard the input has to push before the voice cracks, 0 to 1.
    pub strain: f64,
    /// Subharmonics under the note, 0 to 1.
    pub growl: f64,
    /// Breath noise, 0 to 1.
    pub rasp: f64,
    /// 0 is OH, 0.5 AH and 1 EH.
    pub vowel: f64,
    /// Cupped-mic honk, 0 to 1.
    pub cup: f64,
    /// Output level in dB.
    pub level: f64,
    /// The footswitch. Off fades the effect out and passes the input.
    pub stomp: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { strain: 0.5, growl: 0.4, rasp: 0.4, vowel: 0.5, cup: 0.3, level: 0.0, stomp: true }
    }
}

/// First and second formants for OH, AH and EH, with the first raised a
/// little, as it is when screaming.
const FORMANTS: [(f64, f64); 3] = [(520.0, 880.0), (820.0, 1250.0), (650.0, 1850.0)];
const FORMANT_BOOST_DB: f64 = 9.0;
/// Loudness, in dBFS, at which the voice starts to crack, at Strain 0 and 1.
const CRACK_DB: (f64, f64) = (-8.0, -44.0);
/// How many dB past that point the crack takes to fully break.
const CRACK_SPAN_DB: f64 = 18.0;
/// The divider follows the input below this, where the voice's fundamental
/// lives.
const DIVIDER_HZ: f64 = 380.0;

fn db_to_gain(db: f64) -> f64 {
    10f64.powf(db / 20.0)
}

fn gain_to_db(gain: f64) -> f64 {
    20.0 * gain.max(1e-9).log10()
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
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

/// xorshift64*, seeded so renders repeat exactly.
struct Rng(u64);

impl Rng {
    const SEED: u64 = 0x5EED_F00D_CAFE_BABE;

    fn next(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    }

    fn bipolar(&mut self) -> f64 {
        self.next() * 2.0 - 1.0
    }
}

/// Andrew Simper's state-variable filter, which stays stable while its
/// frequency moves every sample.
#[derive(Clone, Copy, Default)]
struct Svf {
    ic1: f64,
    ic2: f64,
}

impl Svf {
    fn g(sample_rate: f64, hz: f64) -> f64 {
        (PI * hz.min(sample_rate * 0.45) / sample_rate).tan()
    }

    /// Returns (low-pass, band-pass, high-pass) for `g` from [`Svf::g`] and
    /// `k = 1 / Q`.
    fn tick(&mut self, x: f64, g: f64, k: f64) -> (f64, f64, f64) {
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let v3 = x - self.ic2;
        let v1 = a1 * self.ic1 + a2 * v3;
        let v2 = self.ic2 + a2 * self.ic1 + a3 * v3;
        self.ic1 = flush(2.0 * v1 - self.ic1);
        self.ic2 = flush(2.0 * v2 - self.ic2);
        (v2, v1, x - k * v1 - v2)
    }

    /// A bell that boosts by `gain` (linear) around the corner, with
    /// bandwidth `k = 1 / Q`.
    fn bell(&mut self, x: f64, g: f64, k: f64, gain: f64) -> f64 {
        let (_, bp, _) = self.tick(x, g, k);
        x + (gain - 1.0) * k * bp
    }
}

/// Splits the voice into alternate cycles, like an analog flip-flop octave
/// divider: a Schmitt trigger on the low-passed input toggles every time it
/// rises through zero.
#[derive(Default)]
struct Divider {
    filter: [Svf; 2],
    level: f64,
    armed: bool,
    /// +1 or -1, flipping once per cycle of the input.
    state: f64,
    /// `state`, slewed so the flips don't click.
    out: f64,
    /// A random loudness for this cycle, so the growl wobbles.
    shimmer: f64,
}

impl Divider {
    fn tick(&mut self, x: f64, g: f64, follow: f64, slew: f64, rng: &mut Rng) -> f64 {
        let (y, _, _) = self.filter[0].tick(x, g, 1.4);
        let (y, _, _) = self.filter[1].tick(y, g, 1.4);
        self.level = flush(self.level + (y.abs() - self.level) * follow);
        let hysteresis = 0.3 * self.level;
        if y < -hysteresis {
            self.armed = true;
        } else if self.armed && y > hysteresis {
            self.armed = false;
            self.state = if self.state > 0.0 { -1.0 } else { 1.0 };
            self.shimmer = rng.bipolar();
        }
        self.out = flush(self.out + (self.state - self.out) * slew);
        self.out
    }
}

/// The settings that glide, so turning a knob doesn't click.
#[derive(Clone, Copy)]
struct Smooth {
    strain: f64,
    growl: f64,
    rasp: f64,
    vowel: f64,
    cup: f64,
    gain: f64,
}

impl Smooth {
    fn target(s: &Settings) -> Self {
        Self { strain: s.strain, growl: s.growl, rasp: s.rasp, vowel: s.vowel, cup: s.cup, gain: db_to_gain(s.level) }
    }

    fn glide(&mut self, to: &Self, c: f64) {
        self.strain += (to.strain - self.strain) * c;
        self.growl += (to.growl - self.growl) * c;
        self.rasp += (to.rasp - self.rasp) * c;
        self.vowel += (to.vowel - self.vowel) * c;
        self.cup += (to.cup - self.cup) * c;
        self.gain += (to.gain - self.gain) * c;
    }
}

fn formants(vowel: f64) -> (f64, f64) {
    let at = vowel.clamp(0.0, 1.0) * 2.0;
    let i = (at as usize).min(1);
    let t = at - i as f64;
    (lerp(FORMANTS[i].0, FORMANTS[i + 1].0, t), lerp(FORMANTS[i].1, FORMANTS[i + 1].1, t))
}

/// The plugin's DSP, apart from the host plumbing so tests can run it
/// offline.
pub struct Engine {
    sample_rate: f32,
    glide: f64,
    settings: Settings,
    smooth: Smooth,
    /// How far the effect is engaged, gliding between 0 and 1 with the
    /// footswitch.
    wet: f64,
    attack: f64,
    release: f64,
    envelope: f64,
    crack_glide: f64,
    crack: f64,
    divider_g: f64,
    divider_follow: f64,
    divider_slew: f64,
    divider: Divider,
    rng: Rng,
    breath: [Svf; 2],
    breath_lo_g: f64,
    breath_hi_g: f64,
    pre: Svf,
    pre_g: f64,
    formant: [Svf; 2],
    proximity: Svf,
    proximity_g: f64,
    honk: Svf,
    honk_g: f64,
    muffle: Svf,
    dc_in: f64,
    dc_out: f64,
    dc_coef: f64,
}

impl Engine {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate as f64;
        let settings = Settings::default();
        let smooth = Smooth::target(&settings);
        Self {
            sample_rate,
            glide: coef(sr, 0.02),
            settings,
            smooth,
            wet: 1.0,
            attack: coef(sr, 0.002),
            release: coef(sr, 0.08),
            envelope: 0.0,
            crack_glide: coef(sr, 0.03),
            crack: 0.0,
            divider_g: Svf::g(sr, DIVIDER_HZ),
            divider_follow: coef(sr, 0.01),
            divider_slew: coef(sr, 0.0003),
            divider: Divider { state: 1.0, out: 1.0, ..Divider::default() },
            rng: Rng(Rng::SEED),
            breath: [Svf::default(); 2],
            breath_lo_g: Svf::g(sr, 700.0),
            breath_hi_g: Svf::g(sr, 6000.0),
            pre: Svf::default(),
            pre_g: Svf::g(sr, 110.0),
            formant: [Svf::default(); 2],
            proximity: Svf::default(),
            proximity_g: Svf::g(sr, 160.0),
            honk: Svf::default(),
            honk_g: Svf::g(sr, 1100.0),
            muffle: Svf::default(),
            dc_in: 0.0,
            dc_out: 0.0,
            dc_coef: 1.0 - coef(sr, 0.02),
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Jumps to the current settings instead of gliding to them.
    pub fn reset(&mut self) {
        let settings = self.settings;
        *self = Self::new(self.sample_rate);
        self.settings = settings;
        self.smooth = Smooth::target(&settings);
        self.wet = if settings.stomp { 1.0 } else { 0.0 };
    }

    pub fn set(&mut self, settings: Settings) {
        self.settings = settings;
    }

    /// 0 while the voice is clean, rising to 1 as the input pushes past the
    /// point Strain sets.
    fn crack_target(&self, strain: f64) -> f64 {
        let start = lerp(CRACK_DB.0, CRACK_DB.1, strain);
        ((gain_to_db(self.envelope) - start) / CRACK_SPAN_DB).clamp(0.0, 1.0)
    }

    fn scream(&mut self, x: f64) -> f64 {
        let sr = self.sample_rate as f64;
        let s = self.smooth;

        let level = x.abs();
        let c = if level > self.envelope { self.attack } else { self.release };
        self.envelope = flush(self.envelope + (level - self.envelope) * c);
        let target = self.crack_target(s.strain);
        self.crack = flush(self.crack + (target - self.crack) * self.crack_glide);
        let crack = self.crack;

        let flip = self.divider.tick(x, self.divider_g, self.divider_follow, self.divider_slew, &mut self.rng);
        let growl = 0.6 * s.growl * (0.3 + 0.7 * crack);
        let shimmer = 1.0 + 0.35 * s.growl * self.divider.shimmer;
        let mut y = x * (1.0 - growl) + x * flip * growl * shimmer;

        let noise = self.rng.bipolar();
        let (_, _, noise) = self.breath[0].tick(noise, self.breath_lo_g, 1.4);
        let (noise, _, _) = self.breath[1].tick(noise, self.breath_hi_g, 1.4);
        let rasp = s.rasp * (0.25 + 0.75 * crack);
        y += 1.6 * rasp * noise * (0.4 * self.envelope + 0.6 * level);

        let (_, _, y_hp) = self.pre.tick(y, self.pre_g, 1.4);
        let drive = db_to_gain(lerp(0.0, 26.0, crack) + 4.0 * s.strain);
        let bias = 0.35;
        let y = ((drive * y_hp + bias).tanh() - bias.tanh()) / drive.sqrt();

        let (f1, f2) = formants(s.vowel);
        let boost = db_to_gain(FORMANT_BOOST_DB * (0.5 + 0.5 * crack));
        let y = self.formant[0].bell(y, Svf::g(sr, f1), 1.0 / 3.0, boost);
        let y = self.formant[1].bell(y, Svf::g(sr, f2), 1.0 / 4.0, boost);

        let (lows, _, _) = self.proximity.tick(y, self.proximity_g, 1.4);
        let y = y + 1.2 * s.cup * lows;
        let y = self.honk.bell(y, self.honk_g, 1.0 / 1.3, db_to_gain(12.0 * s.cup));
        let (y, _, _) = self.muffle.tick(y, Svf::g(sr, lerp(18_000.0, 3_200.0, s.cup)), 1.4);

        let out = y - self.dc_in + self.dc_coef * self.dc_out;
        self.dc_in = y;
        self.dc_out = flush(out);
        self.dc_out
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let target = Smooth::target(&self.settings);
        let wet_target = if self.settings.stomp { 1.0 } else { 0.0 };
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            self.smooth.glide(&target, self.glide);
            self.wet = flush(self.wet + (wet_target - self.wet) * self.glide);
            let (dl, dr) = (*l as f64, *r as f64);
            let scream = self.scream(0.5 * (dl + dr)) * self.smooth.gain;
            *l = (dl + (scream - dl) * self.wet) as f32;
            *r = (dr + (scream - dr) * self.wet) as f32;
        }
    }
}
