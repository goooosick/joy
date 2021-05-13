use crate::interrupt::{Interrupt, InterruptHandler};
use crate::{GB_LCD_HEIGHT, GB_LCD_WIDTH};
use bitflags::bitflags;

use fetch::*;
use palette::*;
use vram::*;

mod fetch;
mod palette;
mod vram;

const MAX_SPRITE_PER_LINE: usize = 10;
const FRAME_BUFFER_SIZE: usize = GB_LCD_WIDTH * GB_LCD_HEIGHT * 3;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    win_ly: u8,

    hdma_avaliable: bool,
    bg_palette: Palette,
    obj_palette: Palette,
    cgb: bool,

    clocks: u32,
    current_x: usize,
    ly_154: bool,

    fet: Fetcher,
    oam_buffer: Vec<Sprite>,
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
            win_ly: 0,

            hdma_avaliable: false,
            bg_palette: Palette::build(cgb),
            obj_palette: Palette::build(cgb),
            cgb,

            clocks: 0,
            current_x: 0,
            ly_154: false,

            fet: Default::default(),
            oam_buffer: Default::default(),
        }
    }

    fn oam_search(&mut self) {
        // sprite size: 8 x 8 or 8 x 16
        let sprite_size = 8 * (1 + self.lcdc.contains(LCDC::OBJECT_SIZE) as i16);
        let ly = self.ly as i16;

        let mut sprites = self
            .vram
            .sprites()
            .iter()
            .filter(|sp| sp.y <= (ly + 16) && (sp.y + sprite_size) > (ly + 16))
            .take(MAX_SPRITE_PER_LINE)
            .filter(|sp| sp.x > 0)
            .cloned()
            .collect::<Vec<_>>();
        sprites.sort_by(|sp0, sp1| sp0.x.cmp(&sp1.x));
        self.oam_buffer = sprites;
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

                    self.oam_search();
                    self.pixel_fetch_reset(false);
                }
            }
            LcdMode::Transfer => {
                for _ in 0..clocks {
                    if !self.pixel_fetch() {
                        self.mode = LcdMode::HBlank;

                        self.hdma_avaliable = true;
                        stat_interrupt = self.stat.contains(STAT::HBLANK_INTERRUPT);

                        break;
                    }
                }
            }
            LcdMode::HBlank => {
                if self.clocks >= 376 {
                    self.clocks -= 376;

                    self.ly += 1;
                    self.check_lyc(interrupts);

                    if self.ly == 144 {
                        self.mode = LcdMode::VBlank;
                        self.ly_154 = false;

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
                // LY 153 lasts 4 cycles, but it's still in VBLANK.
                if self.ly == 153 && self.clocks == 4 {
                    self.ly = 0;
                    self.ly_154 = true;
                    self.check_lyc(interrupts);
                }

                if self.clocks >= 456 {
                    self.clocks -= 456;

                    if self.ly_154 {
                        self.ly = 0;
                        self.win_ly = 0;

                        self.mode = LcdMode::OamSearch;
                        stat_interrupt = self.stat.contains(STAT::OAM_INTERRUPT);
                    } else {
                        self.ly += 1;
                        self.check_lyc(interrupts);
                    }
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
                    self.win_ly = 0;
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
        let ret = self.hdma_avaliable;
        self.hdma_avaliable = false;
        ret
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
