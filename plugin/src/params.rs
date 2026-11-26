use crate::engine::Settings;
use nice_plug::prelude::*;

#[derive(Params)]
pub struct VentricleParams {
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "stomp"]
    pub stomp: BoolParam,
}

impl Default for VentricleParams {
    fn default() -> Self {
        let s = Settings::default();
        Self {
            gain: FloatParam::new("Gain", s.gain as f32, FloatRange::gain_range(-30.0, 30.0))
                .with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            stomp: BoolParam::new("Stomp", s.stomp),
        }
    }
}

impl VentricleParams {
    pub fn settings(&self) -> Settings {
        Settings { gain: self.gain.value() as f64, stomp: self.stomp.value() }
    }
}
