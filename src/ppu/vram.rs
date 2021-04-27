use super::LcdMode;

const TILESET_SIZE: usize = 0x1800;
const TILEMAP_SIZE: usize = 0x800;
const TILES_COUNT: usize = 0x180;
const OAM_SIZE: usize = 0xa0;
const SPRITE_COUNT: usize = 40;

pub struct VideoRam {
    sprite_table: Box<[u8; OAM_SIZE]>,
    sprites: Box<[Sprite; SPRITE_COUNT]>,

    tile_sets: [Box<[u8; TILESET_SIZE]>; 2],
    tiles: [Box<[Tile; TILES_COUNT]>; 2],

    tile_map: Box<[u8; TILEMAP_SIZE]>,
    attr_map: Box<[BgAttr; TILEMAP_SIZE]>,

    vram_bank: usize,
    cgb: bool,
}

impl VideoRam {
    pub fn new(cgb: bool) -> Self {
        let empty_tile = [[TileValue::B00; 8]; 8];

        let mut sprites = Box::new([Sprite::default(); SPRITE_COUNT]);
        sprites
            .iter_mut()
            .enumerate()
            .for_each(|(i, sp)| sp.index = i);

        VideoRam {
            sprite_table: Box::new([0u8; OAM_SIZE]),
            sprites,

            tile_sets: [Box::new([0u8; TILESET_SIZE]), Box::new([0u8; TILESET_SIZE])],
            tiles: [
                Box::new([empty_tile; TILES_COUNT]),
                Box::new([empty_tile; TILES_COUNT]),
            ],

            tile_map: Box::new([0u8; TILEMAP_SIZE]),
            attr_map: Box::new([Default::default(); TILEMAP_SIZE]),

            vram_bank: 0,
            cgb,
        }
    }

    pub fn bank(&self) -> u8 {
        self.vram_bank as u8
    }

    pub fn switch_bank(&mut self, data: u8) {
        self.vram_bank = (data & 0b01) as usize;
    }

    pub fn sprites(&self) -> &[Sprite; SPRITE_COUNT] {
        &self.sprites
    }

    pub fn read_sprite(&self, addr: usize, mode: LcdMode) -> u8 {
        if mode == LcdMode::VBlank || mode == LcdMode::HBlank {
            self.sprite_table[addr]
        } else {
            0xff
        }
    }

    pub fn write_sprite(&mut self, addr: usize, data: u8, mode: LcdMode) {
        if mode == LcdMode::VBlank || mode == LcdMode::HBlank {
            self.sprite_table[addr] = data;

            let sprite = &mut self.sprites[addr >> 2];
            match addr & 0x03 {
                0x00 => sprite.y = data as i16,
                0x01 => sprite.x = data as i16,
                0x02 => sprite.tile_index = data,
                0x03 => {
                    sprite.above_bg = (data & 0b1000_0000) == 0;
                    sprite.flip_y = (data & 0b0100_0000) != 0;
                    sprite.flip_x = (data & 0b0010_0000) != 0;
                    if self.cgb {
                        sprite.palette = data & 0b0000_0111;
                        sprite.vram_bank = (data & 0b0000_1000) >> 3;
                    } else {
                        sprite.palette = (data & 0b0001_0000) >> 4;
                        sprite.vram_bank = 0;
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn tile(&self, bank: u8, index: usize) -> &Tile {
        &self.tiles[bank as usize][index]
    }

    pub fn read_tile(&self, addr: usize, mode: LcdMode) -> u8 {
        if mode != LcdMode::Transfer {
            self.tile_sets[self.vram_bank][addr]
        } else {
            0xff
        }
    }

    pub fn write_tile(&mut self, addr: usize, data: u8, mode: LcdMode) {
        if mode == LcdMode::Transfer {
            return;
        }

        self.tile_sets[self.vram_bank][addr] = data;

        // 384 tiles * 16 bytes
        // 0001 1111 1111 1110
        // 000T TTTT TTTT YYY0
        let addr = addr & 0x1ffe;

        let tile = &mut self.tiles[self.vram_bank][addr >> 4];
        let y = (addr >> 1) & 0b0111;

        for x in 0..8 {
            let mask = 1 << (7 - x);
            let lsb = mask & self.tile_sets[self.vram_bank][addr];
            let msb = mask & self.tile_sets[self.vram_bank][addr + 1];

            tile[y][x] = match (msb == 0, lsb == 0) {
                (true, true) => TileValue::B00,
                (true, false) => TileValue::B01,
                (false, true) => TileValue::B10,
                (false, false) => TileValue::B11,
            };
        }
    }

    pub fn tilemap(&self, index: usize) -> usize {
        self.tile_map[index] as usize
    }

    pub fn attrmap(&self, index: usize) -> &BgAttr {
        &self.attr_map[index]
    }

    pub fn read_map(&self, addr: usize, mode: LcdMode) -> u8 {
        if mode != LcdMode::Transfer {
            if self.vram_bank == 0 {
                self.tile_map[addr]
            } else {
                self.attr_map[addr].raw
            }
        } else {
            0xff
        }
    }

    pub fn write_map(&mut self, addr: usize, data: u8, mode: LcdMode) {
        if mode != LcdMode::Transfer {
            if self.vram_bank == 0 {
                self.tile_map[addr] = data;
            } else {
                self.attr_map[addr] = BgAttr::from_u8(data);
            }
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct Sprite {
    pub index: usize,
    pub x: i16,
    pub y: i16,
    pub tile_index: u8,
    pub above_bg: bool,
    pub flip_y: bool,
    pub flip_x: bool,
    pub palette: u8,
    pub vram_bank: u8,
}

#[derive(Default, Copy, Clone)]
pub struct BgAttr {
    raw: u8,
    pub bg_pal_index: u8,
    pub vram_bank: u8,
    pub flip_x: bool,
    pub flip_y: bool,
    pub above_all: bool,
}

impl BgAttr {
    fn from_u8(data: u8) -> Self {
        BgAttr {
            raw: data,
            bg_pal_index: data & 0b0111,
            vram_bank: (data & 0b1000) >> 3,
            flip_x: (data & 0b10_0000) != 0x00,
            flip_y: (data & 0b100_0000) != 0x00,
            above_all: (data & 0x80) != 0x00,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum TileValue {
    B00 = 0,
    B01 = 1,
    B10 = 2,
    B11 = 3,
}

pub type Tile = [[TileValue; 8]; 8];
