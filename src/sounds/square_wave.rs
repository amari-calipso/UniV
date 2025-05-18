use crate::{sound, utils::sound::square_wave};

pub struct SquareWave;

sound! {
    name = "Square Wave";

    SquareWave::new(self) {
        SquareWave
    }

    play(value, max, length) {
        square_wave(value, max, length)
    }
}