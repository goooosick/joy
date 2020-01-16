pub use self::reg::*;
use crate::{mem::Memory, Apu, Cartridge, Ppu};
use crate::{InterruptHandler, Timer};
use crate::{Joypad, JoypadState};
use crate::{AUDIO_FREQ_DIVIDER, GB_CLOCK_SPEED, GB_DEVICE_FPS};

mod ins;
mod ops;
mod reg;

pub struct GameBoy {
    reg: Reg,
    mem: Memory,

    apu: Apu,
    ppu: Ppu,
    timer: Timer,
    joypad: Joypad,
    cart: Cartridge,
    interrupt_handler: InterruptHandler,

    interrupt_master_enable: bool,
    interrupt_enable_delay: bool,
    halt: bool,

    cycles: u32,
    debug_output: bool,
}

impl GameBoy {
    pub fn new(cart: Cartridge) -> Self {
        let mut gameboy = GameBoy {
            cart,
            mem: Memory::new(),
            reg: Default::default(),

            apu: Apu::new(),
            ppu: Ppu::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            interrupt_handler: InterruptHandler::new(),

            cycles: 0,

            halt: false,
            interrupt_master_enable: true,
            interrupt_enable_delay: false,

            debug_output: false,
        };

        gameboy.reset();
        gameboy.reg.pc = gameboy.cart.entry_point();

        gameboy
    }

    pub fn set_debug(&mut self, b: bool) {
        self.debug_output = b;
    }

    // Resets gameboy to the states after bootrom.
    // Ref: [powerupsequence](http://problemkaputt.de/pandocs.htm#powerupsequence)
    pub fn reset(&mut self) {
        self.reg.set_af(0x0100 | 0b1011_0000);
        self.reg.bc = 0x0013;
        self.reg.de = 0x00d8;
        self.reg.hl = 0x014d;
        self.reg.sp = 0xfffe;
        self.reg.pc = 0x0100;

        self.write_io(0x05, 0x00); // TIMA
        self.write_io(0x06, 0x00); // TMA
        self.write_io(0x07, 0x00); // TAC
        self.write_io(0x10, 0x80); // NR10
        self.write_io(0x11, 0xbf); // NR11
        self.write_io(0x12, 0xf3); // NR12
        self.write_io(0x14, 0xbf); // NR14
        self.write_io(0x16, 0x3f); // NR21
        self.write_io(0x17, 0x00); // NR22
        self.write_io(0x19, 0xbf); // NR24
        self.write_io(0x1a, 0x7f); // NR30
        self.write_io(0x1b, 0xff); // NR31
        self.write_io(0x1c, 0x9f); // NR32
        self.write_io(0x1e, 0xbf); // NR33
        self.write_io(0x20, 0xff); // NR41
        self.write_io(0x21, 0x00); // NR42
        self.write_io(0x22, 0x00); // NR43
        self.write_io(0x23, 0xbf); // NR30
        self.write_io(0x24, 0x77); // NR50
        self.write_io(0x25, 0xf3); // NR51
        self.write_io(0x26, 0xf1); // NR52
        self.write_io(0x40, 0x91); // LCDC
        self.write_io(0x42, 0x00); // SCY
        self.write_io(0x43, 0x00); // SCX
        self.write_io(0x45, 0x00); // LYC
        self.write_io(0x47, 0xfc); // BGP
        self.write_io(0x48, 0xff); // OBP0
        self.write_io(0x49, 0xff); // OBP1
        self.write_io(0x4a, 0x00); // WY
        self.write_io(0x4b, 0x00); // WX
        self.write_io(0xff, 0x00); // IE

        self.interrupt_master_enable = true;
        self.halt = false;
        self.cycles = 0;
    }

    pub fn step(&mut self) -> u32 {
        if self.debug_output {
            let op = self.read(self.reg.pc);
            print!("{:?} (cy: {})", self.reg, self.cycles);

            let (op, desc) = if op != 0xcb {
                (op, ops::OP_TABLE[op as usize].2)
            } else {
                let op = self.read(self.reg.pc + 1);
                (op, ops::OP_CB_TABLE[op as usize].2)
            };
            println!(" [{:02x}] {}", op, desc);
        }

        let cycles = self.cycles;

        self.handle_interrupts();
        if self.interrupt_enable_delay {
            self.interrupt_enable_delay = false;
            self.interrupt_master_enable = true;
        }

        if !self.halt {
            let op = self.fetch_byte();

            self.cycles += if op != 0xcb {
                self.dispatch_op(op);
                ops::OP_TABLE[op as usize].1
            } else {
                let op = self.fetch_byte();
                self.dispatch_op_cb(op);
                ops::OP_CB_TABLE[op as usize].1
            };
        } else {
            self.cycles += 1;
        };

        let cycles = self.cycles - cycles;
        self.timer.update(cycles * 4, &mut self.interrupt_handler);
        self.ppu.update(cycles * 4, &mut self.interrupt_handler);
        // here to divide the audio frequency
        self.apu.update(cycles * 4 / AUDIO_FREQ_DIVIDER);

        cycles
    }

