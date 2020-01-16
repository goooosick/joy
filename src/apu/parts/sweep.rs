use super::Duty;

pub struct Sweep {
    pub(crate) period: u8,
    pub(crate) counter: u8,

    pub(crate) negate: bool,
    pub(crate) shift: u8,

    pub(crate) shadow_freq: u32,
}

impl Sweep {
    pub fn new() -> Self {
        Sweep {
            period: 0,
            counter: 0,

            negate: false,
            shift: 0,

            shadow_freq: 0,
        }
    }

    pub fn next(&mut self, duty: &mut Duty) -> bool {
        if self.period > 0 && self.counter > 0 {
            self.counter -= 1;

            if self.counter == 0 {
                self.reset_counter();

                let new = self.calc_freq();
                if new <= 2047 {
                    if self.shift > 0 {
                        self.shadow_freq = new;
                        duty.set_freq(new, wave_timer_period(new));

                        let new = self.calc_freq();
                        if new > 2047 {
                            return false;
                        }
                    }
                } else {
                    return false;
                }
            }
        }

        true
    }

    pub fn calc_freq(&self) -> u32 {
        let new = self.shadow_freq >> self.shift;

        if self.negate {
            self.shadow_freq - new
        } else {
            self.shadow_freq + new
        }
    }

    pub fn trigger(&mut self, freq: u32) -> bool {
        self.shadow_freq = freq;
        self.reset_counter();

        if self.period > 0 && self.shift > 0 {
            let new = self.calc_freq();
            if new > 2047 {
                return false;
            }
        }

        true
    }

    fn reset_counter(&mut self) {
        self.counter = if self.period == 0 { 8 } else { self.period };
    }
}

fn wave_timer_period(frequency: u32) -> u32 {
    (2048 - frequency) * 4
}
