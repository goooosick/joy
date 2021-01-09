pub struct Timer {
    period: u32,
    counter: u32,
}

impl Timer {
    pub fn new(period: u32) -> Self {
        Timer {
            period: period,
            counter: 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.counter > 0 {
            self.counter -= 1;
            false
        } else {
            self.counter = self.period;
            true
        }
    }

    pub fn set_period(&mut self, period: u32) {
        self.period = period;
    }

    pub fn reset(&mut self) {
        self.counter = self.period;
    }
}
