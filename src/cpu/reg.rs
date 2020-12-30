use std::ops::{Deref, DerefMut};

#[cfg(target_endian = "big")]
#[derive(Default)]
#[repr(C)]
pub struct Reg {
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub a: u8,
    _f: u8,
    _sp: u16,
    _pc: u16,
    pub f: Flag,
}

#[cfg(target_endian = "little")]
#[derive(Default)]
#[repr(C)]
pub struct Reg {
    pub c: u8,
    pub b: u8,
    pub e: u8,
    pub d: u8,
    pub l: u8,
    pub h: u8,
    _f: u8,
    pub a: u8,
    _sp: u16,
    _pc: u16,
    pub f: Flag,
}

#[derive(Default)]
#[repr(C)]
pub struct WordReg {
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    _af: u16,
    pub sp: u16,
    pub pc: u16,
    _f: Flag,
}

impl Deref for Reg {
    type Target = WordReg;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl DerefMut for Reg {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

const MASK_Z: u8 = 0b1000_0000;
const MASK_N: u8 = 0b0100_0000;
const MASK_H: u8 = 0b0010_0000;
const MASK_C: u8 = 0b0001_0000;

impl Reg {
    pub fn set_af(&mut self, data: u16) {
        self.a = ((data & 0xff00) >> 8) as u8;

        let f = (data & 0x00ff) as u8;
        self.f.zero = f & MASK_Z != 0;
        self.f.substract = f & MASK_N != 0;
        self.f.half_carry = f & MASK_H != 0;
        self.f.carry = f & MASK_C != 0;
    }

    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | self.f.to_u8() as u16
    }
}

impl std::fmt::Debug for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A:{:02X} F:{}{}{}{} BC:{:04X} DE:{:04x} HL:{:04x} SP:{:04x} PC:{:04x}",
            self.a,
            if self.f.zero { "Z" } else { "-" },
            if self.f.substract { "N" } else { "-" },
            if self.f.half_carry { "H" } else { "-" },
            if self.f.carry { "C" } else { "-" },
            self.bc,
            self.de,
            self.hl,
            self.sp,
            self.pc,
        )
    }
}

#[derive(Default)]
#[repr(C)]
pub struct Flag {
    pub zero: bool,
    pub substract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl Flag {
    pub fn clear(&mut self) {
        *self = Default::default();
    }

    pub fn to_u8(&self) -> u8 {
        ((self.zero as u8) << 7)
            | ((self.substract as u8) << 6)
            | ((self.half_carry as u8) << 5)
            | ((self.carry as u8) << 4)
    }
}
