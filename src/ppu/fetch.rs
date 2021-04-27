use super::{BgAttr, Ppu, Sprite, TileValue, LCDC};
use std::collections::VecDeque;

pub enum FetchState {
    ReadTile,
    ReadData0,
    ReadData1,
    Push,
}

impl Default for FetchState {
    fn default() -> Self {
        FetchState::ReadTile
    }
}

#[derive(Default)]
pub struct Fetcher {
    ticks: usize,
    fb_offset: usize,
    state: FetchState,

    window_start: bool,
    map_start: usize,
    tile_index: usize,
    tile_attr: BgAttr,
    fx: usize,
    fy: usize,
    scx: usize,
    bg_fifo: VecDeque<(BgAttr, TileValue)>,

    sprite_fetching: bool,
    sprite_index: usize,
    sprite_fifo: VecDeque<(Sprite, TileValue)>,
}

impl Ppu {
    pub fn pixel_fetch_reset(&mut self, window_start: bool) {
        self.fet.ticks = 0;
        self.fet.fx = 0;
        self.fet.tile_index = 0;
        self.fet.window_start = window_start;
        self.fet.state = FetchState::ReadTile;
        self.fet.bg_fifo.clear();

        if !window_start {
            self.current_x = 0;
            self.fet.fb_offset = self.ly as usize * crate::GB_LCD_WIDTH * 3;
            self.fet.sprite_fetching = false;
            self.fet.sprite_fifo.clear();

            self.fet.map_start = self.lcdc.contains(LCDC::BG_MAP) as usize;
            self.fet.fy = self.ly.wrapping_add(self.scy) as usize;
            self.fet.scx = self.scx as usize & 0x07;
        } else {
            self.fet.map_start = self.lcdc.contains(LCDC::WINDOW_MAP) as usize;
            self.fet.fy = (self.ly - self.winy) as usize;
            self.fet.scx = 0;
        }

        self.fet.map_start = (self.fet.map_start * 0x400) + (self.fet.fy / 8 * 32);
    }

    pub fn pixel_fetch(&mut self) {
        if !self.fet.sprite_fetching && self.lcdc.contains(LCDC::OBJECT_ON) {
            if let Some(sp) = self.oam_buffer.first() {
                if sp.x <= self.current_x as i16 + 8 {
                    self.fet.sprite_fetching = true;
                    self.fet.state = FetchState::ReadTile;

                    // for convenience
                    self.oam_buffer[0].y -= 16;
                }
            }
        }

        self.fet.ticks += 1;
        if self.fet.ticks == 2 {
            self.fet.ticks = 0;

            if self.fet.sprite_fetching {
                self.sprite_fetching();
            }
            if !self.fet.sprite_fetching {
                self.bg_fetching();
            }
        }

        if !self.fet.sprite_fetching {
            if let Some((bg_attr, bg_tile)) = self.fet.bg_fifo.pop_front() {
                if self.fet.scx > 0 {
                    self.fet.scx -= 1;
                } else {
                    let bg_color = *self.bg_palette.color(bg_attr.bg_pal_index, bg_tile);
                    let color = if let Some((sp, sp_tile)) = self.fet.sprite_fifo.pop_front() {
                        let sp_color = *self.obj_palette.color(sp.palette, sp_tile);

                        let sp_priority = self.cgb && !self.lcdc.contains(LCDC::BG_ON);
                        if sp_priority
                            || (sp_tile != TileValue::B00
                                && (bg_tile == TileValue::B00
                                    || (!bg_attr.above_all && sp.above_bg)))
                        {
                            sp_color
                        } else {
                            bg_color
                        }
                    } else {
                        bg_color
                    };

                    self.frame_buffer[self.fet.fb_offset..][0..3].copy_from_slice(&color);

                    self.current_x += 1;
                    self.fet.fb_offset += 3;
                }
            }
        }
    }

