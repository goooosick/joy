use crate::interrupt::{Interrupt, InterruptHandler};
use bitflags::bitflags;

const BUTTON_SELECT_MASK: u8 = 0b0010_0000;
const DIRECTION_SELECT_MASK: u8 = 0b0001_0000;
const EMPTY_INPUT: u8 = 0b0000_1111;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct JoypadState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub start: bool,
    pub select: bool,
    pub button_a: bool,
    pub button_b: bool,
}

pub struct Joypad {
    select: SelectFlag,

    button_bits: u8,
    direction_bits: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            select: SelectFlag::all(),

            button_bits: 0xff,
            direction_bits: 0xff,
        }
    }

    pub fn read(&self, _addr: u16) -> u8 {
        let bits = if !self.select.contains(SelectFlag::BUTTON) {
            self.button_bits
        } else if !self.select.contains(SelectFlag::DIRECTION) {
            self.direction_bits
        } else {
            EMPTY_INPUT
        };
        self.select.bits() | bits | 0b1100_0000
    }

    pub fn write(&mut self, _addr: u16, data: u8) {
        self.select = SelectFlag::from_bits_truncate(data);
    }

    pub fn set_input(&mut self, states: JoypadState) {
        self.button_bits = ((!states.start as u8) << 3)
            | ((!states.select as u8) << 2)
            | ((!states.button_b as u8) << 1)
            | ((!states.button_a as u8) << 0);
        self.direction_bits = ((!states.down as u8) << 3)
            | ((!states.up as u8) << 2)
            | ((!states.left as u8) << 1)
            | ((!states.right as u8) << 0);
    }

    pub fn update(&mut self, interrupts: &mut InterruptHandler) {
        if interrupts.interrupt_enabled(Interrupt::Joypad) {
            if (!self.select.contains(SelectFlag::BUTTON) && self.button_bits != EMPTY_INPUT)
                || (!self.select.contains(SelectFlag::DIRECTION)
                    && self.direction_bits != EMPTY_INPUT)
            {
                interrupts.request_interrupt(Interrupt::Joypad);
            }
        }
    }
}

bitflags! {
    #[derive(Default)]
    struct SelectFlag: u8 {
        const BUTTON = BUTTON_SELECT_MASK;
        const DIRECTION = DIRECTION_SELECT_MASK;
    }
}
