use super::MemoryBankController;

enum Mode {
    Rom,
    Ram,
}

pub struct MBC1 {
    ram: Vec<u8>,
    rom_bank: u8,
    ram_bank: u8,
    ram_enable: bool,
    mode: Mode,

    max_rom: u8,
}

impl MBC1 {
    pub fn new(rom_size: usize, ram_size: usize) -> Self {
        MBC1 {
            ram: vec![0u8; ram_size],
            rom_bank: 0x01,
            ram_bank: 0x00,
            ram_enable: false,
            mode: Mode::Rom,

            max_rom: (rom_size / 0x4000) as u8,
        }
    }
}

impl MemoryBankController for MBC1 {
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
                    0xff
                }
            }

            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        fn map_rom_bank(n: u8) -> u8 {
            match n {
                0x00 | 0x20 | 0x40 | 0x60 => n + 1,
                _ => n,
            }
        }

        match addr {
            // enable ram
            0x0000..=0x1fff => self.ram_enable = self.ram.len() > 0 && (data & 0x0f == 0x0a),
            // lower 5 bits of rom_bank
            0x2000..=0x3fff => {
                let bank = ((self.rom_bank & 0b0110_0000) | (data & 0x1f)) % self.max_rom;
                self.rom_bank = map_rom_bank(bank);
            }
            // upper 5-6bits of ram/rom bank
            0x4000..=0x5fff => match self.mode {
                Mode::Rom => {
                    let bank = ((self.rom_bank & 0x1f) | (data & 0b11) << 5) % self.max_rom;
                    self.rom_bank = map_rom_bank(bank);
                }
                Mode::Ram => self.ram_bank = data & 0b11,
            },
            // mode select
            0x6000..=0x7fff => {
                self.mode = if data & 0b01 == 1 {
                    self.rom_bank = map_rom_bank(self.rom_bank % 0x20);
                    Mode::Ram
                } else {
                    self.ram_bank = 0x00;
                    Mode::Rom
                };
            }
            // write extern ram banks
            0xa000..=0xbfff => {
                if self.ram_enable {
                    let addr = addr - 0xa000 + self.ram_bank as u16 * 0x2000;
                    self.ram[addr as usize] = data;
                }
            }

            _ => unreachable!(),
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
        "MBC1"
    }
}
