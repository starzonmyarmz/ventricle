use std::sync::Arc;

use crate::engine::Settings;
use nice_plug::prelude::*;

#[derive(Params)]
pub struct VentricleParams {
    #[id = "strain"]
    pub strain: FloatParam,
    #[id = "growl"]
    pub growl: FloatParam,
    #[id = "rasp"]
    pub rasp: FloatParam,
    #[id = "vowel"]
    pub vowel: FloatParam,
    #[id = "cup"]
    pub cup: FloatParam,
    #[id = "level"]
    pub level: FloatParam,
    #[id = "stomp"]
    pub stomp: BoolParam,
}

fn amount(name: &str, value: f64) -> FloatParam {
    FloatParam::new(name, value as f32, FloatRange::Linear { min: 0.0, max: 1.0 })
        .with_unit("%")
        .with_value_to_string(formatters::v2s_f32_percentage(0))
        .with_string_to_value(formatters::s2v_f32_percentage())
}

const VOWELS: [&str; 5] = ["OH", "OH-AH", "AH", "AH-EH", "EH"];

fn vowel_name(value: f32) -> String {
    VOWELS[(value * 4.0).round().clamp(0.0, 4.0) as usize].to_string()
}

fn vowel_value(text: &str) -> Option<f32> {
    let text = text.trim();
    VOWELS.iter().position(|name| name.eq_ignore_ascii_case(text)).map(|i| i as f32 / 4.0)
}

impl Default for VentricleParams {
    fn default() -> Self {
        let s = Settings::default();
        Self {
            strain: amount("Strain", s.strain),
            growl: amount("Growl", s.growl),
            rasp: amount("Rasp", s.rasp),
            vowel: FloatParam::new("Vowel", s.vowel as f32, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_value_to_string(Arc::new(vowel_name))
                .with_string_to_value(Arc::new(vowel_value)),
            cup: amount("Cup", s.cup),
            level: FloatParam::new("Level", s.level as f32, FloatRange::Linear { min: -24.0, max: 12.0 })
                .with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_rounded(1)),
            stomp: BoolParam::new("Stomp", s.stomp),
        }
    }
}

impl VentricleParams {
    pub fn settings(&self) -> Settings {
        Settings {
            strain: self.strain.value() as f64,
            growl: self.growl.value() as f64,
            rasp: self.rasp.value() as f64,
            vowel: self.vowel.value() as f64,
            cup: self.cup.value() as f64,
            level: self.level.value() as f64,
            stomp: self.stomp.value(),
        }
    }
}
