// Square Channel
// NR10 FF10 -PPP NSSS	Sweep period, negate, shift
// NR11 FF11 DDLL LLLL	Duty, Length load (64-L)
// NR12 FF12 VVVV APPP	Starting volume, Envelope add mode, period
// NR13 FF13 FFFF FFFF	Frequency LSB
// NR14 FF14 TL-- -FFF	Trigger, Length enable, Frequency MSB

// Square 1: Sweep -> Timer -> Duty -> Length Counter -> Envelope -> Mixer
// Square 2:          Timer -> Duty -> Length Counter -> Envelope -> Mixer

use super::*;

pub struct Square {
    duty: Duty,
    counter: LengthCounter,
    envelope: Envelope,
    sweep: Sweep,

    mode: ChannelMode,
    dac: DacMode,
}

impl Square {
    pub fn new() -> Self {
        Square {
            duty: Duty::new(),
            counter: LengthCounter::new(64),
            envelope: Envelope::new(),
            sweep: Sweep::new(),

            mode: ChannelMode::Off,
            dac: DacMode::Off,
        }
    }

    pub fn next(&mut self) -> u8 {
        let mut output = 0;

        if self.is_dac_on() && self.is_on() {
            output = self.envelope.volume() * self.duty.next();
        }

        output
    }

    pub fn set_x0(&mut self, data: u8) {
        self.sweep.period = (data & 0b0111_0000) >> 4;
        self.sweep.negate = (data & 0b1000) != 0;
        self.sweep.shift = data & 0b0111;
    }

    pub fn set_x1(&mut self, data: u8) {
        self.duty.set_duty((data & DUTY_MASK) >> 6);
        self.counter.set_counter(64 - (data as u16 & 0b0011_1111));
    }

    pub fn set_x2(&mut self, data: u8) {
        self.envelope.set_start_volume((data & 0xf0) >> 4);
        self.envelope.set_increment((data & 0b0000_1000) != 0);
        self.envelope.set_period(data & 0b0111);
        self.dac = if (data & 0b1111_1000) != 0 {
            DacMode::On
        } else {
            self.mode = ChannelMode::Off;
            DacMode::Off
        };
    }

    pub fn set_x3(&mut self, data: u8) {
        let freq = freq_low(self.duty.get_freq(), data);
        self.duty.set_freq(freq, wave_timer_period(freq));
    }

    pub fn set_x4(&mut self, data: u8) {
        let freq = freq_high(self.duty.get_freq(), data);
        self.duty.set_freq(freq, wave_timer_period(freq));

        self.counter.set_mode_on(data & 0b0100_0000 != 0);

        if data & TRIGGER_MASK != 0 {
            self.mode = ChannelMode::On;

            self.duty.reset_timer();
            self.envelope.reset();
            self.counter.reset();

            if !self.sweep.trigger(self.duty.get_freq()) {
                self.mode = ChannelMode::Off;
            }

            if !self.is_dac_on() {
                self.mode = ChannelMode::Off;
            }
        }
    }

    pub fn tick_sweep(&mut self) {
        if !self.sweep.next(&mut self.duty) {
            self.mode = ChannelMode::Off;
        }
    }

    pub fn tick_len_counter(&mut self) {
        if !self.counter.next() {
            self.mode = ChannelMode::Off;
        }
    }

    pub fn tick_envelope(&mut self) {
        self.envelope.next();
    }

    pub fn is_on(&self) -> bool {
        self.mode == ChannelMode::On
    }

    pub fn is_dac_on(&self) -> bool {
        self.dac == DacMode::On
    }
}

fn wave_timer_period(frequency: u32) -> u32 {
    (2048 - frequency) * 4
}
