#[derive(Eq, PartialEq)]
pub enum CounterMode {
    Counter,
    Continuous,
}

pub struct LengthCounter {
    counter: u16,

    max_len: u16,
    mode: CounterMode,
}

impl LengthCounter {
    pub fn new(max_len: u16) -> Self {
        LengthCounter {
            counter: 0,

            max_len,
            mode: CounterMode::Continuous,
        }
    }

    pub fn next(&mut self) -> bool {
        if self.is_on() && self.counter > 0 {
            self.counter -= 1;

            if self.counter == 0 {
                return false;
            }
        }

        true
    }

    pub fn reset(&mut self) {
        if self.counter == 0 {
            self.counter = self.max_len;
        }
        // self.mode = CounterMode::Counter;
    }

    pub fn set_counter(&mut self, counter: u16) {
        self.counter = counter;
    }

    pub fn set_mode_on(&mut self, counter: bool) {
        self.mode = if counter {
            CounterMode::Counter
        } else {
            CounterMode::Continuous
        };
    }

    pub fn is_on(&self) -> bool {
        self.mode == CounterMode::Counter
    }
}
