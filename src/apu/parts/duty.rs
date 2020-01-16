use super::Timer;

const SQUARE_WAVE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

pub struct Duty {
    duty: usize,
    step: usize,

    freq: u32,
    timer: Timer,
}

impl Duty {
    pub fn new() -> Self {
        Duty {
            duty: 0,
            step: 0,

            freq: 0,
            timer: Timer::new(std::u32::MAX),
        }
    }

    pub fn next(&mut self) -> u8 {
        if self.timer.tick() {
            self.step = (self.step + 1) % 8;
        }

        SQUARE_WAVE[self.duty][self.step]
    }

    pub fn set_duty(&mut self, data: u8) {
        self.duty = data as usize;
    }

    pub fn set_freq(&mut self, freq: u32, period: u32) {
        self.freq = freq;
        self.timer.set_period(period);
    }

    pub fn reset_timer(&mut self) {
        self.timer.reset();
    }

    pub fn get_freq(&self) -> u32 {
        self.freq
    }
}
