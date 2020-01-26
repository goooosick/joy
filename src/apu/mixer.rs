const MASK_VEC: [u8; 2] = [0x00, 0xff];

pub struct Mixer {
    so1_masks: [u8; 4],
    so2_masks: [u8; 4],

    so1_volume: u8,
    so2_volume: u8,
}

impl Mixer {
    pub fn new() -> Self {
        Mixer {
            so1_masks: [0u8; 4],
            so2_masks: [0u8; 4],
            so1_volume: 0,
            so2_volume: 0,
        }
    }

    pub fn mix(&self, chs: [u8; 4]) -> (u8, u8) {
        let mut so1 = 0;
        let mut so2 = 0;

        for i in 0..4 {
            so1 += self.so1_masks[i] & chs[i];
            so2 += self.so2_masks[i] & chs[i];
        }

        // (0..15) * 4 + 128 maps to (128..248)
        so1 = ((so1 as u16 * self.so1_volume as u16) / 4) as u8 + 128;
        so2 = ((so2 as u16 * self.so2_volume as u16) / 4) as u8 + 128;

        (so2, so1)
    }

    pub fn set_volume(&mut self, data: u8) {
        self.so1_volume = (data & 0b0111) + 1;
        self.so2_volume = ((data >> 4) & 0b0111) + 1;
    }

    pub fn set_output(&mut self, data: u8) {
        let mut data = data as usize;
        for i in 0..4 {
            self.so1_masks[i] = MASK_VEC[data & 0b01];
            data >>= 1;
        }
        for i in 0..4 {
            self.so2_masks[i] = MASK_VEC[data & 0b01];
            data >>= 1;
        }
    }
}
