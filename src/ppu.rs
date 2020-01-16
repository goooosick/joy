use crate::interrupt::{Interrupt, InterruptHandler};
use crate::{GB_LCD_HEIGHT, GB_LCD_WIDTH};
use bitflags::bitflags;
use std::ops::Index;

const TILESET_SIZE: usize = 0x1800;
const TILEMAP_SIZE: usize = 0x800;
const TILES_COUNT: usize = 0x180;
const OAM_SIZE: usize = 0xa0;
const SPRITE_COUNT: usize = 40;
const MAX_SPRITE_PER_LINE: usize = 10;

// black-white
// const COLOR_PALETTE: [u32; 4] = [0x00ff_ffff, 0x00c0_c0c0, 0x0060_6060, 0x0000_0000];
// classic
// const COLOR_PALETTE: [u32; 4] = [0x00ef_ffde, 0x00ad_d794, 0x0052_9273, 0x0018_3442];
// bgb
// const COLOR_PALETTE: [u32; 4] = [0x00e0_f8d0, 0x0088_c070, 0x0034_6856, 0x0008_1820];
// kirokaze gameboy
// const COLOR_PALETTE: [u32; 4] = [0x00e2_f3e4, 0x0094e_344, 0x0046_878f, 0x0033_2c50];
// mist gb
const COLOR_PALETTE: [u32; 4] = [0x00c4_f0c2, 0x005a_b9a8, 0x001e_606e, 0x002d_1b00];

pub struct Ppu {
    frame_buffer: Box<[u32; GB_LCD_WIDTH * GB_LCD_HEIGHT]>,

    tile_sets: Box<[u8; TILESET_SIZE]>,
    tiles: Box<[Tile; TILES_COUNT]>,

    tile_maps: Box<[u8; TILEMAP_SIZE]>,

    sprite_table: Box<[u8; OAM_SIZE]>,
    sprites: Box<[Sprite; SPRITE_COUNT]>,

    lcdc: LCDC,
    stat: STAT,
    mode: Mode,

    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    winy: u8,
    winx: u8,

    bg_palette: Palette,
    obj_palettes: [Palette; 2],

    clocks: u32,
}

impl Ppu {
    pub fn new() -> Ppu {
        let empty_tile = [[TileValue::B00; 8]; 8];
        Ppu {
            frame_buffer: Box::new([0x00ff_ffff; GB_LCD_WIDTH * GB_LCD_HEIGHT]),

            tile_sets: Box::new([0u8; TILESET_SIZE]),
            tiles: Box::new([empty_tile; TILES_COUNT]),

            tile_maps: Box::new([0u8; TILEMAP_SIZE]),

            sprite_table: Box::new([0u8; OAM_SIZE]),
            sprites: Box::new([Sprite::default(); SPRITE_COUNT]),

            lcdc: Default::default(),
            stat: Default::default(),
            mode: Mode::Transfer,

            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            winy: 0,
            winx: 0,

            bg_palette: Default::default(),
            obj_palettes: [Default::default(); 2],

            clocks: 0,
        }
    }

    pub fn get_frame_buffer(&self) -> &[u32] {
        self.frame_buffer.as_ref()
    }

