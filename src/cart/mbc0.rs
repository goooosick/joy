use super::MemoryBankController;

pub struct MBC0;

impl MBC0 {
    pub fn new() -> Self {
        MBC0
    }
}

impl MemoryBankController for MBC0 {
    fn read(&self, rom: &[u8], addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => rom[addr as usize],
            _ => 0xff,
        }
    }

    fn write(&mut self, _addr: u16, _data: u8) {}

    fn mbc_type(&self) -> &'static str {
        "MBC0"
    }
}
