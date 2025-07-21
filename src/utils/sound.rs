use std::{cell::OnceCell, sync::Arc};

use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

use crate::{Shared, SAMPLE_RATE};

const SQUARE_WAVE_AMPLITUDE_DIV: i16 = 8;
const SQUARE_WAVE_EXTRA_SAMPLES: usize = 2048;

pub fn square_wave(value: i64, max: i64, length: usize) -> Vec<i16> {
    let frequency = 450.0 + (value as f32 * (500.0 / max as f32));

    (0..length + SQUARE_WAVE_EXTRA_SAMPLES).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        if (std::f32::consts::TAU * frequency * t).sin() >= 0.0 {
            i16::MAX / SQUARE_WAVE_AMPLITUDE_DIV
        } else {
            i16::MIN / SQUARE_WAVE_AMPLITUDE_DIV
        }
    }).collect()
}

mod midi {
    pub const CONTROLLER_COMMAND: i32 = 0xB0;
    pub const SET_PRESET_COMMAND: i32 = 0xC0;
    pub const SELECT_BANK_DATA1: i32 = 0x00;
}

pub struct BaseSoundFontEngine {
    pub synthesizer: OnceCell<Synthesizer>,
    pub decay: f64,
}

impl BaseSoundFontEngine {
    const VELOCITY: i32 = 127;
    const AMPLITUDE: f32 = 16384.0;
    const MASTER_VOLUME: f32 = 1.0;
    const DEFAULT_DECAY: f64 = 1.0;

    pub fn new() -> Self {
        Self { 
            synthesizer: OnceCell::new(), 
            decay: BaseSoundFontEngine::DEFAULT_DECAY 
        }
    }

    pub fn prepare(&mut self, shared: &Shared, sf: SoundFont, bank: i32, preset: i32) {
        let mut settings = SynthesizerSettings::new(SAMPLE_RATE as i32);
        settings.enable_reverb_and_chorus = shared.reverb;

        let mut synth = Synthesizer::new(&Arc::new(sf), &settings)
            .expect("Invalid synth configuration");

        synth.set_master_volume(BaseSoundFontEngine::MASTER_VOLUME);
        synth.process_midi_message(
            0, 
            midi::CONTROLLER_COMMAND, 
            midi::SELECT_BANK_DATA1, 
            bank
        );

        synth.process_midi_message(
            0, 
            midi::SET_PRESET_COMMAND, 
            preset, 
            0
        );

        let _ = self.synthesizer.set(synth);
    }

    pub fn play(&mut self, value: i64, max: i64, length: usize) -> Vec<i16> {
        let note = (24.0 + value as f64 * 67.0 / max as f64) as i32;
        
        if let Some(synth) = self.synthesizer.get_mut() {
            synth.note_on(0, note, BaseSoundFontEngine::VELOCITY);

            let full_length = length + (SAMPLE_RATE as f64 * self.decay) as usize;
            let mut left = vec![0f32; full_length];
            let mut right = vec![0f32; full_length];
            
            synth.render(&mut left[..length], &mut right[..length]);
            synth.note_off(0, note);
            synth.render(&mut left[length..], &mut right[length..]); // renders decay

            left.into_iter().enumerate().map(|(i, x)| ((x + right[i]) as f32 * BaseSoundFontEngine::AMPLITUDE) as i16).collect()
        } else {
            square_wave(value, max, length)
        }
    }
}