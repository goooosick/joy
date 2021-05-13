const DMA_LEN: u16 = 160;

pub struct Dma {
    src: u16,
    offset: u16,
    active: bool,
    delay: bool,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            src: 0,
            offset: 0,
            active: false,
            delay: false,
        }
    }

    pub fn update(&mut self) -> Option<(u16, u16)> {
        if self.delay {
            self.delay = false;
        } else if self.active {
            let out = Some((self.src + self.offset, self.offset));
            self.offset += 1;
            if self.offset == DMA_LEN {
                self.active = false;
            }
            return out;
        }

        None
    }

    pub fn read(&self, _addr: u16) -> u8 {
        0
    }

    pub fn write(&mut self, _addr: u16, data: u8) {
        self.src = (data as u16) << 8;
        self.offset = 0;
        self.active = true;
        self.delay = true;
    }
}
