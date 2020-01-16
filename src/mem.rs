pub struct Memory {
    work_ram: [u8; 0x4000],
    io_ports: [u8; 0x80],
    high_ram: [u8; 0x7f],
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            work_ram: [0u8; 0x4000],
            io_ports: [0u8; 0x80],
            high_ram: [0u8; 0x7f],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0xc000..=0xdfff => self.work_ram[addr - 0xc000],
            0xe000..=0xfdff => self.work_ram[addr - 0xe000],
            0xff00..=0xff7f => self.io_ports[addr - 0xff00],
            0xff80..=0xfffe => self.high_ram[addr - 0xff80],
            _ => panic!("invalid ram read: 0x{:04x}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        let addr = addr as usize;
        match addr {
            0xc000..=0xdfff => self.work_ram[addr - 0xc000] = data,
            0xe000..=0xfdff => self.work_ram[addr - 0xe000] = data,
            0xff00..=0xff7f => self.io_ports[addr - 0xff00] = data,
            0xff80..=0xfffe => self.high_ram[addr - 0xff80] = data,
            _ => panic!("invalid ram write: 0x{:04x}", addr),
        }
    }
}
