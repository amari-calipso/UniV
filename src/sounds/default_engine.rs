use std::io::Cursor;

use rustysynth::SoundFont;

use crate::{sound, utils::sound::BaseSoundFontEngine, DEFAULT_SOUNDFONT};

pub const BANK: i32 = 0;
pub const PRESET: i32 = 5;

pub struct DefaultEngine {
    base: BaseSoundFontEngine
}

sound! {
    name = "Default";

    DefaultEngine::new(self) {
        DefaultEngine { 
            base: BaseSoundFontEngine::new()
        }
    }

    prepare(shared, _gui, _rl, _thread) {
        let sf = SoundFont::new(&mut Cursor::new(DEFAULT_SOUNDFONT))
            .expect("Could not load default SoundFont");

        self.base.prepare(shared, sf, BANK, PRESET);
    }

    play(value, max, length) {
        self.base.play(value, max, length)
    }
}