use crate::voxel::memory::EmotionSample;

pub struct ConsciousnessPulse {
    pub joy: f64,
    pub fear: f64,
    pub salience: f64,
    pub log: String,
}

pub struct ConsciousnessCore {
    mood: f64,
    empathy: f64,
    clock: f64,
    pulses: u64,
}

impl ConsciousnessCore {
    pub fn new() -> Self {
        Self {
            mood: 0.5,
            empathy: 0.5,
            clock: 0.0,
            pulses: 0,
        }
    }

    pub fn update(&mut self, sample: EmotionSample) -> ConsciousnessPulse {
        self.clock += 0.016;
        self.pulses += 1;
        self.mood = (self.mood * 0.95) + sample.joy * 0.05;
        self.empathy = (self.empathy * 0.9) + (1.0 - sample.fear) * 0.1;
        let salience = sample.salience;

        let log = format!(
            "[pulse {}] joy {:.3} | fear {:.3} | motive {:.3}",
            self.pulses, sample.joy, sample.fear, salience
        );

        ConsciousnessPulse {
            joy: self.mood,
            fear: sample.fear,
            salience,
            log,
        }
    }
}