    fn handle_interrupts(&mut self) {
        if self.interrupt_handler.has_interrupts() {
            // Note: quit halt mode but not serve interrupts.
            if self.halt {
                self.halt = false;
                self.cycles += 1;
            }

            if self.interrupt_master_enable {
                if let Some(addr) = self.interrupt_handler.service_interrupt() {
                    self.interrupt_master_enable = false;
                    self.cycles += 5;

                    self.push(self.reg.pc);
                    self.reg.pc = addr;
                };
            }
        }
    }

    pub fn emulate(&mut self, input: JoypadState) {
        self.joypad.set_input(input);

        let mut current = 0;
        const MAX_CYCLES: u32 = GB_CLOCK_SPEED / GB_DEVICE_FPS / 4;

        while current < MAX_CYCLES {
            current += self.step();
            self.joypad.update(&mut self.interrupt_handler);
        }
    }

    pub fn get_frame_buffer(&self) -> &[u32] {
        self.ppu.get_frame_buffer()
    }

    // *Consumes* the audio buffer.
    // The audio buffer has to be cleared after the call,
    // or the memory usage will keep growing.
    pub fn consume_audio_buffer(&mut self) -> &mut Vec<u8> {
        self.apu.consume_audio_buffer()
    }
}

impl GameBoy {
    fn fetch_byte(&mut self) -> u8 {
        let byte = self.read(self.reg.pc);
        self.reg.pc += 1;
        byte
    }

    fn fetch_word(&mut self) -> u16 {
        let word = self.read_word(self.reg.pc);
        self.reg.pc += 2;
        word
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => self.cart.read(addr),
            0x8000..=0x9fff => self.ppu.read(addr), // vram
            0xa000..=0xbfff => self.cart.read(addr),

            0xfe00..=0xfe9f => self.ppu.read(addr), // oam
            0xfea0..=0xfeff => 0x00,

            0xff00 => self.joypad.read(addr),
            0xff04..=0xff07 => self.timer.read(addr),
            0xff0f | 0xffff => self.interrupt_handler.read(addr),

            0xff10..=0xff3f => self.apu.read(addr),

            0xff46 => 0, // dma
            0xff40..=0xff4b => self.ppu.read(addr),

            _ => self.mem.read(addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7fff => self.cart.write(addr, data),
            0x8000..=0x9fff => self.ppu.write(addr, data), // vram
            0xa000..=0xbfff => self.cart.write(addr, data),

            0xfe00..=0xfe9f => self.ppu.write(addr, data), // oam
            0xfea0..=0xfeff => (),

            0xff00 => self.joypad.write(addr, data),
            0xff04..=0xff07 => self.timer.write(addr, data),
            0xff0f | 0xffff => self.interrupt_handler.write(addr, data),

            0xff10..=0xff3f => self.apu.write(addr, data),

            0xff46 => self.dma(data), // dma
            0xff40..=0xff4b => self.ppu.write(addr, data),

            _ => self.mem.write(addr, data),
        }
    }

    fn dma(&mut self, addr: u8) {
        // 0x100 aligned
        let src = (addr as u16) << 8;
        for offset in 0x00..0xa0 {
            self.ppu.dma_write(offset, self.read(src + offset));
        }
    }

    #[inline]
    fn read_io(&self, addr: u8) -> u8 {
        self.read(addr as u16 + 0xff00)
    }

    #[inline]
    fn write_io(&mut self, addr: u8, data: u8) {
        self.write(addr as u16 + 0xff00, data);
    }

    fn read_word(&self, addr: u16) -> u16 {
        let b0 = self.read(addr) as u16;
        let b1 = self.read(addr + 1) as u16;
        b0 | (b1 << 8)
    }

    fn write_word(&mut self, addr: u16, w: u16) {
        self.write(addr, w as u8);
        self.write(addr + 1, (w >> 8) as u8);
    }

    fn push(&mut self, word: u16) {
        self.write(self.reg.sp - 1, (word >> 8) as u8);
        self.write(self.reg.sp - 2, (word & 0xff) as u8);
        self.reg.sp -= 2;
    }

    fn pop(&mut self) -> u16 {
        let word = self.read_word(self.reg.sp);
        self.reg.sp += 2;
        word
    }
}
