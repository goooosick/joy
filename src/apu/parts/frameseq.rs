use super::freq_to_period;
use super::Timer;

const FRAME_SEQUENCER_FREQUENCY: u32 = 512;

pub struct FrameSequencer {
    timer: Timer,
    step: u8,
}

impl FrameSequencer {
    pub fn new() -> Self {
        FrameSequencer {
            timer: Timer::new(freq_to_period(FRAME_SEQUENCER_FREQUENCY)),
            step: 7,
        }
    }

    pub fn next(&mut self) -> Option<u8> {
        if self.timer.tick() {
            self.step = (self.step + 1) % 8;
            Some(self.step)
        } else {
            None
        }
    }
}
