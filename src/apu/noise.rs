// Noise
//      FF1F ---- ---- Not used
// NR41 FF20 --LL LLLL	Length load (64-L)
// NR42 FF21 VVVV APPP	Starting volume, Envelope add mode, period
// NR43 FF22 SSSS WDDD	Clock shift, Width mode of LFSR, Divisor code
// NR44 FF23 TL-- ----	Trigger, Length enable

// Noise:    Timer -> LFSR -> Length Counter -> Envelope -> Mixer

use super::*;

pub struct Noise {
    rand: LFSR,
    envelope: Envelope,
    counter: LengthCounter,

    mode: ChannelMode,
    dac: DacMode,
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            rand: LFSR::new(),
            envelope: Envelope::new(),
            counter: LengthCounter::new(64),

            mode: ChannelMode::Off,
            dac: DacMode::Off,
        }
    }

    pub fn next(&mut self) -> u8 {
        let mut output = 0;

        if self.is_dac_on() && self.is_on() {
            output = self.envelope.volume() * self.rand.next();
        }

        output
    }

    pub fn set_x0(&mut self, _: u8) {}

    pub fn set_x1(&mut self, data: u8) {
        self.counter.set_counter(64 - (data as u16 & 0b0011_1111));
    }

    pub fn set_x2(&mut self, data: u8) {
        self.envelope.set_start_volume((data & 0xf0) >> 4);
        self.envelope.set_increment((data & 0b0000_1000) != 0);
        self.envelope.set_period(data & 0b0111);
        self.dac = if (data & 0b1111_1000) != 0 {
            DacMode::On
        } else {
            DacMode::Off
        };
    }

    pub fn set_x3(&mut self, data: u8) {
        self.rand.set_state(data);
    }

    pub fn set_x4(&mut self, data: u8) {
        self.counter.set_mode_on(data & 0b0100_0000 != 0);

        if data & TRIGGER_MASK != 0 {
            self.mode = ChannelMode::On;

            self.envelope.reset();
            self.counter.reset();
            self.rand.reset();

            if !self.is_dac_on() {
                self.mode = ChannelMode::Off;
            }
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
