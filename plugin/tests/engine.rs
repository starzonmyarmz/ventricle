mod support;

use std::f64::consts::PI;

use ventricle::engine::{Engine, Settings};

const SR: f32 = 48_000.0;

/// Runs mono input through the engine in host-sized blocks and returns the
/// left output. The engine starts at its defaults and glides to `settings`.
fn render(input: &[f32], settings: Settings) -> Vec<f32> {
    let mut engine = Engine::new(SR);
    engine.set(settings);
    let mut left = input.to_vec();
    let mut right = input.to_vec();
    for (l, r) in left.chunks_mut(512).zip(right.chunks_mut(512)) {
        engine.process(l, r);
    }
    left
}

/// A buzzy, voice-like pulse train at `hz` with a little vibrato, swelling
/// from `from` to `to` peak level.
fn voice(hz: f64, seconds: f64, from: f64, to: f64) -> Vec<f32> {
    let sr = SR as f64;
    let n = (seconds * sr) as usize;
    let mut phase = 0.0;
    (0..n)
        .map(|i| {
            let t = i as f64 / sr;
            phase += hz * (1.0 + 0.01 * (2.0 * PI * 5.5 * t).sin()) / sr;
            let p = phase.fract();
            let pulse = if p < 0.4 { (PI * p / 0.4).sin().powi(2) } else { 0.0 };
            let amp = from + (to - from) * i as f64 / n as f64;
            (amp * (pulse - 0.2)) as f32
        })
        .collect()
}

fn sine(hz: f64, seconds: f64, amp: f64) -> Vec<f32> {
    let sr = SR as f64;
    (0..(seconds * sr) as usize).map(|i| (amp * (2.0 * PI * hz * i as f64 / sr).sin()) as f32).collect()
}

/// Energy at `hz` over the second half of `x`, by the Goertzel algorithm.
fn energy_at(x: &[f32], hz: f64) -> f64 {
    let x = &x[x.len() / 2..];
    let w = 2.0 * PI * hz / SR as f64;
    let c = 2.0 * w.cos();
    let (mut s1, mut s2) = (0.0, 0.0);
    for &v in x {
        let s = v as f64 + c * s1 - s2;
        s2 = s1;
        s1 = s;
    }
    (s1 * s1 + s2 * s2 - c * s1 * s2) / (x.len() as f64).powi(2)
}

fn quiet() -> Settings {
    Settings { growl: 0.0, rasp: 0.0, cup: 0.0, ..Settings::default() }
}

#[test]
fn swelling_voice_matches_golden() {
    let input = voice(160.0, 2.0, 0.02, 0.9);
    support::check_golden("voice-swell", &render(&input, Settings::default()), SR as u32);
}

#[test]
fn silence_stays_silent() {
    let output = render(&vec![0.0; 48_000], Settings { rasp: 1.0, growl: 1.0, ..Settings::default() });
    assert!(output.iter().all(|&x| x == 0.0));
}

#[test]
fn growl_adds_the_octave_below() {
    let input = voice(200.0, 1.0, 0.6, 0.6);
    let clean = energy_at(&render(&input, quiet()), 100.0);
    let growl = energy_at(&render(&input, Settings { growl: 1.0, ..quiet() }), 100.0);
    assert!(growl > 20.0 * clean, "growl {growl:e} vs clean {clean:e}");
}

#[test]
fn strain_makes_a_quiet_voice_crack() {
    let input = sine(220.0, 1.0, 0.05);
    let relaxed = energy_at(&render(&input, Settings { strain: 0.0, ..quiet() }), 660.0);
    let strained = energy_at(&render(&input, Settings { strain: 1.0, ..quiet() }), 660.0);
    assert!(strained > 10.0 * relaxed, "strained {strained:e} vs relaxed {relaxed:e}");
}

#[test]
fn rasp_adds_breath_between_the_harmonics() {
    let input = sine(220.0, 1.0, 0.5);
    let clean = energy_at(&render(&input, quiet()), 3_190.0);
    let rasp = energy_at(&render(&input, Settings { rasp: 1.0, ..quiet() }), 3_190.0);
    assert!(rasp > 10.0 * clean, "rasp {rasp:e} vs clean {clean:e}");
}

#[test]
fn loud_input_stays_bounded() {
    let input = voice(120.0, 1.0, 1.0, 1.0);
    let settings = Settings { strain: 1.0, growl: 1.0, rasp: 1.0, cup: 1.0, level: 12.0, ..Settings::default() };
    let output = render(&input, settings);
    assert!(output.iter().all(|x| x.is_finite() && x.abs() < 16.0));
}

#[test]
fn stomp_off_fades_to_the_dry_signal() {
    let input = voice(160.0, 1.0, 0.5, 0.5);
    let output = render(&input, Settings { stomp: false, ..Settings::default() });
    let tail = input.len() / 2;
    assert_eq!(output[tail..], input[tail..]);
}