    fn render_line(&mut self) {
        let pattern_offset = (!self.lcdc.contains(LCDC::TILE_PATTERN_TABLE)) as usize;
        let fb_offset = self.ly as usize * GB_LCD_WIDTH;
        let mut bg_row = [TileValue::B00; GB_LCD_WIDTH];

        if self.lcdc.contains(LCDC::BG_DISPLAY_ON) {
            // PRE:
            //     tilemap: 256 x 256 pixels, 32 x 32 tiles, 32 x 32 bytes
            //     lcd screen: 160 x 144 pixels
            //     tile: 8 x 8 pixels

            // base index of used tilemap
            let tilemap_offset = 0x400 * (self.lcdc.contains(LCDC::BG_TILE_TABLE) as usize);

            // wrapping around y
            let tilemap_y = self.ly.wrapping_add(self.scy) as usize;
            // line start of tilemap, row(tilemap_y / tile_height) * tile_per_row(32)
            let tilemap_offset = tilemap_offset + (tilemap_y / 8) * 32;
            // y in that tile (tilemap_y % 8)
            let tile_y = tilemap_y & 0x07;

            // whole line
            for x in 0..GB_LCD_WIDTH {
                // wrapping around x
                let tilemap_x = (x as u8).wrapping_add(self.scx) as usize;
                // x in that tile (tilemap_x % 8)
                let tile_x = tilemap_x & 0x07;

                let tilemap_index = tilemap_offset + tilemap_x / 8;
                let tile_index = {
                    let index = self.tile_maps[tilemap_index] as usize;
                    // when using pattern 0, the index is signed
                    // but -128 ~ -0 is the same in bits as 128 ~ 255
                    // so only the 0 ~ 127 part (< 0x80) need offset
                    index + (pattern_offset * (index < 0x80) as usize) * 0x100
                };

                let palette_index = self.tiles[tile_index][tile_y][tile_x];
                let color_index = self.bg_palette.pal[palette_index as usize];

                self.frame_buffer[fb_offset + x] = COLOR_PALETTE[color_index];
                bg_row[x] = palette_index;
            }
        }

        if self.lcdc.contains(LCDC::WINDOW_DISPLAY_ON) && self.ly >= self.winy {
            let tilemap_offset = 0x400 * (self.lcdc.contains(LCDC::WINDOW_TILE_TABLE) as usize);

            // the window always draw from left-upper corner
            let tilemap_y = (self.ly - self.winy) as usize;
            let tilemap_offset = tilemap_offset + (tilemap_y / 8) * 32;
            let tile_y = tilemap_y & 0x07;

            let win_x = self.winx.saturating_sub(7) as usize;
            // for winx < 7 (I'm not sure about this, but it fixes some game)
            let winx_offset = (7 - (self.winx as isize)).max(0) as usize;

            // draw from win_x - the left edge of window, but tilemap starts from 0
            for (tilemap_x, x) in (win_x..GB_LCD_WIDTH).enumerate() {
                let tilemap_x = tilemap_x + winx_offset;

                let tilemap_index = tilemap_offset + tilemap_x / 8;
                let tile_index = {
                    let index = self.tile_maps[tilemap_index] as usize;
                    index + (pattern_offset * (index < 0x80) as usize) * 0x100
                };

                let tile_x = tilemap_x & 0x07;
                let palette_index = self.tiles[tile_index][tile_y][tile_x];
                let color_index = self.bg_palette.pal[palette_index as usize];

                self.frame_buffer[fb_offset + x] = COLOR_PALETTE[color_index];
                bg_row[x] = palette_index;
            }
        }

        if self.lcdc.contains(LCDC::OBJECT_DISPLAY_ON) {
            // sprite size: 8 x 8 or 8 x 16
            let sprite_size = 8 * (1 + self.lcdc.contains(LCDC::OBJECT_SIZE) as i16);
            let ly = self.ly as i16;

            let mut sprites = self
                .sprites
                .iter()
                .filter(|sp| {
                    sp.y <= ly
                        && (sp.y + sprite_size) > ly
                        && (sp.x + 8 >= 0)
                        && sp.x < GB_LCD_WIDTH as i16
                })
                .take(MAX_SPRITE_PER_LINE)
                .collect::<Vec<_>>();
            // why does this sort needed (to fix background-sprite interaction)?
            sprites.sort_by(|sp0, sp1| sp0.x.cmp(&sp1.x));

            // draw reverse, so samller x is on top
            for sprite in sprites.iter().rev() {
                let sprite_y = (ly - sprite.y) as usize;

                // sprite size indepedent
                let mut tile_y = sprite_y & 0x07;
                if sprite.flip_y {
                    tile_y = 7 - tile_y;
                }

                // http://problemkaputt.de/pandocs.htm#vramspriteattributetableoam
                //              flip_y                   upper "NN AND FEh"
                //              0    1                   lower "NN OR 01h"
                //   y < 8  0   lo   hi
                //          1   hi   lo
                // ---------------------------------------------------------------
                let tile_index = if sprite_size == 16 {
                    if sprite.flip_y ^ (sprite_y < 8) {
                        sprite.tile_index & 0xfe
                    } else {
                        sprite.tile_index | 0x01
                    }
                } else {
                    sprite.tile_index
                };
                let tile = self.tiles[tile_index as usize];

                // only draw sprite pixels
                for x in 0..8 {
                    let screen_x = sprite.x + x;

                    let tile_x = if sprite.flip_x { 7 - x } else { x };
                    let color = tile[tile_y as usize][tile_x as usize];

                    let on_screen = screen_x >= 0 && screen_x < GB_LCD_WIDTH as i16;
                    if on_screen && color != TileValue::B00 {
                        if sprite.above_bg || bg_row[screen_x as usize] == TileValue::B00 {
                            let color_index = self.obj_palettes[sprite.palette as usize][color];
                            self.frame_buffer[fb_offset + screen_x as usize] =
                                COLOR_PALETTE[color_index];
                        }
                    }
                }
            }
        }
    }

