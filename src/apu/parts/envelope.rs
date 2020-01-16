pub enum EnvelopeMode {
    Inc,
    Dec,
}

pub struct Envelope {
    period: u8,
    counter: u8,

    volume: u8,
    start_volume: u8,

    mode: EnvelopeMode,
}

impl Envelope {
    pub fn new() -> Self {
        Envelope {
            period: 0,
            counter: 0,

            volume: 0,
            start_volume: 0,

            mode: EnvelopeMode::Inc,
        }
    }

    pub fn next(&mut self) {
        // Note: refrence -> binjgb
        if self.period > 0 && self.counter > 0 {
            self.counter -= 1;

            if self.counter == 0 {
                self.counter = self.period;

                let volume = match self.mode {
                    EnvelopeMode::Inc => self.volume + 1,
                    EnvelopeMode::Dec => self.volume.wrapping_sub(1),
                };

                if volume < 16 {
                    self.volume = volume;
                } else {
                    self.counter = 0;
                }
            }
        }
    }

    pub fn volume(&self) -> u8 {
        self.volume
    }

    pub fn reset(&mut self) {
        self.counter = if self.period > 0 { self.period } else { 8 };
        self.volume = self.start_volume;
    }

    pub fn set_start_volume(&mut self, volume: u8) {
        self.start_volume = volume;
    }

    pub fn set_period(&mut self, period: u8) {
        self.period = period;
    }

    pub fn set_increment(&mut self, increment: bool) {
        self.mode = if increment {
            EnvelopeMode::Inc
        } else {
            EnvelopeMode::Dec
        };
    }
}
