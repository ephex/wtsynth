// note.rs

use rodio::{OutputStream, source::Source};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::wtoscillator::WavetableOscillator;

struct NoteParameters {
    amplitude: f32,
    frequency: f32,
    attack: u16,
    decay: u16,
    sustain: f32,
    release: u16,
    filter_cutoff: f32,
    filter_resonance: f32,
    note_on: Arc<Mutex<bool>>,
}

impl NoteParameters {
    fn new(frequency: f32, amplitude: f32, attack: u16, decay: u16, sustain: f32, release: u16, filter_cutoff: f32, filter_resonance: f32, note_on: Arc<Mutex<bool>>) -> NoteParameters {
        return NoteParameters {
            amplitude: amplitude,
            frequency: frequency,
            attack: attack,
            decay: decay,
            sustain: sustain,
            release: release,
            filter_cutoff: filter_cutoff,
            filter_resonance: filter_resonance,
            note_on: note_on,
        }
    }
}

pub struct Note {
    thread: JoinHandle<()>,
    parameters: NoteParameters,
}

impl Note {
    pub fn new(wave_table_type: u8, frequency: f32, amplitude: f32) -> Note {
        let note_on = Arc::new(Mutex::new(true));
        let note_parameters = NoteParameters::new(frequency, amplitude, 100, 100, 0.3, 300, 0.4, 0.0, Arc::clone(&note_on));
        let handle = std::thread::spawn(move ||  {
            let mut oscillator = WavetableOscillator::new(44100, wave_table_type, Arc::clone(&note_on), 0.2, 1.0);
            oscillator.set_frequency(note_parameters.frequency);
            oscillator.set_amplitude(note_parameters.amplitude);
            // Set attack, delay, sustain, release.
            oscillator.set_attack(note_parameters.attack);
            oscillator.set_decay(note_parameters.decay);
            oscillator.set_sustain(note_parameters.sustain);
            oscillator.set_release(note_parameters.release);

            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let _result = stream_handle.play_raw(oscillator.convert_samples());
            while *note_on.lock().unwrap() {}
            // Allow thread to live until release is done.
            std::thread::sleep(std::time::Duration::from_millis(note_parameters.release as u64 + 300 as u64));
        });
        return Note {
            thread: handle,
            parameters: note_parameters
        }
    }

    pub fn stop(&self) {
        *self.parameters.note_on.lock().unwrap() = false;
    }

}