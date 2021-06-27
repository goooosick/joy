// Note: https://raw.githubusercontent.com/AntonioND/giibiiadvance/master/docs/other_docs/Game%20Boy%20Sound%20Operation.txt
// Note: https://www.reddit.com/r/EmuDev/comments/5gkwi5/gb_apu_sound_emulation/
mod parts;

mod mixer;
mod noise;
mod resampler;
mod square;
mod wave;

pub use self::mixer::Mixer;
pub use self::noise::Noise;
pub use self::parts::*;
pub use self::square::Square;
pub use self::wave::Wave;

use resampler::StereoBlipBuf;

pub struct Apu {
    frameseq: FrameSequencer,
    square1: Square,
    square2: Square,
    noise: Noise,
    wave: Wave,
    mixer: Mixer,

    regs: [u8; 0x30],
    sound_enable: bool,

    resampler: StereoBlipBuf,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            frameseq: FrameSequencer::new(),
            square1: Square::new(),
            square2: Square::new(),
            noise: Noise::new(),
            wave: Wave::new(),
            mixer: Mixer::new(),

            regs: [0u8; 0x30],
            sound_enable: false,

            resampler: StereoBlipBuf::new(
                crate::AUDIO_FREQUENCY / 30,
                crate::GB_CLOCK_SPEED,
                crate::AUDIO_FREQUENCY,
            ),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        let offset = addr as usize - 0xff10;
        match addr {
            // square 1
            0xff10 => self.regs[offset] | 0x80,
            0xff11 => self.regs[offset] | 0x3f,
            0xff12 => self.regs[offset] | 0x00,
            0xff13 => self.regs[offset] | 0xff,
            0xff14 => self.regs[offset] | 0xbf,

            // square 2
            0xff16 => self.regs[offset] | 0x3f,
            0xff17 => self.regs[offset] | 0x00,
            0xff18 => self.regs[offset] | 0xff,
            0xff19 => self.regs[offset] | 0xbf,

            // wave
            0xff1a => self.regs[offset] | 0x7f,
            0xff1b => self.regs[offset] | 0xff,
            0xff1c => self.regs[offset] | 0x9f,
            0xff1d => self.regs[offset] | 0xff,
            0xff1e => self.regs[offset] | 0xbf,

            // noise
            0xff20 => self.regs[offset] | 0xff,
            0xff21 => self.regs[offset] | 0x00,
            0xff22 => self.regs[offset] | 0x00,
            0xff23 => self.regs[offset] | 0xbf,

            0xff24 => self.regs[offset],
            0xff25 => self.regs[offset],
            0xff26 => self.read_control() | 0x70,

            // wave table
            0xff30..=0xff3f => self.wave.read_wave(addr - 0xff30),

            _ => 0xff,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        if !self.sound_enable {
            if addr != 0xff20 && addr != 0xff26 {
                return;
            }
        }

        self.regs[addr as usize - 0xff10] = data;
        match addr {
            0xff10 => self.square1.set_x0(data),
            0xff11 => self.square1.set_x1(data),
            0xff12 => self.square1.set_x2(data),
            0xff13 => self.square1.set_x3(data),
            0xff14 => self.square1.set_x4(data),

            0xff16 => self.square2.set_x1(data),
            0xff17 => self.square2.set_x2(data),
            0xff18 => self.square2.set_x3(data),
            0xff19 => self.square2.set_x4(data),

            0xff1a => self.wave.set_x0(data),
            0xff1b => self.wave.set_x1(data),
            0xff1c => self.wave.set_x2(data),
            0xff1d => self.wave.set_x3(data),
            0xff1e => self.wave.set_x4(data),

            0xff20 => self.noise.set_x1(data),
            0xff21 => self.noise.set_x2(data),
            0xff22 => self.noise.set_x3(data),
            0xff23 => self.noise.set_x4(data),

            0xff24 => self.mixer.set_volume(data),
            0xff25 => self.mixer.set_output(data),

            0xff26 => {
                if self.sound_enable && (data & 0b1000_0000 == 0) {
                    self.sound_off();
                } else if !self.sound_enable && (data & 0b1000_0000 != 0) {
                    self.frameseq.set_step(7);
                    self.sound_enable = true;
                }
            }

            0xff30..=0xff3f => self.wave.write_wave(addr - 0xff30, data),

            _ => {}
        };
    }

    fn read_control(&self) -> u8 {
        ((self.sound_enable as u8) << 7)
            | ((self.noise.is_on() as u8) << 3)
            | ((self.wave.is_on() as u8) << 2)
            | ((self.square2.is_on() as u8) << 1)
            | ((self.square1.is_on() as u8) << 0)
    }

    fn sound_off(&mut self) {
        for addr in 0xff10..0xff30 {
            if addr != 0xff26 {
                self.write(addr, 0);
            }
        }
        self.sound_enable = false;
    }

    pub fn update(&mut self, clocks: u32) {
        if self.sound_enable {
            for _ in 0..clocks {
                self.frame_sequence();

                let (so1, so2) = self.mixer.mix([
                    self.square1.next(),
                    self.square2.next(),
                    self.wave.next(),
                    self.noise.next(),
                ]);

                self.resampler.push((so1, so2));
            }
        } else {
            for _ in 0..clocks {
                self.resampler.push((0, 0));
            }
        }
    }

    fn frame_sequence(&mut self) {
        if let Some(step) = self.frameseq.next() {
            if step % 2 == 0 {
                self.square1.tick_len_counter();
                self.square2.tick_len_counter();
                self.wave.tick_len_counter();
                self.noise.tick_len_counter();
            }
            if step == 2 || step == 6 {
                self.square1.tick_sweep();
            }
            if step == 7 {
                self.square1.tick_envelope();
                self.square2.tick_envelope();
                self.noise.tick_envelope();
            }
        }
    }

    pub fn output(&mut self, cb: impl FnMut(&[i16])) {
        self.resampler.output(cb);
    }
}

#[derive(Eq, PartialEq)]
pub enum ChannelMode {
    On,
    Off,
}

#[derive(Eq, PartialEq)]
pub enum DacMode {
    On,
    Off,
}

const TRIGGER_MASK: u8 = 0b1000_0000;
const DUTY_MASK: u8 = 0b1100_0000;

fn freq_to_period(freq: u32) -> u32 {
    crate::GB_CLOCK_SPEED as u32 / freq
}

fn freq_high(freq: u32, data: u8) -> u32 {
    (freq & 0x00ff) | (((data & 0b0111) as u32) << 8)
}

fn freq_low(freq: u32, data: u8) -> u32 {
    (freq & 0xff00) | data as u32
}
