use std::{collections::VecDeque, iter::repeat};

use crate::{POLYPHONY_LIMIT, SAMPLE_RATE};

const ENFORCE_POLYPHONY_LIMIT: bool = false; // sounds better without it, since the mixer can't tell if part of a sample is silent

pub struct SoundMixer {
    buffer: VecDeque<VecDeque<i16>>,
    pub output: Vec<i16>
}

impl SoundMixer {
    pub fn new() -> Self {
        SoundMixer {
            buffer: VecDeque::with_capacity(POLYPHONY_LIMIT),
            output: Vec::new()
        }
    }

    /// Resets the buffer
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    /// Converts a duration (in seconds) to an amount of samples
    pub fn duration_to_samples(secs: f64) -> usize {
        (SAMPLE_RATE as f64 * secs) as usize
    }

    /// Removes all empty samples from the buffer
    pub fn cleanup(&mut self) {
        self.buffer.retain(|x| !x.is_empty());
    }

    /// Consumes the buffer and outputs the requested amount of mixed samples
    pub fn get(&mut self, amt: usize) {        
        self.output.clear();
        
        if amt > self.output.capacity() {
            self.output.reserve(amt - self.output.capacity());
        }

        for _ in 0 .. amt {
            let mut value = 0i16;
            for i in 0 .. self.buffer.len() {
                if !self.buffer[i].is_empty() {
                    value = value.saturating_add(self.buffer[i].pop_front().unwrap());
                }
            }

            self.output.push(value);
        }
    }

    /// Mixes N samples of silence without advancing
    pub fn put_silence(&mut self, amt: usize) {
        self.buffer.push_back(VecDeque::from_iter(repeat(0).take(amt)));
    }

    /// Mixes new samples
    pub fn put(&mut self, new_samples: Vec<i16>) {
        if ENFORCE_POLYPHONY_LIMIT && self.buffer.len() == POLYPHONY_LIMIT {
            self.buffer.pop_front();
        }

        self.put_silence(new_samples.len());
        
        let curr = self.buffer.iter_mut().last().unwrap();
        for i in 0 .. new_samples.len() {
            curr[i] = new_samples[i];
        }
    }
}