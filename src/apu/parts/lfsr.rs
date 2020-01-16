use super::Timer;

const DIVISOR: [u32; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

#[derive(Eq, PartialEq)]
pub enum WidthMode {
    Low,
    High,
}

pub struct LFSR {
    timer: Timer,

    shift_reg: u16,
    mode: WidthMode,
}

impl LFSR {
    pub fn new() -> Self {
        LFSR {
            timer: Timer::new(0),

            shift_reg: 0x7fff,
            mode: WidthMode::High,
        }
    }

    pub fn next(&mut self) -> u8 {
        if self.timer.tick() {
            self.randomize();
        }
        (!self.shift_reg & 0b01) as u8
    }

    fn randomize(&mut self) {
        let xor = ((self.shift_reg & 0b0010) >> 1) ^ (self.shift_reg & 0b0001);
        self.shift_reg >>= 1;
        self.shift_reg = if self.mode == WidthMode::Low {
            (self.shift_reg & 0x7fbf) | (xor << 6)
        } else {
            (self.shift_reg & 0x3fff) | (xor << 14)
        };
    }

    pub fn set_state(&mut self, data: u8) {
        self.mode = if data & 0b0000_1000 != 0 {
            WidthMode::Low
        } else {
            WidthMode::High
        };

        let shift = (data & 0xf0) >> 4;
        let code = (data & 0b0111) as usize;
        self.timer.set_period(DIVISOR[code] << shift);
    }

    pub fn reset(&mut self) {
        self.timer.reset();
        self.shift_reg = 0x7fff;
    }
}
