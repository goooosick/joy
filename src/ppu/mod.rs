use crate::interrupt::{Interrupt, InterruptHandler};
use crate::{GB_LCD_HEIGHT, GB_LCD_WIDTH};
use bitflags::bitflags;

use palette::*;
use vram::*;

mod palette;
mod vram;

const MAX_SPRITE_PER_LINE: usize = 10;
const FRAME_BUFFER_SIZE: usize = GB_LCD_WIDTH * GB_LCD_HEIGHT * 3;

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum LcdMode {
    HBlank = 0,
    VBlank = 1,
    OamSearch = 2,
    Transfer = 3,
}

pub struct Ppu {
    frame_buffer: Box<[u8; FRAME_BUFFER_SIZE]>,
    back_buffer: Box<[u8; FRAME_BUFFER_SIZE]>,

    vram: VideoRam,

    lcdc: LCDC,
    stat: STAT,
    mode: LcdMode,

    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    winy: u8,
    winx: u8,

    hdma_avaliable: bool,
    bg_palette: Palette,
    obj_palette: Palette,
    cgb: bool,

    current_x: usize,
    bg_above: [bool; GB_LCD_WIDTH],
    bg_b00: [bool; GB_LCD_WIDTH],

    clocks: u32,
}

impl Ppu {
    pub fn new(cgb: bool) -> Ppu {
        Ppu {
            frame_buffer: Box::new([0u8; FRAME_BUFFER_SIZE]),
            back_buffer: Box::new([0u8; FRAME_BUFFER_SIZE]),

            vram: VideoRam::new(cgb),

            lcdc: Default::default(),
            stat: Default::default(),
            mode: LcdMode::Transfer,

            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            winy: 0,
            winx: 0,

            hdma_avaliable: false,
            bg_palette: Palette::build(cgb),
            obj_palette: Palette::build(cgb),
            cgb,

            current_x: 0,
            bg_above: [false; GB_LCD_WIDTH],
            bg_b00: [false; GB_LCD_WIDTH],

            clocks: 0,
        }
    }

