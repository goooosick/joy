#[derive(PartialEq, Eq)]
enum HdmaMode {
    GDMA,
    HDMA,
}

pub struct Hdma {
    hdma_src: u16,
    hdma_dst: u16,
    hdma_len: u16,
    mode: Option<HdmaMode>,
}

impl Hdma {
    pub fn new() -> Self {
        Self {
            hdma_src: 0,
            hdma_dst: 0,
            hdma_len: 0,
            mode: None,
        }
    }

    pub fn update(&mut self, hblank: bool) -> Option<(u16, u16, u16)> {
        match self.mode {
            Some(HdmaMode::GDMA) => {
                let addr = (self.hdma_src, self.hdma_dst, self.hdma_len << 4);

                self.hdma_src += addr.2;
                self.hdma_dst += addr.2;
                self.hdma_len = 0;
                self.mode = None;

                Some(addr)
            }
            Some(HdmaMode::HDMA) if hblank => {
                let addr = (self.hdma_src, self.hdma_dst, 0x10);

                self.hdma_src += 0x10;
                self.hdma_dst += 0x10;
                self.hdma_len -= 1;

                if self.hdma_len == 0 {
                    self.mode = None;
                }

                return Some(addr);
            }
            _ => None,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff55 => {
                ((!self.mode.is_some() as u8) << 7) | (self.hdma_len.wrapping_sub(1) as u8 & 0x7f)
            }
            _ => 0xff,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff51 => self.hdma_src = (self.hdma_src & 0xff) | ((data as u16) << 8),
            0xff52 => self.hdma_src = (self.hdma_src & 0xff00) | ((data & 0xf0) as u16),
            0xff53 => {
                self.hdma_dst = (self.hdma_dst & 0xff) | (((data & 0x1f) as u16) << 8) | 0x8000
            }
            0xff54 => self.hdma_dst = (self.hdma_dst & 0xff00) | ((data & 0xf0) as u16),
            0xff55 => {
                self.hdma_len = (data as u16 & 0x7f) + 1;

                // stop hdma
                if data & 0x80 == 0 && self.mode == Some(HdmaMode::HDMA) {
                    self.hdma_len = 0x80 | data as u16;
                    self.mode = None;
                } else {
                    if data & 0x80 == 0 {
                        self.mode = Some(HdmaMode::GDMA);
                    } else {
                        self.mode = Some(HdmaMode::HDMA);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
