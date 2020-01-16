// Wave
// NR30 FF1A E--- ----	DAC power
// NR31 FF1B LLLL LLLL	Length load (256-L)
// NR32 FF1C -VV- ----	Volume code (00=0%, 01=100%, 10=50%, 11=25%)
// NR33 FF1D FFFF FFFF	Frequency LSB
// NR34 FF1E TL-- -FFF	Trigger, Length enable, Frequency MSB

// Wave:     Timer -> Wave -> Length Counter -> Volume   -> Mixer

// Wave Table
// FF30 0000 1111	Samples 0 and 1
// ....
// FF3F 0000 1111	Samples 30 and 31

use super::*;

const VOLUME_SHIFT: [u8; 4] = [4, 0, 1, 2];

pub struct Wave {
    wave_table: WaveTable,
    counter: LengthCounter,

    volume_shift: u8,

    mode: ChannelMode,
    dac: DacMode,
}

impl Wave {
    pub fn new() -> Self {
        Wave {
            wave_table: WaveTable::new(),
            counter: LengthCounter::new(256),

            volume_shift: 0,

            mode: ChannelMode::Off,
            dac: DacMode::Off,
        }
    }

    pub fn next(&mut self) -> u8 {
        let mut output = 0;

        if self.is_dac_on() && self.is_on() {
            output = self.wave_table.next() >> self.volume_shift;
        }

        output
    }

    pub fn write_wave(&mut self, addr: u16, data: u8) {
        if !self.is_on() {
            self.wave_table.set_entry(addr, data);
        }
    }

    pub fn read_wave(&self, addr: u16) -> u8 {
        if self.is_on() {
            self.wave_table.get_current()
        } else {
            self.wave_table.get_entry(addr)
        }
    }

    pub fn set_x0(&mut self, data: u8) {
        self.dac = if data & 0b1000_0000 != 0 {
            DacMode::On
        } else {
            DacMode::Off
        };
    }

    pub fn set_x1(&mut self, data: u8) {
        self.counter.set_counter(256 - (data as u16));
    }

    pub fn set_x2(&mut self, data: u8) {
        self.volume_shift = VOLUME_SHIFT[(data as usize & 0b0110_0000) >> 5];
    }

    pub fn set_x3(&mut self, data: u8) {
        let freq = freq_low(self.wave_table.get_freq(), data);
        self.wave_table.set_freq(freq, wave_timer_period(freq));
    }

    pub fn set_x4(&mut self, data: u8) {
        let freq = freq_high(self.wave_table.get_freq(), data);
        self.wave_table.set_freq(freq, wave_timer_period(freq));

        self.counter.set_mode_on(data & 0b0100_0000 != 0);

        if data & TRIGGER_MASK != 0 {
            self.mode = ChannelMode::On;

            self.wave_table.reset();
            self.counter.reset();

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

    pub fn is_on(&self) -> bool {
        self.mode == ChannelMode::On
    }

    pub fn is_dac_on(&self) -> bool {
        self.dac == DacMode::On
    }
}

fn wave_timer_period(frequency: u32) -> u32 {
    (2048 - frequency) * 2
}
