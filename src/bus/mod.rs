use crate::AUDIO_FREQ_DIVIDER;
use crate::{Apu, Cartridge, Ppu};
use crate::{InterruptHandler, Timer};
use crate::{Joypad, JoypadState};
use hdma::Hdma;

mod hdma;

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum SpeedMode {
    Normal = 0b00,
    Double = 0b01,
}

pub struct Bus {
    // memory
    work_ram0: Box<[u8; 0x1000]>,
    work_ram1: Box<[[u8; 0x1000]; 7]>,
    wram_bank: usize,

    io_ports: [u8; 0x80],
    high_ram: [u8; 0x80],

    // devices
    hdma: Hdma,
    timer: Timer,
    joypad: Joypad,
    pub(crate) cart: Cartridge,
    pub(crate) ppu: Ppu,
    pub(crate) apu: Apu,
    pub(crate) interrupt_handler: InterruptHandler,

    prepare_speed_switch: bool,
    speed_mode: SpeedMode,
    cycles: u32,
    mcycles: u32,
}

impl Bus {
    pub fn new(cart: Cartridge) -> Self {
        let cgb = cart.cgb();
        Self {
            work_ram0: Box::new([0u8; 0x1000]),
            work_ram1: Box::new([[0u8; 0x1000]; 7]),
            wram_bank: 0,

            io_ports: [0u8; 0x80],
            high_ram: [0u8; 0x80],

            hdma: Hdma::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            cart,
            ppu: Ppu::new(cgb),
            apu: Apu::new(),
            interrupt_handler: InterruptHandler::new(),

            prepare_speed_switch: false,
            speed_mode: SpeedMode::Normal,
            cycles: 0,
            mcycles: 0,
        }
    }

    pub fn step(&mut self) {
        let scaled_tcycles = 4 >> (self.speed_mode as u8);

        self.timer.update(4, &mut self.interrupt_handler);
        self.ppu.update(scaled_tcycles, &mut self.interrupt_handler);
        self.apu.update(scaled_tcycles / AUDIO_FREQ_DIVIDER);

        self.do_hdma();

        self.cycles += scaled_tcycles;
        self.mcycles += 1;
    }

    pub fn switch_mode(&mut self) {
        if self.prepare_speed_switch {
            if self.speed_mode == SpeedMode::Normal {
                self.speed_mode = SpeedMode::Double;
            } else {
                self.speed_mode = SpeedMode::Normal;
            }
            self.prepare_speed_switch = false;
        }
    }

    pub fn cycles(&self) -> u32 {
        self.cycles
    }

    pub fn mcycles(&self) -> u32 {
        self.mcycles
    }

    pub fn reset(&mut self) {
        for &(addr, data) in INIT_PORTS.iter() {
            self.write(addr, data);
        }

        self.cycles = 0;
        self.mcycles = 0;
    }
}

impl Bus {
    pub fn read(&mut self, addr: u16) -> u8 {
        let data = self.read_direct(addr);
        self.step();
        data
    }

    pub(crate) fn read_direct(&self, addr: u16) -> u8 {
        let index = addr as usize;
        let data = match addr {
            0x0000..=0x7fff => self.cart.read(addr),
            0x8000..=0x9fff => self.ppu.read(addr),
            0xa000..=0xbfff => self.cart.read(addr),
            0xc000..=0xcfff => self.work_ram0[index - 0xc000],
            0xd000..=0xdfff => self.work_ram1[self.wram_bank][index - 0xd000],
            0xe000..=0xefff => self.work_ram0[index - 0xe000],
            0xf000..=0xfdff => self.work_ram1[self.wram_bank][index - 0xf000],
            0xfe00..=0xfe9f => self.ppu.read(addr),
            0xfea0..=0xfeff => panic!("unused memory"),
            0xff00..=0xff7f => self.read_io(addr),
            0xff80..=0xfffe => self.high_ram[index - 0xff80],
            0xffff => self.interrupt_handler.read(addr),
        };

        data
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        let index = addr as usize;
        match addr {
            0x0000..=0x7fff => self.cart.write(addr, data),
            0x8000..=0x9fff => self.ppu.write(addr, data),
            0xa000..=0xbfff => self.cart.write(addr, data),
            0xc000..=0xcfff => self.work_ram0[index - 0xc000] = data,
            0xd000..=0xdfff => self.work_ram1[self.wram_bank][index - 0xd000] = data,
            0xe000..=0xefff => self.work_ram0[index - 0xe000] = data,
            0xf000..=0xfdff => self.work_ram1[self.wram_bank][index - 0xf000] = data,
            0xfe00..=0xfe9f => self.ppu.write(addr, data),
            0xfea0..=0xfeff => {}
            0xff00..=0xff7f => self.write_io(addr, data),
            0xff80..=0xfffe => self.high_ram[index - 0xff80] = data,
            0xffff => self.interrupt_handler.write(addr, data),
        };
        self.step();
    }

