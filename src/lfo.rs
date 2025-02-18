// lfo.rs

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

pub struct Lfo {
    frequency: Arc<Mutex<f32>>,
    start: SystemTime,
}

impl Lfo {
    pub fn new(frequency: f32) -> Lfo {
        return Lfo {
            frequency: Arc::new(Mutex::new(frequency)),
            start: SystemTime::now(),
        }
    }

    pub fn get_value(&self) -> f32 {
        match self.start.elapsed() {
            Ok(elapsed) => {
                return (f32::cos(*self.frequency.lock().unwrap() * (elapsed.as_millis() as f32 / 1000.0) * 2.0 * std::f32::consts::PI) + 1.0) / 2.0
            },
            Err(e) => {
                println!("Error getting lfo value: {:?}", e);
                return 1.0
            }
        }
    }
}