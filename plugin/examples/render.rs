//! Runs a WAV file through the engine, so you can hear a change without a
//! host.
//!
//! cargo run --release --example render -- in.wav out.wav [strain growl rasp vowel cup]
//!
//! The knob values are 0 to 1 and default to the plugin's defaults.

use ventricle::engine::{Engine, Settings};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [input, output, knobs @ ..] = args.as_slice() else {
        eprintln!("usage: render in.wav out.wav [strain growl rasp vowel cup]");
        std::process::exit(2);
    };

    let mut settings = Settings::default();
    let fields =
        [&mut settings.strain, &mut settings.growl, &mut settings.rasp, &mut settings.vowel, &mut settings.cup];
    for (field, value) in fields.into_iter().zip(knobs) {
        *field = value.parse().unwrap_or_else(|_| panic!("not a number: {value}"));
    }

    let mut reader = hound::WavReader::open(input).unwrap_or_else(|e| panic!("{input}: {e}"));
    let spec = reader.spec();
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().map(Result::unwrap).collect(),
        hound::SampleFormat::Int => {
            let scale = (1u32 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>().map(|s| s.unwrap() as f32 / scale).collect()
        }
    };
    let channels = spec.channels as usize;
    let mut left: Vec<f32> = samples.chunks(channels).map(|frame| frame[0]).collect();
    let mut right: Vec<f32> = samples.chunks(channels).map(|frame| frame[channels.min(2) - 1]).collect();

    let mut engine = Engine::new(spec.sample_rate as f32);
    engine.set(settings);
    engine.reset();
    for (l, r) in left.chunks_mut(512).zip(right.chunks_mut(512)) {
        engine.process(l, r);
    }

    let out_spec = hound::WavSpec {
        channels: 2,
        sample_rate: spec.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(output, out_spec).unwrap();
    for (l, r) in left.iter().zip(&right) {
        writer.write_sample(*l).unwrap();
        writer.write_sample(*r).unwrap();
    }
    writer.finalize().unwrap();
    println!("{settings:?}");
}
