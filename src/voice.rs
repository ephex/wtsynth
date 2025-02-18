// voice.rs

use crate::envelope::Envelope;
use crate::lfo::Lfo;

pub struct Voice {
    wave_table_type: u16,
    lfo: Lfo,
    amp_envelope: Envelope,
}

impl Voice {
    pub fn new(wave_table_type: u16, lfo: Lfo, amp_envelope: Envelope) -> Voice {
        return Voice {
            wave_table_type: wave_table_type,
            lfo: lfo,
            amp_envelope: amp_envelope,
        }
    }
}