    pub fn sprite_fetching(&mut self) {
        match self.fet.state {
            FetchState::ReadTile => {
                let sp = &self.oam_buffer[0];
                self.fet.sprite_index = if self.lcdc.contains(LCDC::OBJECT_SIZE) {
                    if sp.flip_y ^ ((self.ly as i16 - sp.y) < 8) {
                        sp.tile_index & 0xfe
                    } else {
                        sp.tile_index | 0x01
                    }
                } else {
                    sp.tile_index
                } as usize;

                self.fet.state = FetchState::ReadData0;
            }
            FetchState::ReadData0 => {
                self.fet.state = FetchState::ReadData1;
            }
            FetchState::ReadData1 => {
                self.fet.state = FetchState::Push;
            }
            FetchState::Push => {
                let sp = self.oam_buffer[0];

                let sprite_y = (self.ly as i16 - sp.y) as usize;
                let tile_y = (sprite_y & 7) ^ (sp.flip_y as usize * 7);
                let mut tile_line = self.vram.tile(sp.vram_bank, self.fet.sprite_index)[tile_y];

                if sp.flip_x {
                    tile_line.reverse();
                }

                for i in 0..8 {
                    if sp.x + (i as i16) < 8 {
                        continue;
                    }

                    if let Some(&(s, t)) = self.fet.sprite_fifo.get(i) {
                        if t == TileValue::B00 {
                            self.fet.sprite_fifo[i] = (sp.clone(), tile_line[i]);
                        } else if self.cgb {
                            if sp.index < s.index && tile_line[i] != TileValue::B00 {
                                self.fet.sprite_fifo[i] = (sp.clone(), tile_line[i]);
                            }
                        }
                    } else {
                        self.fet.sprite_fifo.push_back((sp.clone(), tile_line[i]));
                    }
                }

                self.oam_buffer.remove(0);
                self.fet.sprite_fetching = false;
                self.fet.state = FetchState::ReadTile;
            }
        }
    }

    pub fn bg_fetching(&mut self) {
        if !self.fet.window_start {
            if self.lcdc.contains(LCDC::WINDOW_ON) && self.ly >= self.winy {
                if self.current_x >= self.winx.saturating_sub(7) as usize {
                    self.pixel_fetch_reset(true);
                }
            }
        }

        match self.fet.state {
            FetchState::ReadTile => {
                let index = if self.fet.window_start {
                    self.fet.fx
                } else {
                    (self.scx as usize / 8 + self.fet.fx) & 0x1f
                };

                let map_index = self.fet.map_start + index;
                self.fet.tile_index = {
                    let index = self.vram.tilemap(map_index);
                    let pattern_offset = (!self.lcdc.contains(LCDC::BG_TILE_TABLE)) as usize;
                    index + (pattern_offset * (index < 0x80) as usize) * 0x100
                };
                self.fet.tile_attr = *self.vram.attrmap(map_index);

                self.fet.state = FetchState::ReadData0;
            }
            FetchState::ReadData0 => {
                self.fet.state = FetchState::ReadData1;
            }
            FetchState::ReadData1 => {
                self.fet.state = FetchState::Push;
            }
            FetchState::Push => {
                if self.fet.bg_fifo.len() == 0 {
                    let tile_y = (self.fet.fy & 0x07) ^ ((self.fet.tile_attr.flip_y as usize) * 7);
                    let mut tile_line =
                        if self.cgb || self.lcdc.contains(LCDC::BG_ON) || self.fet.window_start {
                            self.vram
                                .tile(self.fet.tile_attr.vram_bank, self.fet.tile_index)[tile_y]
                        } else {
                            // no background and window
                            self.fet.tile_attr = Default::default();
                            [TileValue::B00; 8]
                        };

                    if self.fet.tile_attr.flip_x {
                        tile_line.reverse();
                    }
                    tile_line.iter().for_each(|tile| {
                        self.fet.bg_fifo.push_back((self.fet.tile_attr, *tile));
                    });

                    self.fet.fx += 1;
                    self.fet.state = FetchState::ReadTile;
                } else {
                    self.fet.state = FetchState::Push;
                    self.fet.ticks = 1;
                }
            }
        }
    }
}
