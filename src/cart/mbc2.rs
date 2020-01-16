use super::MemoryBankController;

pub struct MBC2 {
    ram: Vec<u8>,
    rom_bank: u8,
    ram_enable: bool,

    max_rom: u8,
}

impl MBC2 {
    pub fn new(rom_size: usize) -> Self {
        MBC2 {
            ram: vec![0u8; 0x200],
            rom_bank: 0x01,
            ram_enable: false,

            max_rom: (rom_size / 0x4000) as u8,
        }
    }
}

impl MemoryBankController for MBC2 {
    fn read(&self, rom: &[u8], addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => rom[addr as usize],
            0x4000..=0x7fff => {
                let addr = addr as usize - 0x4000;
                rom[addr + 0x4000 * self.rom_bank as usize]
            }
            0xa000..=0xa1ff => {
                if self.ram_enable {
                    self.ram[addr as usize - 0xa000] & 0x0f
                } else {
                    0xff
                }
            }

            _ => 0xff,
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        fn map_rom_bank(n: u8) -> u8 {
            match n {
                0x00 => n + 1,
                _ => n,
            }
        }
        match addr {
            // enable ram
            0x0000..=0x1fff => {
                if addr & 0x100 == 0 {
                    self.ram_enable = data & 0x0f == 0x0a;
                }
            }
            // select rom_bank
            0x2000..=0x3fff => {
                if addr & 0x100 != 0 {
                    self.rom_bank = map_rom_bank((data & 0x0f) % self.max_rom);
                }
            }
            // read extern ram banks
            0xa000..=0xa1ff => {
                if self.ram_enable {
                    self.ram[addr as usize - 0xa000] = data & 0x0f;
                }
            }

            _ => {}
        }
    }

    fn get_ram(&self) -> Option<&[u8]> {
        Some(self.ram.as_slice())
    }

    fn get_ram_mut(&mut self) -> Option<&mut [u8]> {
        Some(self.ram.as_mut_slice())
    }

    fn mbc_type(&self) -> &'static str {
        "MBC2"
    }
}
