use bitflags::bitflags;

/// interrupt enable register
const IE_PORT: u16 = 0xffff;
/// interrupt flag register
const IF_PORT: u16 = 0xff0f;

const INTERRUPT_VECTOR: [u16; 5] = [0x40, 0x48, 0x50, 0x58, 0x60];
const INTERRUPT_FLAGS: [InterruptFlag; 5] = [
    InterruptFlag::V_BLANK,
    InterruptFlag::LCD,
    InterruptFlag::TIMER,
    InterruptFlag::SERIAL,
    InterruptFlag::JOY_PAD,
];

#[repr(u8)]
pub enum Interrupt {
    VBlank = 0,
    Lcd = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}

bitflags! {
    #[derive(Default)]
    struct InterruptFlag: u8 {
        const V_BLANK = 0b0000_0001;
        const LCD     = 0b0000_0010;
        const TIMER   = 0b0000_0100;
        const SERIAL  = 0b0000_1000;
        const JOY_PAD = 0b0001_0000;
    }
}

pub struct InterruptHandler {
    ie_port: InterruptFlag,
    if_port: InterruptFlag,
}

impl InterruptHandler {
    pub fn new() -> Self {
        InterruptHandler {
            ie_port: InterruptFlag::empty(),
            if_port: InterruptFlag::empty(),
        }
    }

    pub fn has_interrupts(&self) -> bool {
        self.ie_port.intersects(self.if_port)
    }

    pub fn interrupt_enabled(&self, interrupt: Interrupt) -> bool {
        self.ie_port.contains(INTERRUPT_FLAGS[interrupt as usize])
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.if_port.insert(INTERRUPT_FLAGS[interrupt as usize])
    }

    pub fn service_interrupt(&mut self) -> Option<u16> {
        for (i, &flag) in INTERRUPT_FLAGS.iter().enumerate() {
            if self.ie_port.contains(flag) && self.if_port.contains(flag) {
                self.if_port.remove(flag);

                return Some(INTERRUPT_VECTOR[i]);
            }
        }

        None
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            IE_PORT => self.ie_port.bits(),
            IF_PORT => self.if_port.bits() | 0b1110_0000,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            IE_PORT => self.ie_port = InterruptFlag::from_bits_truncate(data),
            IF_PORT => self.if_port = InterruptFlag::from_bits_truncate(data),
            _ => unreachable!(),
        }
    }
}
