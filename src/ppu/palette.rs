use super::TileValue;

type Color = [u8; 3];

// black-white
// const COLOR_PALETTE: [u32; 4] = [0x00ff_ffff, 0x00c0_c0c0, 0x0060_6060, 0x0000_0000];
// classic
// const COLOR_PALETTE: [u32; 4] = [0x00ef_ffde, 0x00ad_d794, 0x0052_9273, 0x0018_3442];
// bgb
// const COLOR_PALETTE: [u32; 4] = [0x00e0_f8d0, 0x0088_c070, 0x0034_6856, 0x0008_1820];
// kirokaze gameboy
// const COLOR_PALETTE: [u32; 4] = [0x00e2_f3e4, 0x0094e_344, 0x0046_878f, 0x0033_2c50];
// mist gb
const COLOR_PALETTE: [[u8; 3]; 4] = [
    [0xc4, 0xf0, 0xc2],
    [0x5a, 0xb9, 0xa8],
    [0x1e, 0x60, 0x6e],
    [0x2d, 0x1b, 0x00],
];

pub struct Palette {
    palette_index: [PaletteIndex; 2],
    palettes_rgb: [[Color; 4]; 8],
    palattes_555: [[u16; 4]; 8],
    data_index: usize,
    index_inc: bool,
}

impl Palette {
    pub fn build(cgb: bool) -> Self {
        if !cgb {
            Palette {
                palette_index: [Default::default(); 2],
                palettes_rgb: [COLOR_PALETTE; 8],
                palattes_555: [[0u16; 4]; 8],
                data_index: 0,
                index_inc: false,
            }
        } else {
            Palette {
                palette_index: [Default::default(); 2],
                palettes_rgb: Default::default(),
                palattes_555: [[0u16; 4]; 8],
                data_index: 0,
                index_inc: false,
            }
        }
    }

    pub fn read_index(&self) -> u8 {
        self.data_index as u8 | ((self.index_inc as u8) << 7)
    }

    pub fn read_data(&self) -> u8 {
        // palatte color byte
        // PP_PCCB
        let pal_index = (self.data_index & 0b11_1000) >> 3;
        let color_index = (self.data_index & 0b00_0110) >> 1;
        let first_byte = (self.data_index & 0b01) == 0b00;

        if first_byte {
            self.palattes_555[pal_index][color_index] as u8
        } else {
            (self.palattes_555[pal_index][color_index] >> 8) as u8
        }
    }

    pub fn write_index(&mut self, index: u8) {
        self.index_inc = (index & 0x80) != 0x00;
        self.data_index = (index & 0b11_1111) as usize;
    }

    pub fn write_data(&mut self, data: u8) {
        let pal_index = (self.data_index & 0b11_1000) >> 3;
        let color_index = (self.data_index & 0b00_0110) >> 1;
        let first_byte = (self.data_index & 0b01) == 0b00;

        let mut color = self.palattes_555[pal_index][color_index];
        if first_byte {
            color = (color & 0xff00) | (data as u16);
        } else {
            color = (color & 0x00ff) | (((data & 0b0111_1111) as u16) << 8);
        }
        self.palattes_555[pal_index][color_index] = color;

        // f e d c b a 9 8 7 6 5 4 3 2 1 0
        //                       --------- red
        //             --------- green
        //   --------- blue
        let r = color & 0b0001_1111;
        let g = (color >> 5) & 0b0001_1111;
        let b = (color >> 10) & 0b0001_1111;

        // ref: https://byuu.net/video/color-emulation
        let r_adjusted = ((r * 26 + g * 4 + b * 2).min(960) / 4) as u8;
        let g_adjusted = ((g * 24 + b * 8).min(960) / 4) as u8;
        let b_adjusted = ((r * 6 + g * 4 + b * 22).min(960) / 4) as u8;

        self.palettes_rgb[pal_index][color_index][0] = r_adjusted;
        self.palettes_rgb[pal_index][color_index][1] = g_adjusted;
        self.palettes_rgb[pal_index][color_index][2] = b_adjusted;

        if self.index_inc {
            self.data_index = (self.data_index + 1) % 0x40;
        }
    }

    pub fn read_dmg(&self, pal: u8) -> u8 {
        self.palette_index[pal as usize].raw
    }

    pub fn write_dmg(&mut self, pal: u8, data: u8) {
        let pal = pal as usize;
        self.palette_index[pal] = PaletteIndex::from_u8(data);

        for i in 0..4 {
            self.palettes_rgb[pal][i] = COLOR_PALETTE[self.palette_index[pal].pal[i]];
        }
    }

    pub fn color(&self, pal: u8, color: TileValue) -> &Color {
        &self.palettes_rgb[pal as usize][color as usize]
    }
}

#[derive(Copy, Clone)]
pub struct PaletteIndex {
    raw: u8,
    pal: [usize; 4],
}

impl Default for PaletteIndex {
    fn default() -> PaletteIndex {
        PaletteIndex {
            raw: 0xe4,
            pal: [0, 1, 2, 3],
        }
    }
}

impl PaletteIndex {
    pub fn from_u8(data: u8) -> PaletteIndex {
        let data = data as usize;
        PaletteIndex {
            raw: data as u8,
            pal: [
                (data & 0b0000_0011) >> 0,
                (data & 0b0000_1100) >> 2,
                (data & 0b0011_0000) >> 4,
                (data & 0b1100_0000) >> 6,
            ],
        }
    }
}
