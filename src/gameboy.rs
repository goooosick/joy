use crate::{Bus, Cartridge, Cpu, JoypadState};

pub struct Gameboy {
    cpu: Cpu,
    bus: Bus,
}

impl Gameboy {
    pub fn new(cart: Cartridge) -> Self {
        let cgb = cart.cgb();
        let mut g = Self {
            cpu: Cpu::new(cgb),
            bus: Bus::new(cart),
        };
        g.reset();

        g
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.bus.reset();
    }

    pub fn emulate(&mut self, max_cycles: u32, states: JoypadState) {
        self.bus.set_input(states);

        let mut current = 0;
        while current < max_cycles {
            current += self.cpu.step(&mut self.bus);
        }
    }

    pub fn save_game(&self) {
        self.bus.cart.save_game();
    }

    pub fn apu_output(&mut self, cb: impl FnMut(&[i16])) {
        self.bus.apu.output(cb);
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        self.bus.ppu.get_frame_buffer()
    }
}
