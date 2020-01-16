#![allow(clippy::new_without_default)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::needless_range_loop)]

#[doc(inline)]
pub use self::{
    apu::Apu,
    cart::{load_cartridge, Cartridge},
    gameboy::GameBoy,
    interrupt::InterruptHandler,
    joypad::{Joypad, JoypadState},
    mem::Memory,
    ppu::Ppu,
    timer::Timer,
};

pub mod apu;
pub mod cart;
pub mod gameboy;
pub mod interrupt;
pub mod joypad;
pub mod mem;
pub mod ppu;
pub mod timer;

/// LCD screen width
pub const GB_LCD_WIDTH: usize = 160;
/// LCD screen height
pub const GB_LCD_HEIGHT: usize = 144;

/// Gameboy cpu clock speed - 4.194304MHz
pub const GB_CLOCK_SPEED: u32 = 4_194_304;
/// Emulator update speed
pub const GB_DEVICE_FPS: u32 = 60;

/// Audio frequency divider (1, 2, 4), using bigger value to reduce apu update rate.
pub const AUDIO_FREQ_DIVIDER: u32 = 4;
