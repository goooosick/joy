use super::MemoryBankController;

pub struct MBC5 {
    ram: Vec<u8>,
    rom_bank: u16,
    ram_bank: u8,
    ram_enable: bool,

    max_rom: u16,
    max_ram: u8,
}

impl MBC5 {
    pub fn new(rom_size: usize, ram_size: usize) -> Self {
        MBC5 {
            ram: vec![0u8; ram_size],
            rom_bank: 0x01,
            ram_bank: 0x00,
            ram_enable: false,

            max_rom: (rom_size / 0x4000) as u16,
            max_ram: (ram_size / 0x2000) as u8,
        }
    }
}

impl MemoryBankController for MBC5 {
    fn read(&self, rom: &[u8], addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => rom[addr as usize],
            0x4000..=0x7fff => {
                let addr = addr as usize - 0x4000;
                rom[addr + 0x4000 * self.rom_bank as usize]
            }
            0xa000..=0xbfff => {
                let addr = (addr - 0xa000) as usize;
                if self.ram_enable {
                    self.ram[addr + self.ram_bank as usize * 0x2000]
                } else {
                    0
                }
            }

            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // enable ram
            0x0000..=0x1fff => self.ram_enable = (self.max_ram > 0) && (data & 0x0f == 0x0a),
            // lower 8 bits of rom bank
            0x2000..=0x2fff => {
                self.rom_bank = (self.rom_bank & 0x100) | data as u16;
                self.rom_bank %= self.max_rom;
            }
            // upper bit of rom bank
            0x3000..=0x3fff => {
                let bit = (data as u16 & 0b01) << 8;
                self.rom_bank = (self.rom_bank & 0x0ff) | bit;
                self.rom_bank %= self.max_rom;
            }
            // ram bank select
            0x4000..=0x5fff => {
                if self.max_ram > 0 {
                    self.ram_bank = (data & 0x0f) % self.max_ram;
                }
            }
            // write extern ram banks
            0xa000..=0xbfff => {
                if self.ram_enable {
                    let addr = addr - 0xa000 + self.ram_bank as u16 * 0x2000;
                    self.ram[addr as usize] = data;
                }
            }

            _ => {}
        }
    }

    fn get_ram(&self) -> Option<&[u8]> {
        if self.ram.len() > 0 {
            Some(self.ram.as_slice())
        } else {
            None
        }
    }

    fn get_ram_mut(&mut self) -> Option<&mut [u8]> {
        if self.ram.len() > 0 {
            Some(self.ram.as_mut_slice())
        } else {
            None
        }
    }

    fn mbc_type(&self) -> &'static str {
        "MBC5"
    }
}