    // since the default attrs map is valid for dmg, the
    // rendering of dmg and cgb is unified.
    fn render_line(&mut self, count: usize) {
        let pattern_offset = (!self.lcdc.contains(LCDC::BG_TILE_TABLE)) as usize;
        let fb_offset = self.ly as usize * GB_LCD_WIDTH * 3;

        if count > 0 {
            if self.cgb || self.lcdc.contains(LCDC::BG_ON) {
                // PRE:
                //     tilemap: 256 x 256 pixels, 32 x 32 tiles, 32 x 32 bytes
                //     lcd screen: 160 x 144 pixels
                //     tile: 8 x 8 pixels

                // base index of used tilemap
                let tilemap_offset = 0x400 * (self.lcdc.contains(LCDC::BG_MAP) as usize);

                // wrapping around y
                let tilemap_y = self.ly.wrapping_add(self.scy) as usize;
                // line start of tilemap, row(tilemap_y / tile_height) * tile_per_row(32)
                let tilemap_offset = tilemap_offset + (tilemap_y / 8) * 32;
                // y in that tile (tilemap_y % 8)
                let tile_y = tilemap_y & 0x07;

                let start = self.current_x;
                let end = (start + count).min(GB_LCD_WIDTH);

                // whole line
                for x in start..end {
                    // wrapping around x
                    let tilemap_x = (x as u8).wrapping_add(self.scx) as usize;
                    let tilemap_index = tilemap_offset + tilemap_x / 8;
                    let attr = self.vram.attrmap(tilemap_index);

                    let tile_index = {
                        let index = self.vram.tilemap(tilemap_index);
                        // when using pattern 0, the index is signed
                        // but -128 ~ -0 is the same in bits as 128 ~ 255
                        // so only the 0 ~ 127 part (< 0x80) need offset
                        index + (pattern_offset * (index < 0x80) as usize) * 0x100
                    };

                    // x in that tile (tilemap_x % 8)
                    let tile_x = (tilemap_x & 7) ^ ((attr.flip_x as usize) * 7);
                    let tile_y = tile_y ^ ((attr.flip_y as usize) * 7);

                    let color_index = self.vram.tile(attr.vram_bank, tile_index)[tile_y][tile_x];
                    let color = self.bg_palette.color(attr.bg_pal_index, color_index);

                    self.frame_buffer[(fb_offset + x * 3)..][0..3].copy_from_slice(color);
                    self.bg_above[x] = attr.above_all;
                    self.bg_b00[x] = color_index == TileValue::B00;
                }
            }

            self.current_x += count;
        } else {
            if self.lcdc.contains(LCDC::WINDOW_ON) && self.ly >= self.winy {
                let tilemap_offset = 0x400 * (self.lcdc.contains(LCDC::WINDOW_MAP) as usize);

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
                    let attr = self.vram.attrmap(tilemap_index);

                    let tile_index = {
                        let index = self.vram.tilemap(tilemap_index);
                        index + (pattern_offset * (index < 0x80) as usize) * 0x100
                    };

                    let tile_x = (tilemap_x & 7) ^ ((attr.flip_x as usize) * 7);
                    let tile_y = tile_y ^ ((attr.flip_y as usize) * 7);

                    let color_index = self.vram.tile(attr.vram_bank, tile_index)[tile_y][tile_x];
                    let color = self.bg_palette.color(attr.bg_pal_index, color_index);

                    self.frame_buffer[(fb_offset + x * 3)..][0..3].copy_from_slice(color);
                    self.bg_above[x] = attr.above_all;
                    self.bg_b00[x] = color_index == TileValue::B00;
                }
            }

            let sprite_above = self.cgb && !self.lcdc.contains(LCDC::BG_ON);
            if self.lcdc.contains(LCDC::OBJECT_ON) {
                // sprite size: 8 x 8 or 8 x 16
                let sprite_size = 8 * (1 + self.lcdc.contains(LCDC::OBJECT_SIZE) as i16);
                let ly = self.ly as i16;

                let mut sprites = self
                    .vram
                    .sprites()
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
                if !self.cgb {
                    sprites.sort_by(|sp0, sp1| sp0.x.cmp(&sp1.x));
                }

                // draw reverse, so samller x is on top
                for sprite in sprites.iter().rev() {
                    let sprite_y = (ly - sprite.y) as usize;
                    // sprite size indepedent
                    let tile_y = (sprite_y & 7) ^ (sprite.flip_y as usize * 7);

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
                    let tile = self.vram.tile(sprite.vram_bank, tile_index as usize);

                    // only draw sprite pixels
                    for x in 0..8 {
                        let screen_x = sprite.x + x;

                        let tile_x = x as usize ^ (sprite.flip_x as usize * 7);
                        let color_index = tile[tile_y][tile_x];

                        let on_screen = screen_x >= 0 && screen_x < GB_LCD_WIDTH as i16;
                        if on_screen && color_index != TileValue::B00 {
                            if sprite_above
                                || (!self.bg_above[screen_x as usize] && sprite.above_bg)
                                || self.bg_b00[screen_x as usize]
                            {
                                let color = self.obj_palette.color(sprite.palette, color_index);
                                self.frame_buffer[(fb_offset + screen_x as usize * 3)..][0..3]
                                    .copy_from_slice(color);
                            }
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
            LcdMode::OamSearch => {
                if self.clocks >= 80 {
                    self.clocks -= 80;
                    self.mode = LcdMode::Transfer;

                    // prepare drawing states
                    self.current_x = 0;
                    self.bg_above.iter_mut().for_each(|x| *x = false);
                    self.bg_b00.iter_mut().for_each(|x| *x = false);
                }
            }
            LcdMode::Transfer => {
                if self.current_x < GB_LCD_WIDTH {
                    self.render_line(clocks as usize);
                }

                if self.clocks >= 172 {
                    if self.current_x < GB_LCD_WIDTH {
                        self.render_line(GB_LCD_WIDTH - self.current_x);
                    }
                    self.render_line(0);

                    self.clocks -= 172;
                    self.mode = LcdMode::HBlank;

                    self.hdma_avaliable = true;
                    stat_interrupt = self.stat.contains(STAT::HBLANK_INTERRUPT);
                }
            }
            LcdMode::HBlank => {
                if self.clocks >= 204 {
                    self.clocks -= 204;

                    self.ly += 1;
                    self.check_lyc(interrupts);

                    if self.ly == 144 {
                        self.mode = LcdMode::VBlank;

                        interrupts.request_interrupt(Interrupt::VBlank);
                        stat_interrupt = self.stat.contains(STAT::VBLANK_INTERRUPT);

                        std::mem::swap(&mut self.frame_buffer, &mut self.back_buffer);
                    } else {
                        self.mode = LcdMode::OamSearch;
                        stat_interrupt = self.stat.contains(STAT::OAM_INTERRUPT);
                    }

                    self.hdma_avaliable = false;
                }
            }
            LcdMode::VBlank => {
                if self.clocks >= 456 {
                    self.clocks -= 456;

                    self.ly += 1;

                    if self.ly == 154 {
                        self.ly = 0;

                        self.mode = LcdMode::OamSearch;
                        stat_interrupt = self.stat.contains(STAT::OAM_INTERRUPT);
                    }

                    self.check_lyc(interrupts);
                }
            }
        };

        if stat_interrupt {
            interrupts.request_interrupt(Interrupt::Lcd);
        }
    }

    fn check_lyc(&mut self, interrupts: &mut InterruptHandler) {
        let coincidence = self.lyc == self.ly;
        self.stat.set(STAT::COINCIDENCE, coincidence);

        if self.stat.contains(STAT::SCANLINE_INTERRUPT) && coincidence {
            interrupts.request_interrupt(Interrupt::Lcd);
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x8000..=0x97ff => self.vram.read_tile(addr - 0x8000, self.mode),
            0x9800..=0x9fff => self.vram.read_map(addr - 0x9800, self.mode),
            0xfe00..=0xfe9f => self.vram.read_sprite(addr - 0xfe00, self.mode),

            0xff40 => self.lcdc.bits(),
            0xff41 => self.stat.bits() | (self.mode as u8),
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff45 => self.lyc,
            0xff47 => self.bg_palette.read_dmg(0),
            0xff48 => self.obj_palette.read_dmg(0),
            0xff49 => self.obj_palette.read_dmg(1),

            0xff4a => self.winy,
            0xff4b => self.winx,

            0xff4f if self.cgb => self.vram.bank() | 0xfe,
            0xff68 if self.cgb => self.bg_palette.read_index(),
            0xff69 if self.cgb => self.bg_palette.read_data(),
            0xff6a if self.cgb => self.obj_palette.read_index(),
            0xff6b if self.cgb => self.obj_palette.read_data(),

            _ => 0xff,
        }
    }

    pub fn write(&mut self, addr: u16, b: u8) {
        let addr = addr as usize;
        match addr {
            0x8000..=0x97ff => self.vram.write_tile(addr - 0x8000, b, self.mode),
            0x9800..=0x9fff => self.vram.write_map(addr - 0x9800, b, self.mode),
            0xfe00..=0xfe9f => self.vram.write_sprite(addr - 0xfe00, b, self.mode),

            0xff40 => {
                let new = LCDC::from_bits_truncate(b);
                if !new.contains(LCDC::LCD_ON) && self.lcdc.contains(LCDC::LCD_ON) {
                    self.ly = 0;
                    self.clocks = 0;
                    self.mode = LcdMode::HBlank;
                }
                self.lcdc = new;
            }
            0xff41 => {
                self.stat =
                    self.stat & STAT::COINCIDENCE | STAT::from_bits_truncate(b & 0b0111_1000)
            }
            0xff42 => self.scy = b,
            0xff43 => self.scx = b,
            0xff44 => {}
            0xff45 => self.lyc = b,
            0xff47 if !self.cgb => self.bg_palette.write_dmg(0, b),
            0xff48 if !self.cgb => self.obj_palette.write_dmg(0, b),
            0xff49 if !self.cgb => self.obj_palette.write_dmg(1, b),

            0xff4a => self.winy = b,
            0xff4b => self.winx = b,

            0xff4f if self.cgb => self.vram.switch_bank(b),
            0xff68 if self.cgb => self.bg_palette.write_index(b),
            0xff69 if self.cgb => self.bg_palette.write_data(b),
            0xff6a if self.cgb => self.obj_palette.write_index(b),
            0xff6b if self.cgb => self.obj_palette.write_data(b),

            _ => {}
        }
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        self.back_buffer.as_ref()
    }

    pub fn dma_write(&mut self, addr: u16, data: u8) {
        // write condition is always true
        self.vram.write_sprite(addr as usize, data, LcdMode::VBlank);
    }

    pub fn hdma_write(&mut self, addr: u16, data: u8) {
        let addr = addr as usize;
        match addr {
            0x8000..=0x97ff => self.vram.write_tile(addr - 0x8000, data, LcdMode::VBlank),
            0x9800..=0x9fff => self.vram.write_map(addr - 0x9800, data, LcdMode::VBlank),
            _ => unreachable!(),
        }
    }

    pub fn hdma_avaliable(&mut self) -> bool {
        if self.hdma_avaliable {
            self.hdma_avaliable = false;
            true
        } else {
            false
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct LCDC: u8 {
        /// lcd display enable
        const LCD_ON                = 0b1000_0000;
        /// select window map address, 0=9800-9bff, 1=9c00-9fff
        const WINDOW_MAP            = 0b0100_0000;
        /// window display enable
        const WINDOW_ON             = 0b0010_0000;
        /// select bg/window tile address, 0=8800-97ff, 1=8000-8fff
        const BG_TILE_TABLE         = 0b0001_0000;
        /// select background map address, 0=9800-9bff, 1=9c00-9fff
        const BG_MAP                = 0b0000_1000;
        /// select object size, 0=8x8, 1=8x16
        const OBJECT_SIZE           = 0b0000_0100;
        /// object display enable
        const OBJECT_ON     = 0b0000_0010;
        /// background display enable
        const BG_ON                 = 0b0000_0001;
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
