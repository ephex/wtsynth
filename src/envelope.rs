// envelope.rs

pub const ENV_TYPE_ADSR: u16 = 0;

pub struct Envelope {
    envelope_type: u16,
    attack: u16,
    decay: u16,
    sustain: f32,
    release: u16,
}

impl Envelope {
    pub fn new(envelope_type: u16, attack: u16, decay: u16, sustain: f32, release: u16) -> Envelope {
        return Envelope {
            envelope_type: envelope_type,
            attack: attack,
            decay: decay,
            sustain: sustain,
            release: release,
        }
    }
}