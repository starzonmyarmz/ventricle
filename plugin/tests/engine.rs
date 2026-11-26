mod support;

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

#[test]
fn gain_ramp_matches_golden() {
    let input = support::tone(110.0, 0.25, SR);
    support::check_golden("gain-ramp", &render(&input, Settings { gain: 0.5, ..Settings::default() }), SR as u32);
}

#[test]
fn unity_gain_passes_input_through() {
    let input = support::tone(110.0, 0.1, SR);
    assert_eq!(render(&input, Settings::default()), input);
}

#[test]
fn stomp_off_fades_to_the_dry_signal() {
    let input = support::tone(110.0, 1.0, SR);
    let output = render(&input, Settings { gain: 0.5, stomp: false });
    let tail = input.len() / 2;
    assert_eq!(output[tail..], input[tail..]);
}
