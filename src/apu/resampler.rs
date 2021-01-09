use blip_buf::BlipBuf;

pub struct StereoBlipBuf {
    left_buf: BlipBuf,
    right_buf: BlipBuf,
    left_sample: i32,
    right_sample: i32,
    clocks: u32,
}

impl StereoBlipBuf {
    pub fn new(sample_count: u32, clock_rate: u32, sample_rate: u32) -> Self {
        let mut left_buf = BlipBuf::new(sample_count);
        let mut right_buf = BlipBuf::new(sample_count);
        left_buf.set_rates(clock_rate as f64, sample_rate as f64);
        right_buf.set_rates(clock_rate as f64, sample_rate as f64);

        Self {
            left_buf,
            right_buf,
            left_sample: 0,
            right_sample: 0,
            clocks: 0,
        }
    }

    pub fn push(&mut self, sample: (u16, u16)) {
        self.clocks += 1;
        self.left_buf
            .add_delta(self.clocks, sample.0 as i32 - self.left_sample);
        self.right_buf
            .add_delta(self.clocks, sample.1 as i32 - self.right_sample);
        self.left_sample = sample.0 as i32;
        self.right_sample = sample.1 as i32;
    }

    pub fn output(&mut self, mut cb: impl FnMut(&[i16])) {
        self.left_buf.end_frame(self.clocks);
        self.right_buf.end_frame(self.clocks);

        self.clocks = 0;

        let buf = &mut [0i16; 2049];
        while self.left_buf.samples_avail() > 0 {
            let count1 = self.left_buf.read_samples(buf, true);
            let count2 = self.right_buf.read_samples(&mut buf[1..], true);

            assert!(count1 == count2);

            cb(&buf[..(count1 + count2)]);
        }
    }
}
