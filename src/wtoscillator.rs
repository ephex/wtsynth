// wtoscillator.rs
//
// Wave table oscillator.

use rodio::source::Source;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::filter::Filter;

pub const WAVE_TYPE_SINE: u8 = 0;
pub const WAVE_TYPE_SAW: u8 = 1;
pub const WAVE_TYPE_SQUARE: u8 = 2;
pub const WAVE_TYPE_TRI: u8 = 3;


fn wavetable_sine() -> Vec<f32> {
    let wave_table_size = 64;
    let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
    for n in 0..wave_table_size {
        wave_table.push((2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin());
    }
    return wave_table;
}

fn wavetable_saw() -> Vec<f32> {
    let wave_table_size = 64;
    let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
    for n in 0..wave_table_size {
        wave_table.push(1.0 - 2.0 * (n as f32 / wave_table_size as f32));
    }
    return wave_table;
}

fn wavetable_square(pulse_width: f32) -> Vec<f32> {
    let wave_table_size = 64;
    let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
    for n in 0..wave_table_size {
        let mut val: f32 = 1.0;
        if n < ((wave_table_size / 2) as f32 * pulse_width) as usize {
            val = -1.0;
        }
        wave_table.push(val);
    }
    return wave_table;
}

fn wavetable_tri() -> Vec<f32> {
    let wave_table_size = 64;
    let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
    for n in 0..wave_table_size {
        if n < wave_table_size / 2 {
            wave_table.push((-4.0 / wave_table_size as f32) * (n as f32) + 1.0);
        }
        else {
            wave_table.push((4.0 / wave_table_size as f32) * (n as f32) - 3.0);

        }
    }
    return wave_table;
}

pub struct WavetableOscillator {
    note_on: bool,
    note_on_time: SystemTime,
    note_off_time: SystemTime,
    sample_rate: u32,
    wave_table: Vec<f32>,
    index: f32,
    index_increment: f32,
    amplitude: f32,
    voice_amplitude: f32,
    attack: u16,
    decay: u16,
    sustain: f32,
    release: u16,
    singing: Arc<Mutex<bool>>,
    filter: Filter,
}


impl WavetableOscillator {
    pub fn new(sample_rate: u32, wave_table_type: u8, singing: Arc<Mutex<bool>>, filter_cutoff: f32, filter_resonance: f32, pulse_width: f32) -> WavetableOscillator {
        let wave_table: Vec<f32> = match wave_table_type {
            WAVE_TYPE_SINE => {
                wavetable_sine()
            },
            WAVE_TYPE_SAW => {
                wavetable_saw()
            },
            WAVE_TYPE_SQUARE => {
                wavetable_square(pulse_width)
            }
            WAVE_TYPE_TRI => {
                wavetable_tri()
            }
            _ =>  {
                wavetable_sine()
            }
        };
        return WavetableOscillator {
            note_on: true,
            note_on_time: SystemTime::now(),
            note_off_time: SystemTime::now(),
            sample_rate: sample_rate,
            wave_table: wave_table,
            index: 0.0,
            index_increment: 0.0,
            amplitude: 1.0,
            voice_amplitude: 0.0,
            attack: 0,
            decay: 0,
            sustain: 0.0,
            release: 0,
            singing: singing,
            filter: Filter::new(filter_cutoff, filter_resonance, 0.0)
        }
    }

    pub fn set_attack(&mut self, attack: u16) {
        self.attack = attack;
    }

    pub fn set_decay(&mut self, decay: u16) {
        self.decay = decay;
    }

    pub fn set_sustain(&mut self, sustain: f32) {
        self.sustain = sustain;
    }

    pub fn set_release(&mut self, release: u16) {
        self.release = release;
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.index_increment = frequency * self.wave_table.len() as f32 / self.sample_rate as f32;
    }

    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }

    pub fn set_note_on(&mut self, note_on: bool) {
        self.note_on = note_on;
        if !note_on {
            self.note_off_time = SystemTime::now();
        }
    }

    fn get_amplitude(&mut self) -> f32 {
        let mut amp: f32 = self.amplitude;
        if self.note_on {
            match self.note_on_time.elapsed() {
                Ok(elapsed) => {
                    // Attack amplitude envelope.
                    if self.attack > 0 {
                        if elapsed.as_millis() < self.attack as u128  {
                            amp *= (elapsed.as_millis() as f32 / self.attack as f32);
                        }
                    }
                    if elapsed.as_millis() > self.attack as u128  {
                        // Attack done, start decay and sustain.
                        if self.decay > 0 {
                            let elapsed_since_attack = elapsed.as_millis() - self.attack as u128;
                            if elapsed_since_attack < self.decay as u128  {
                                amp -= (self.amplitude - self.amplitude * self.sustain) * (elapsed_since_attack as f32 / self.decay as f32);
                            }
                            else {
                                amp *= self.sustain;
                            }
                        }
                        else {
                            amp *= self.sustain;
                        }
                    }
                }
                Err(e) => {
                    println!("Error getting amplitude: {:?}", e);
                }
            }
            self.voice_amplitude = amp;
        }
        else {
            // Release amplitude envelope.
            amp = self.voice_amplitude;
            if self.release > 0 {
                match self.note_off_time.elapsed() {
                    Ok(elapsed) => {
                        if elapsed.as_millis() < self.release as u128 {
                            amp *= 1.0 - (elapsed.as_millis() as f32 / self.release as f32);
                        }
                        else {
                            amp = 0.0;
                        }
                    }
                    Err(e) => {
                        println!("Error getting amplitude: {:?}", e);
                    }
                }
            }
        }
        return amp;
    }

    fn get_sample(&mut self) -> f32 {
        // Check for singing mutex bool value.
        if self.note_on {
            if !*self.singing.lock().unwrap() {
                self.set_note_on(false);
            }
        }
        let sample = self.lerp();
        self.index += self.index_increment;
        self.index %= self.wave_table.len() as f32;
        //return self.get_amplitude() * sample;
        let ampl = self.get_amplitude();
        let filtered = self.filter.process(sample);
        return ampl * filtered;
        //return ampl * sample;
    }

    fn lerp(&mut self) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % self.wave_table.len();

        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        return (truncated_index_weight * self.wave_table[truncated_index] + next_index_weight * self.wave_table[next_index]);
    }
}

impl Iterator for WavetableOscillator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        return Some(self.get_sample());
    }
}

impl Source for WavetableOscillator {
    fn channels(&self) -> u16 {
        return 1;
    }

    fn sample_rate(&self) -> u32 {
        return self.sample_rate;
    }

    fn current_frame_len(&self) -> Option<usize> {
        return None;
    }

    fn total_duration(&self) -> Option<Duration> {
        return None;
    }
}