    fn read_io(&self, addr: u16) -> u8 {
        let cgb = self.cart.cgb();
        let index = addr as usize;
        match addr {
            0xff00 => self.joypad.read(addr),
            0xff04..=0xff07 => self.timer.read(addr),
            0xff0f => self.interrupt_handler.read(addr),
            0xff10..=0xff3f => self.apu.read(addr),
            0xff46 => 0,
            0xff40..=0xff4b => self.ppu.read(addr),

            0xff4d if cgb => ((self.speed_mode as u8) << 7) | (self.prepare_speed_switch as u8),
            0xff4f => self.ppu.read(addr),
            0xff51..=0xff55 if cgb => self.hdma.read(addr),
            0xff68..=0xff6b => self.ppu.read(addr),
            0xff70 if cgb => (self.wram_bank as u8 + 1) | 0xf8,
            _ => self.io_ports[index - 0xff00],
        }
    }

    fn write_io(&mut self, addr: u16, data: u8) {
        let cgb = self.cart.cgb();
        let index = addr as usize;
        match addr {
            0xff00 => self.joypad.write(addr, data),
            0xff04..=0xff07 => self.timer.write(addr, data),
            0xff0f => self.interrupt_handler.write(addr, data),
            0xff10..=0xff3f => self.apu.write(addr, data),
            0xff46 => self.do_dma(data),
            0xff40..=0xff4b => self.ppu.write(addr, data),

            0xff4d if cgb => self.prepare_speed_switch = (data & 0b01) != 0,
            0xff4f => self.ppu.write(addr, data),
            0xff51..=0xff55 if cgb => self.hdma.write(addr, data),
            0xff68..=0xff6b => self.ppu.write(addr, data),
            0xff70 if cgb => self.wram_bank = (data & 0b0111).saturating_sub(1) as usize,
            _ => self.io_ports[index - 0xff00] = data,
        }
    }

    pub fn set_input(&mut self, states: JoypadState) {
        self.joypad.set_input(states);
        self.joypad.update(&mut self.interrupt_handler);
    }

    fn do_dma(&mut self, addr: u8) {
        let src = (addr as u16) << 8;
        for offset in 0x00..0xa0 {
            let data = self.read_direct(src + offset);
            self.ppu.dma_write(offset, data);
        }
    }

    fn do_hdma(&mut self) {
        if let Some((src, dst, len)) = self.hdma.update(self.ppu.hdma_avaliable()) {
            for offset in 0..len {
                let data = self.read_direct(src + offset);
                self.ppu.hdma_write(dst + offset, data);
            }
        }
    }
}

const INIT_PORTS: [(u16, u8); 31] = [
    (0xff05, 0x00), // TIMA
    (0xff06, 0x00), // TMA
    (0xff07, 0x00), // TAC
    (0xff10, 0x80), // NR10
    (0xff11, 0xbf), // NR11
    (0xff12, 0xf3), // NR12
    (0xff14, 0xbf), // NR14
    (0xff16, 0x3f), // NR21
    (0xff17, 0x00), // NR22
    (0xff19, 0xbf), // NR24
    (0xff1a, 0x7f), // NR30
    (0xff1b, 0xff), // NR31
    (0xff1c, 0x9f), // NR32
    (0xff1e, 0xbf), // NR33
    (0xff20, 0xff), // NR41
    (0xff21, 0x00), // NR42
    (0xff22, 0x00), // NR43
    (0xff23, 0xbf), // NR30
    (0xff24, 0x77), // NR50
    (0xff25, 0xf3), // NR51
    (0xff26, 0xf1), // NR52
    (0xff40, 0x91), // LCDC
    (0xff42, 0x00), // SCY
    (0xff43, 0x00), // SCX
    (0xff45, 0x00), // LYC
    (0xff47, 0xfc), // BGP
    (0xff48, 0xff), // OBP0
    (0xff49, 0xff), // OBP1
    (0xff4a, 0x00), // WY
    (0xff4b, 0x00), // WX
    (0xffff, 0x00), // IE
];
