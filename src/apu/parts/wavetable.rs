use super::Timer;

const INIT_TABLE: [u8; 32] = [
    0x08, 0x04, 0x04, 0x00, 0x04, 0x03, 0x0A, 0x0A, 0x02, 0x0D, 0x07, 0x08, 0x09, 0x02, 0x03, 0x0C,
    0x06, 0x00, 0x05, 0x09, 0x05, 0x09, 0x0B, 0x00, 0x03, 0x04, 0x0B, 0x08, 0x02, 0x0E, 0x0D, 0x0A,
];

pub struct WaveTable {
    timer: Timer,
    freq: u32,

    index: usize,
    wave_table: [u8; 32],
    sample_buffer: u8,
}

impl WaveTable {
    pub fn new() -> Self {
        WaveTable {
            timer: Timer::new(0),
            freq: 0,

            index: 0,
            wave_table: INIT_TABLE,
            sample_buffer: 0,
        }
    }

    pub fn next(&mut self) -> u8 {
        if self.timer.tick() {
            self.index = (self.index + 1) % 32;
            self.sample_buffer = self.wave_table[self.index];
        }
        self.sample_buffer
    }

    pub fn set_entry(&mut self, index: u16, data: u8) {
        let index = index as usize * 2;
        self.wave_table[index] = (data & 0xf0) >> 4;
        self.wave_table[index + 1] = data & 0x0f;
    }

    pub fn get_entry(&self, index: u16) -> u8 {
        let index = index as usize * 2;
        (self.wave_table[index] << 4) | (self.wave_table[index + 1])
    }

    pub fn get_current(&self) -> u8 {
        let index = self.index & !0b01;
        (self.wave_table[index] << 4) | self.wave_table[index + 1]
    }

    pub fn set_freq(&mut self, freq: u32, period: u32) {
        self.freq = freq;
        self.timer.set_period(period);
    }

    pub fn get_freq(&self) -> u32 {
        self.freq
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.sample_buffer = 0;
        self.timer.reset();
    }
}
