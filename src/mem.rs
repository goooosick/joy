pub struct Memory {
    work_ram0: Box<[u8; 0x1000]>,
    work_ram1: Vec<[u8; 0x1000]>,
    io_ports: [u8; 0x80],
    high_ram: [u8; 0x7f],

    wram_bank: usize,
}

impl Memory {
    pub fn new(cgb: bool) -> Self {
        let banks = if cgb { 7 } else { 1 };
        Memory {
            work_ram0: Box::new([0u8; 0x1000]),
            work_ram1: vec![[0u8; 0x1000]; banks],
            io_ports: [0u8; 0x80],
            high_ram: [0u8; 0x7f],

            wram_bank: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0xc000..=0xcfff => self.work_ram0[addr - 0xc000],
            0xd000..=0xdfff => self.work_ram1[self.wram_bank][addr - 0xd000],
            0xe000..=0xefff => self.work_ram0[addr - 0xe000],
            0xf000..=0xfdff => self.work_ram1[self.wram_bank][addr - 0xf000],
            0xff00..=0xff7f => self.io_ports[addr - 0xff00],
            0xff80..=0xfffe => self.high_ram[addr - 0xff80],
            _ => panic!("invalid ram read: 0x{:04x}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        let addr = addr as usize;
        match addr {
            0xc000..=0xcfff => self.work_ram0[addr - 0xc000] = data,
            0xd000..=0xdfff => self.work_ram1[self.wram_bank][addr - 0xd000] = data,
            0xe000..=0xefff => self.work_ram0[addr - 0xe000] = data,
            0xf000..=0xfdff => self.work_ram1[self.wram_bank][addr - 0xf000] = data,
            0xff00..=0xff7f => self.io_ports[addr - 0xff00] = data,
            0xff80..=0xfffe => self.high_ram[addr - 0xff80] = data,
            _ => panic!("invalid ram write: 0x{:04x}", addr),
        }
    }

    pub fn wram_bank(&self) -> u8 {
        (self.wram_bank as u8 + 1) | 0xf8
    }

    pub fn switch_wram_bank(&mut self, data: u8) {
        self.wram_bank = (data & 0b0111).saturating_sub(1) as usize;
    }
}
