use super::MemoryBankController;
use time::{Duration, Instant};

enum Mode {
    Ram,
    Rtc,
}

enum RtcMode {
    Seconds,
    Minutes,
    Hours,
    DaysLow,
    DaysHigh,
}

pub struct MBC3 {
    ram: Vec<u8>,
    rom_bank: u8,
    ram_bank: u8,
    ram_enable: bool,

    max_rom: u8,

    mode: Mode,
    latch: Latch,

    rtc_mode: RtcMode,

    current: Duration,
    instant: Instant,

    carry: bool,
    halt: bool,
}

impl MBC3 {
    pub fn new(rom_size: usize, ram_size: usize) -> Self {
        MBC3 {
            ram: vec![0u8; ram_size],
            rom_bank: 0x01,
            ram_bank: 0x00,
            ram_enable: false,

            max_rom: (rom_size / 0x4000) as u8,

            mode: Mode::Ram,
            latch: Latch::Step0,

            rtc_mode: RtcMode::Seconds,

            current: Duration::zero(),
            instant: Instant::now(),

            carry: false,
            halt: true,
        }
    }
}

impl MemoryBankController for MBC3 {
    fn read(&self, rom: &[u8], addr: u16) -> u8 {
        // println!("{:04x}", addr);
        match addr {
            0x0000..=0x3fff => rom[addr as usize],
            0x4000..=0x7fff => {
                let addr = addr as usize - 0x4000;
                rom[addr + 0x4000 * self.rom_bank as usize]
            }
            0xa000..=0xbfff => {
                if self.ram_enable {
                    match self.mode {
                        Mode::Ram => {
                            let addr = (addr - 0xa000) as usize;
                            self.ram[addr + self.ram_bank as usize * 0x2000]
                        }
                        Mode::Rtc => {
                            let current = if self.halt || self.latch.latch() {
                                self.current
                            } else {
                                self.current + self.instant.elapsed()
                            };

                            match self.rtc_mode {
                                RtcMode::Seconds => {
                                    (current.whole_seconds() - current.whole_minutes() * 60) as u8
                                }
                                RtcMode::Minutes => {
                                    (current.whole_minutes() - current.whole_hours() * 60) as u8
                                }
                                RtcMode::Hours => {
                                    (current.whole_hours() - current.whole_days() * 24) as u8
                                }
                                RtcMode::DaysLow => (current.whole_days() as u16 & 0xff) as u8,
                                RtcMode::DaysHigh => {
                                    ((current.whole_days() > 255) as u8)
                                        | ((self.halt as u8) << 6)
                                        | (((current.whole_days() > 511) as u8) << 7)
                                }
                            }
                        }
                    }
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
                0x00 => n + 1,
                _ => n,
            }
        }

        match addr {
            // enable ram
            0x0000..=0x1fff => self.ram_enable = data & 0x0f == 0x0a,
            // rom banks
            0x2000..=0x3fff => self.rom_bank = map_rom_bank((data & 0x7f) % self.max_rom),
            // rtc or ram
            0x4000..=0x5fff => match data {
                0x00..=0x03 => {
                    self.mode = Mode::Ram;
                    self.ram_bank = data;
                }
                0x08..=0x0c => {
                    self.mode = Mode::Rtc;
                    self.rtc_mode = match data {
                        0x08 => RtcMode::Seconds,
                        0x09 => RtcMode::Minutes,
                        0x0a => RtcMode::Hours,
                        0x0b => RtcMode::DaysLow,
                        0x0c => RtcMode::DaysHigh,
                        _ => unreachable!(),
                    };
                }
                _ => {}
            },
            // latch clock data
            0x6000..=0x7fff => {
                self.latch.step(data);
            }
            // read extern ram banks
            0xa000..=0xbfff => {
                if self.ram_enable {
                    match self.mode {
                        Mode::Ram => {
                            let addr = addr - 0xa000 + self.ram_bank as u16 * 0x2000;
                            self.ram[addr as usize] = data;
                        }
                        Mode::Rtc => {
                            let current = if self.halt || self.latch.latch() {
                                self.current
                            } else {
                                self.current + self.instant.elapsed()
                            };
                            self.instant = Instant::now();

                            match self.rtc_mode {
                                RtcMode::Seconds => {
                                    let seconds =
                                        current.whole_seconds() - current.whole_minutes() * 60;
                                    self.current =
                                        current + Duration::seconds(data as i64 - seconds);
                                }
                                RtcMode::Minutes => {
                                    let minutes =
                                        current.whole_minutes() - current.whole_hours() * 60;
                                    self.current =
                                        current + Duration::minutes(data as i64 - minutes);
                                }
                                RtcMode::Hours => {
                                    let hours = current.whole_hours() - current.whole_days() * 24;
                                    self.current = current + Duration::hours(data as i64 - hours);
                                }
                                RtcMode::DaysLow => {
                                    let days = current.whole_days();
                                    let diff = current - Duration::days(days);
                                    self.current = if days >= 256 {
                                        Duration::days(256 + data as i64)
                                    } else {
                                        Duration::days(data as i64)
                                    } + diff;
                                }
                                RtcMode::DaysHigh => {
                                    // set days
                                    let days = current.whole_days();
                                    {
                                        if data & 0b01 != 0 && days < 256 {
                                            self.current += Duration::days(255);
                                        } else if data & 0b01 == 0 && days > 255 {
                                            self.current -= Duration::days(255);
                                        }
                                    }

                                    self.halt = (data & 0b0100_0000) != 0;
                                    self.carry = (data & 0b1000_0000) != 0;
                                    if !self.carry && days > 511 {
                                        self.current -= Duration::days(511);
                                    }
                                }
                            }
                        }
                    }
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
        "MBC3"
    }
}

#[derive(Eq, PartialEq)]
enum Latch {
    Step0,
    Step1,
    Latch0,
    Latch1,
}

impl Latch {
    fn latch(&self) -> bool {
        match self {
            Latch::Step0 | Latch::Step1 => false,
            Latch::Latch0 | Latch::Latch1 => true,
        }
    }

    fn step(&mut self, data: u8) {
        match self {
            Latch::Step0 => {
                if data == 0x00 {
                    *self = Latch::Step1;
                }
            }
            Latch::Step1 => {
                if data == 0x01 {
                    *self = Latch::Latch0;
                }
            }
            Latch::Latch0 => {
                if data == 0x00 {
                    *self = Latch::Latch1;
                }
            }
            Latch::Latch1 => {
                *self = Latch::Step0;
            }
        }
    }
}