    pub fn update(&mut self, clocks: u32, interrupts: &mut InterruptHandler) {
        if !self.lcdc.contains(LCDC::LCD_ON) {
            return;
        }
        self.clocks += clocks;

        let mut stat_interrupt = false;

        // from:
        // http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-GPU-Timings
        //     OAM       Transfer     HBlank      VBlank
        //      80         172          204         456
        //     -----------------------------     ---------
        // ly            0 - 143                 144 - 153
        match self.mode {
            Mode::OamSearch => {
                if self.clocks >= 80 {
                    self.clocks -= 80;
                    self.mode = Mode::Transfer;
                }
            }
            Mode::Transfer => {
                if self.clocks >= 172 {
                    self.clocks -= 172;
                    self.mode = Mode::HBlank;

                    self.render_line();
                    stat_interrupt = self.stat.contains(STAT::HBLANK_INTERRUPT);
                }
            }
            Mode::HBlank => {
                if self.clocks >= 204 {
                    self.clocks -= 204;
                    self.ly += 1;

                    if self.ly == 143 {
                        // last line lost ???
                        self.render_line();
                        self.mode = Mode::VBlank;

                        interrupts.request_interrupt(Interrupt::VBlank);
                        stat_interrupt = self.stat.contains(STAT::VBLANK_INTERRUPT);
                    } else {
                        self.mode = Mode::OamSearch;
                        stat_interrupt = self.stat.contains(STAT::OAM_INTERRUPT);
                    }
                }
            }
            Mode::VBlank => {
                if self.clocks >= 456 {
                    self.clocks -= 456;
                    self.ly += 1;

                    if self.ly >= 153 {
                        self.ly = 0;
                        self.mode = Mode::OamSearch;
                        stat_interrupt = self.stat.contains(STAT::OAM_INTERRUPT);
                    }
                }
            }
        };

        let coincidence = self.lyc == self.ly;
        self.stat.set(STAT::COINCIDENCE, coincidence);

        if stat_interrupt || (self.stat.contains(STAT::SCANLINE_INTERRUPT) && coincidence) {
            interrupts.request_interrupt(Interrupt::Lcd);
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x8000..=0x97ff => {
                if self.mode != Mode::Transfer {
                    self.tile_sets[addr - 0x8000]
                } else {
                    0xff
                }
            }
            0x9800..=0x9fff => {
                if self.mode != Mode::Transfer {
                    self.tile_maps[addr - 0x9800]
                } else {
                    0xff
                }
            }
            0xfe00..=0xfe9f => {
                if self.mode == Mode::VBlank || self.mode == Mode::HBlank {
                    self.sprite_table[addr - 0xfe00]
                } else {
                    0xff
                }
            }

            0xff40 => self.lcdc.bits(),
            0xff41 => self.stat.bits() | (self.mode as u8),
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff45 => self.lyc,
            0xff47 => self.bg_palette.to_u8(),
            0xff48 => self.obj_palettes[0].to_u8(),
            0xff49 => self.obj_palettes[1].to_u8(),

            0xff4a => self.winy,
            0xff4b => self.winx,

            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, b: u8) {
        let addr = addr as usize;
        match addr {
            0x8000..=0x97ff => {
                // causing donkey kong land 2 sprites corruption (´。＿。｀)
                // if self.mode != Mode::Transfer {
                self.update_tiles(addr - 0x8000, b);
                // }
            }
            0x9800..=0x9fff => {
                if self.mode != Mode::Transfer {
                    self.tile_maps[addr - 0x9800] = b;
                }
            }
            0xfe00..=0xfe9f => {
                if self.mode == Mode::VBlank || self.mode == Mode::HBlank {
                    self.update_sprites(addr - 0xfe00, b);
                }
            }

            0xff40 => {
                let new = LCDC::from_bits(b).unwrap();
                if !new.contains(LCDC::LCD_ON) && self.lcdc.contains(LCDC::LCD_ON) {
                    assert!(self.mode == Mode::VBlank);

                    self.ly = 0;
                    self.clocks = 0;
                    self.mode = Mode::HBlank;
                }
                if new.contains(LCDC::LCD_ON) && !self.lcdc.contains(LCDC::LCD_ON) {
                    self.mode = Mode::HBlank;
                }
                self.lcdc = new;
            }
            0xff41 => self.stat = self.stat & STAT::COINCIDENCE | STAT::from_bits_truncate(b),
            0xff42 => self.scy = b,
            0xff43 => self.scx = b,
            0xff44 => {}
            0xff45 => self.lyc = b,
            0xff47 => self.bg_palette = Palette::from_u8(b),
            0xff48 => self.obj_palettes[0] = Palette::from_u8(b),
            0xff49 => self.obj_palettes[1] = Palette::from_u8(b),

            0xff4a => self.winy = b,
            0xff4b => self.winx = b,

            _ => unreachable!(),
        }
    }

    fn update_tiles(&mut self, addr: usize, data: u8) {
        self.tile_sets[addr] = data;

        // 384 tiles * 16 bytes
        // 0001 1111 1111 1110
        // 000T TTTT TTTT YYY0
        let addr = addr & 0x1ffe;

        let tile = &mut self.tiles[addr >> 4];
        let y = (addr >> 1) & 0b0111;

        for x in 0..8 {
            let mask = 1 << (7 - x);
            let lsb = mask & self.tile_sets[addr];
            let msb = mask & self.tile_sets[addr + 1];

            tile[y][x] = match (msb == 0, lsb == 0) {
                (true, true) => TileValue::B00,
                (true, false) => TileValue::B01,
                (false, true) => TileValue::B10,
                (false, false) => TileValue::B11,
            };
        }
    }

    pub fn dma_write(&mut self, addr: u16, data: u8) {
        self.update_sprites(addr as usize, data);
    }

    fn update_sprites(&mut self, addr: usize, data: u8) {
        self.sprite_table[addr] = data;

        let sprite = &mut self.sprites[addr >> 2];
        match addr & 0x03 {
            0x00 => sprite.y = data as i16 - 16,
            0x01 => sprite.x = data as i16 - 8,
            0x02 => sprite.tile_index = data,
            0x03 => {
                sprite.above_bg = (data & 0b1000_0000) == 0;
                sprite.flip_y = (data & 0b0100_0000) != 0;
                sprite.flip_x = (data & 0b0010_0000) != 0;
                sprite.palette = (data & 0b0001_0000) >> 4;
                // sprite.vram_bank_cgb = (data & 0b0000_1000) >> 3;
                // sprite.palette_cgb = data & 0b0000_0111;
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Default, Copy, Clone)]
struct Sprite {
    x: i16,
    y: i16,
    tile_index: u8,
    above_bg: bool,
    flip_y: bool,
    flip_x: bool,
    palette: u8,
    // vram_bank_cgb: u8,
    // palette_cgb: u8,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum TileValue {
    B00 = 0,
    B01 = 1,
    B10 = 2,
    B11 = 3,
}

type Tile = [[TileValue; 8]; 8];

bitflags! {
    #[derive(Default)]
    pub struct LCDC: u8 {
        /// lcd display enable
        const LCD_ON                = 0b1000_0000;
        /// select window tile table address, 0=9800-9bff, 1=9c00-9fff
        const WINDOW_TILE_TABLE     = 0b0100_0000;
        /// window display enable
        const WINDOW_DISPLAY_ON     = 0b0010_0000;
        /// select tile pattern table address, 0=8800-97ff, 1=8000-8fff
        const TILE_PATTERN_TABLE    = 0b0001_0000;
        /// select background tile table address, 0=9800-9bff, 1=9c00-9fff
        const BG_TILE_TABLE      = 0b0000_1000;
        /// select object size, 0=8x8, 1=8x16
        const OBJECT_SIZE           = 0b0000_0100;
        /// object display enable
        const OBJECT_DISPLAY_ON     = 0b0000_0010;
        /// background display enable
        const BG_DISPLAY_ON         = 0b0000_0001;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct STAT: u8 {
        /// scanline interrupt on/off
        const SCANLINE_INTERRUPT  = 0b0100_0000;
        /// oam interrupt on/off
        const OAM_INTERRUPT       = 0b0010_0000;
        /// vblank interrupt on/off
        const VBLANK_INTERRUPT    = 0b0001_0000;
        /// hblank interrupt on/off
        const HBLANK_INTERRUPT    = 0b0000_1000;
        /// COINCIDENCE flag (0: lyc != ly, 1: lyc == ly), read-only
        const COINCIDENCE         = 0b0000_0100;
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamSearch = 2,
    Transfer = 3,
}

#[derive(Copy, Clone)]
struct Palette {
    pal: [usize; 4],
}

impl Default for Palette {
    fn default() -> Palette {
        Palette { pal: [0, 1, 2, 3] }
    }
}

impl Palette {
    fn from_u8(byte: u8) -> Palette {
        let byte = byte as usize;
        Palette {
            pal: [
                (byte & 0b0000_0011) >> 0,
                (byte & 0b0000_1100) >> 2,
                (byte & 0b0011_0000) >> 4,
                (byte & 0b1100_0000) >> 6,
            ],
        }
    }

    fn to_u8(&self) -> u8 {
        ((self.pal[3] << 6) | (self.pal[2] << 4) | (self.pal[1] << 2) | (self.pal[0] << 0)) as u8
    }
}

impl Index<TileValue> for Palette {
    type Output = usize;

    fn index(&self, tile: TileValue) -> &Self::Output {
        &self.pal[tile as usize]
    }
}
