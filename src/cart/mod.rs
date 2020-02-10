use std::fs::OpenOptions;
use std::io::{Read, Write};

mod mbc0;
mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;

pub use self::mbc0::MBC0;
pub use self::mbc1::MBC1;
pub use self::mbc2::MBC2;
pub use self::mbc3::MBC3;
pub use self::mbc5::MBC5;

pub struct Cartridge {
    rom: Vec<u8>,
    mbc: Box<dyn MemoryBankController>,

    entry_point: u16,
    title: String,
    cgb: bool,
}

impl Cartridge {
    pub fn read(&self, addr: u16) -> u8 {
        self.mbc.read(self.rom.as_ref(), addr)
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.mbc.write(addr, data);
    }

    pub fn entry_point(&self) -> u16 {
        self.entry_point
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn cgb(&self) -> bool {
        self.cgb
    }

    pub fn save_game(&self) {
        save_game(self.mbc.get_ram(), self.title.as_str());
    }
}

pub fn load_cartridge<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Cartridge> {
    let rom = std::fs::read(path)?;
    let entry = 0x100;

    let title = std::str::from_utf8(
        rom[0x134..=0x142]
            .iter()
            .copied()
            .take_while(|x| *x != 0)
            .collect::<Vec<u8>>()
            .as_slice(),
    )
    .unwrap_or("unkown")
    .trim_end_matches(|n| n == 0 as char)
    .to_owned();

    let rom_size = 0x8000 << rom[0x148];
    let ram_size = match rom[0x149] {
        0x00 => 0x00,
        0x01 => 0x800,
        0x02 => 0x2000,
        0x03 => 0x8000,
        0x04 => 0x20000,
        0x05 => 0x10000,
        _ => unreachable!(),
    };
    assert_eq!(rom_size, rom.len());

    let cgb_flag = rom[0x0143];
    let cart_type = rom[0x147];
    let mut mbc: Box<dyn MemoryBankController> = match cart_type {
        0x00 => Box::new(MBC0::new()),
        0x01..=0x03 => Box::new(MBC1::new(rom_size, ram_size)),
        0x05..=0x06 => Box::new(MBC2::new(rom_size)),
        0x0f..=0x13 => Box::new(MBC3::new(rom_size, ram_size)),
        0x19..=0x1e => Box::new(MBC5::new(rom_size, ram_size)),
        _ => panic!("unimplemented type: 0x{:02x}", cart_type),
    };

    println!("title   : {}", title);
    println!("mbc type: 0x{:02x} - {}", cart_type, mbc.mbc_type());
    println!("rom size: 0x{:06x}", rom_size);
    println!("ram size: 0x{:06x}", ram_size);
    println!("cgb flag: 0x{:02x}", cgb_flag);

    load_save(mbc.get_ram_mut(), title.as_str());
    Ok(Cartridge {
        rom,
        mbc,
        entry_point: entry,
        title,
        cgb: cgb_flag == 0xc0 || cgb_flag == 0x80,
    })
}

fn load_save(ram: Option<&mut [u8]>, title: &str) {
    if let Some(ram) = ram {
        if ram.len() > 0 {
            let name = title.to_lowercase() + ".sav";
            if let Ok(mut file) = OpenOptions::new().read(true).open(name.as_str()) {
                if let Err(_) = file.read(ram) {
                    println!("load save failed: {:?}", name);
                }
            }
        }
    }
}

fn save_game(ram: Option<&[u8]>, title: &str) {
    if let Some(ram) = ram {
        if ram.len() > 0 {
            let name = title.to_lowercase() + ".sav";
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(name.as_str())
            {
                if let Err(_) = file.write_all(ram) {
                    eprintln!("save game failed: {}", name);
                } else {
                    println!("saved: {}", name);
                }
            } else {
                eprintln!("open save file failed: {}", name);
            }
        }
    }
}

pub trait MemoryBankController: Send {
    fn read(&self, rom: &[u8], addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
    fn mbc_type(&self) -> &'static str;

    fn get_ram(&self) -> Option<&[u8]> {
        None
    }
    fn get_ram_mut(&mut self) -> Option<&mut [u8]> {
        None
    }
}
