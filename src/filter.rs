// filter.rs

pub struct Filter {
    cutoff: f32,
    resonance: f32,
    feedback: f32,
    buf0: f32,
    buf1: f32,
}

impl Filter {
    pub fn new(cutoff: f32, resonance: f32, feedback: f32) -> Filter {
        return Filter {
            cutoff: cutoff,
            resonance: resonance,
            feedback: feedback,
            buf0: 0.0,
            buf1: 1.0,
        }
    }

    fn calculate_feedback(&mut self) {
        self.feedback = self.resonance + self.resonance/(1.0 - self.cutoff);
    }

    fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
        self.calculate_feedback();
    }

    fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
        self.calculate_feedback();
    }

    fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        self.buf0 += self.cutoff * (input - self.buf0);
        self.buf1 += self.cutoff * (self.buf0 - self.buf1);
        return self.buf1;
        //return input - self.buf0;
        //return self.buf0 - self.buf1;
    }
}

