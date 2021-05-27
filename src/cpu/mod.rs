pub use self::reg::*;
use crate::bus::Bus;

mod ins;
mod ops;
mod reg;

pub struct Cpu {
    reg: Reg,

    interrupt_master_enable: bool,
    interrupt_enable_delay: bool,
    halt: bool,

    cgb: bool,
}

impl Cpu {
    pub fn new(cgb: bool) -> Self {
        Cpu {
            reg: Default::default(),

            halt: false,
            interrupt_master_enable: true,
            interrupt_enable_delay: false,

            cgb,
        }
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

        if self.cgb {
            self.reg.a = 0x11;
        }

        self.interrupt_master_enable = true;
        self.halt = false;
    }

    pub fn step(&mut self, io: &mut Bus) -> u32 {
        if self.interrupt_enable_delay {
            self.interrupt_enable_delay = false;
            self.interrupt_master_enable = true;
        }

        let cycles = io.cycles();

        if !self.halt {
            self.handle_interrupts(io);

            let op = self.fetch_byte(io);

            if op != 0xcb {
                self.dispatch_op(op, io);
            } else {
                let op = self.fetch_byte(io);
                self.dispatch_op_cb(op, io);
            };
        } else {
            if io.interrupt_handler.has_interrupts() {
                self.halt = false;
            }
            io.step();
        };

        io.cycles() - cycles
    }

    fn handle_interrupts(&mut self, io: &mut Bus) {
        if self.interrupt_master_enable && io.interrupt_handler.has_interrupts() {
            if let Some(addr) = io.interrupt_handler.service_interrupt() {
                self.interrupt_master_enable = false;

                io.step();
                io.step();

                self.push(io, self.reg.pc);
                self.reg.pc = addr;
            }
        }
    }

    pub fn debug_output(&self, io: &Bus) {
        let op = io.read_direct(self.reg.pc);
        print!("{:?} (cy: {})", self.reg, io.mcycles());

        let (op, desc) = if op != 0xcb {
            (op, ops::OP_TABLE[op as usize].2)
        } else {
            let op = io.read_direct(self.reg.pc + 1);
            (op, ops::OP_CB_TABLE[op as usize].2)
        };
        println!(" [{:02x}] {}", op, desc);
    }
}

impl Cpu {
    fn read_byte(&self, io: &mut Bus, addr: u16) -> u8 {
        io.read(addr)
    }

    fn write_byte(&self, io: &mut Bus, addr: u16, data: u8) {
        io.write(addr, data);
    }

    fn read_word(&self, io: &mut Bus, addr: u16) -> u16 {
        let b0 = self.read_byte(io, addr) as u16;
        let b1 = self.read_byte(io, addr + 1) as u16;
        b0 | (b1 << 8)
    }

    fn write_word(&mut self, io: &mut Bus, addr: u16, data: u16) {
        self.write_byte(io, addr, data as u8);
        self.write_byte(io, addr + 1, (data >> 8) as u8);
    }

    fn read_io(&self, io: &mut Bus, addr: u8) -> u8 {
        self.read_byte(io, addr as u16 + 0xff00)
    }

    fn write_io(&mut self, io: &mut Bus, addr: u8, data: u8) {
        self.write_byte(io, addr as u16 + 0xff00, data);
    }

    fn push(&mut self, io: &mut Bus, word: u16) {
        self.write_byte(io, self.reg.sp - 1, (word >> 8) as u8);
        self.write_byte(io, self.reg.sp - 2, (word & 0xff) as u8);
        self.reg.sp -= 2;
    }

    fn pop(&mut self, io: &mut Bus) -> u16 {
        let word = self.read_word(io, self.reg.sp);
        self.reg.sp += 2;
        word
    }

    fn fetch_byte(&mut self, io: &mut Bus) -> u8 {
        let byte = self.read_byte(io, self.reg.pc);
        self.reg.pc += 1;
        byte
    }

    fn fetch_word(&mut self, io: &mut Bus) -> u16 {
        let word = self.read_word(io, self.reg.pc);
        self.reg.pc += 2;
        word
    }
}
