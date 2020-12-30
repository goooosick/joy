use crate::interrupt::{Interrupt, InterruptHandler};

/// divider register
const DIV_PORT: u16 = 0xff04;
/// timer counter
const TIMA_PORT: u16 = 0xff05;
/// timer modulo
const TMA_PORT: u16 = 0xff06;
/// timer control
const TAC_PORT: u16 = 0xff07;

const TIMER_ENABLE_MASK: u8 = 0b0100;
const CLOCK_SELECT_MASK: u8 = 0b0011;

const FREQUENCY: [u16; 4] = [1024, 16, 64, 256];

pub struct Timer {
    div_clocks: u16,
    timer_clocks: u16,

    tima: u8,
    tma: u8,
    tac: u8,

    frequency: u16,
    timer_enabled: bool,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div_clocks: 0,
            timer_clocks: 0,

            tima: 0,
            tma: 0,
            tac: 0,

            frequency: FREQUENCY[0],
            timer_enabled: false,
        }
    }

    pub fn update(&mut self, clocks: u32, interrupts: &mut InterruptHandler) {
        let clocks = clocks as u16;

        self.div_clocks = self.div_clocks.wrapping_add(clocks);

        if self.timer_enabled {
            self.timer_clocks += clocks;

            while self.timer_clocks >= self.frequency {
                self.timer_clocks -= self.frequency;
                if self.tima == 0xff {
                    self.tima = self.tma;
                    interrupts.request_interrupt(Interrupt::Timer);
                } else {
                    self.tima += 1;
                }
            }
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // div is just the high byte of internal clock
            DIV_PORT => ((self.div_clocks & 0xff00) >> 8) as u8,
            TIMA_PORT => self.tima,
            TMA_PORT => self.tma,
            TAC_PORT => self.tac,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            DIV_PORT => {
                self.div_clocks = 0;
                self.timer_clocks = 0;
            }
            TIMA_PORT => self.tima = data,
            TMA_PORT => self.tma = data,
            TAC_PORT => {
                self.tac = data;
                self.timer_enabled = data & TIMER_ENABLE_MASK != 0;
                self.frequency = FREQUENCY[(data & CLOCK_SELECT_MASK) as usize];
            }
            _ => unreachable!(),
        }
    }
}
