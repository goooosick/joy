use super::GameBoy;

impl GameBoy {
    pub fn dispatch_op(&mut self, op: u8) {
        #[rustfmt::skip]
        match op {
            0x00 => {},
            0x01 => self.reg.bc = self.fetch_word(),
            0x02 => self.write(self.reg.bc, self.reg.a),
            0x03 => self.reg.bc= self.reg.bc.wrapping_add(1),
            0x04 => self.reg.b = self.inc(self.reg.b),
            0x05 => self.reg.b = self.dec(self.reg.b),
            0x06 => self.reg.b = self.fetch_byte(),
            0x07 => { self.reg.a = self.rlc(self.reg.a); self.reg.f.zero = false; },
            0x08 => {let word = self.fetch_word(); self.write_word(word, self.reg.sp); },
            0x09 => self.add_hl(self.reg.bc),
            0x0a => self.reg.a = self.read(self.reg.bc),
            0x0b => self.reg.bc= self.reg.bc.wrapping_sub(1),
            0x0c => self.reg.c = self.inc(self.reg.c),
            0x0d => self.reg.c = self.dec(self.reg.c),
            0x0e => self.reg.c = self.fetch_byte(),
            0x0f => { self.reg.a = self.rrc(self.reg.a); self.reg.f.zero = false; },

            // Note: `stop` instruction skips the next op 
            0x10 => {let _ = self.fetch_byte(); },
            0x11 => self.reg.de = self.fetch_word(),
            0x12 => self.write(self.reg.de, self.reg.a),
            0x13 => self.reg.de= self.reg.de.wrapping_add(1),
            0x14 => self.reg.d = self.inc(self.reg.d),
            0x15 => self.reg.d = self.dec(self.reg.d),
            0x16 => self.reg.d = self.fetch_byte(),
            0x17 => { self.reg.a = self.rl(self.reg.a); self.reg.f.zero = false; },
            0x18 => self.jump_relative(),
            0x19 => self.add_hl(self.reg.de),
            0x1a => self.reg.a = self.read(self.reg.de),
            0x1b => self.reg.de= self.reg.de.wrapping_sub(1),
            0x1c => self.reg.e = self.inc(self.reg.e),
            0x1d => self.reg.e = self.dec(self.reg.e),
            0x1e => self.reg.e = self.fetch_byte(),
            0x1f => { self.reg.a = self.rr(self.reg.a); self.reg.f.zero = false; },

            0x20 => self.jump_relative_cond(!self.reg.f.zero),
            0x21 => self.reg.hl = self.fetch_word(),
            0x22 => { self.write(self.reg.hl, self.reg.a); self.reg.hl = self.reg.hl.wrapping_add(1); },
            0x23 => self.reg.hl= self.reg.hl.wrapping_add(1),
            0x24 => self.reg.h = self.inc(self.reg.h),
            0x25 => self.reg.h = self.dec(self.reg.h),
            0x26 => self.reg.h = self.fetch_byte(),
            0x27 => self.daa(),
            0x28 => self.jump_relative_cond(self.reg.f.zero),
            0x29 => self.add_hl(self.reg.hl),
            0x2a => { self.reg.a = self.read(self.reg.hl); self.reg.hl = self.reg.hl.wrapping_add(1); },
            0x2b => self.reg.hl= self.reg.hl.wrapping_sub(1),
            0x2c => self.reg.l = self.inc(self.reg.l),
            0x2d => self.reg.l = self.dec(self.reg.l),
            0x2e => self.reg.l = self.fetch_byte(),
            0x2f => self.cpl(),

            0x30 => self.jump_relative_cond(!self.reg.f.carry),
            0x31 => self.reg.sp = self.fetch_word(),
            0x32 => { self.write(self.reg.hl, self.reg.a); self.reg.hl = self.reg.hl.wrapping_sub(1); },
            0x33 => self.reg.sp= self.reg.sp.wrapping_add(1),
            0x34 => {
                let old = self.read(self.reg.hl);
                let new = old.wrapping_add(1);
                self.write(self.reg.hl, new);

                self.reg.f.zero = new == 0;
                self.reg.f.substract = false;
                self.reg.f.half_carry = (old & 0x0f) == 0x0f;
            },
            0x35 => {
                let old = self.read(self.reg.hl);
                let new = old.wrapping_sub(1);
                self.write(self.reg.hl, new);

                self.reg.f.zero = new == 0;
                self.reg.f.substract = true;
                self.reg.f.half_carry = (old & 0x0f) == 0x00;
            },
            0x36 => { let byte = self.fetch_byte(); self.write(self.reg.hl, byte); },
            0x37 => self.scf(),
            0x38 => self.jump_relative_cond(self.reg.f.carry),
            0x39 => self.add_hl(self.reg.sp),
            0x3a => { self.reg.a = self.read(self.reg.hl); self.reg.hl = self.reg.hl.wrapping_sub(1); },
            0x3b => self.reg.sp= self.reg.sp.wrapping_sub(1),
            0x3c => self.reg.a = self.inc(self.reg.a),
            0x3d => self.reg.a = self.dec(self.reg.a),
            0x3e => self.reg.a = self.fetch_byte(),
            0x3f => self.ccf(),

            0x40 => self.reg.b = self.reg.b,
            0x41 => self.reg.b = self.reg.c,
            0x42 => self.reg.b = self.reg.d,
            0x43 => self.reg.b = self.reg.e,
            0x44 => self.reg.b = self.reg.h,
            0x45 => self.reg.b = self.reg.l,
            0x46 => self.reg.b = self.read(self.reg.hl),
            0x47 => self.reg.b = self.reg.a,
            0x48 => self.reg.c = self.reg.b,
            0x49 => self.reg.c = self.reg.c,
            0x4a => self.reg.c = self.reg.d,
            0x4b => self.reg.c = self.reg.e,
            0x4c => self.reg.c = self.reg.h,
            0x4d => self.reg.c = self.reg.l,
            0x4e => self.reg.c = self.read(self.reg.hl),
            0x4f => self.reg.c = self.reg.a,

            0x50 => self.reg.d = self.reg.b,
            0x51 => self.reg.d = self.reg.c,
            0x52 => self.reg.d = self.reg.d,
            0x53 => self.reg.d = self.reg.e,
            0x54 => self.reg.d = self.reg.h,
            0x55 => self.reg.d = self.reg.l,
            0x56 => self.reg.d = self.read(self.reg.hl),
            0x57 => self.reg.d = self.reg.a,
            0x58 => self.reg.e = self.reg.b,
            0x59 => self.reg.e = self.reg.c,
            0x5a => self.reg.e = self.reg.d,
            0x5b => self.reg.e = self.reg.e,
            0x5c => self.reg.e = self.reg.h,
            0x5d => self.reg.e = self.reg.l,
            0x5e => self.reg.e = self.read(self.reg.hl),
            0x5f => self.reg.e = self.reg.a,

            0x60 => self.reg.h = self.reg.b,
            0x61 => self.reg.h = self.reg.c,
            0x62 => self.reg.h = self.reg.d,
            0x63 => self.reg.h = self.reg.e,
            0x64 => self.reg.h = self.reg.h,
            0x65 => self.reg.h = self.reg.l,
            0x66 => self.reg.h = self.read(self.reg.hl),
            0x67 => self.reg.h = self.reg.a,
            0x68 => self.reg.l = self.reg.b,
            0x69 => self.reg.l = self.reg.c,
            0x6a => self.reg.l = self.reg.d,
            0x6b => self.reg.l = self.reg.e,
            0x6c => self.reg.l = self.reg.h,
            0x6d => self.reg.l = self.reg.l,
            0x6e => self.reg.l = self.read(self.reg.hl),
            0x6f => self.reg.l = self.reg.a,

            0x70 => self.write(self.reg.hl, self.reg.b),
            0x71 => self.write(self.reg.hl, self.reg.c),
            0x72 => self.write(self.reg.hl, self.reg.d),
            0x73 => self.write(self.reg.hl, self.reg.e),
            0x74 => self.write(self.reg.hl, self.reg.h),
            0x75 => self.write(self.reg.hl, self.reg.l),
            0x76 => self.halt = true,
            0x77 => self.write(self.reg.hl, self.reg.a),
            0x78 => self.reg.a = self.reg.b,
            0x79 => self.reg.a = self.reg.c,
            0x7a => self.reg.a = self.reg.d,
            0x7b => self.reg.a = self.reg.e,
            0x7c => self.reg.a = self.reg.h,
            0x7d => self.reg.a = self.reg.l,
            0x7e => self.reg.a = self.read(self.reg.hl),
            0x7f => self.reg.a = self.reg.a,

            0x80 => self.add(self.reg.b),
            0x81 => self.add(self.reg.c),
            0x82 => self.add(self.reg.d),
            0x83 => self.add(self.reg.e),
            0x84 => self.add(self.reg.h),
            0x85 => self.add(self.reg.l),
            0x86 => self.add(self.read(self.reg.hl)),
            0x87 => self.add(self.reg.a),
            0x88 => self.adc(self.reg.b),
            0x89 => self.adc(self.reg.c),
            0x8a => self.adc(self.reg.d),
            0x8b => self.adc(self.reg.e),
            0x8c => self.adc(self.reg.h),
            0x8d => self.adc(self.reg.l),
            0x8e => self.adc(self.read(self.reg.hl)),
            0x8f => self.adc(self.reg.a),

            0x90 => self.sub(self.reg.b),
            0x91 => self.sub(self.reg.c),
            0x92 => self.sub(self.reg.d),
            0x93 => self.sub(self.reg.e),
            0x94 => self.sub(self.reg.h),
            0x95 => self.sub(self.reg.l),
            0x96 => self.sub(self.read(self.reg.hl)),
            0x97 => self.sub(self.reg.a),
            0x98 => self.sbc(self.reg.b),
            0x99 => self.sbc(self.reg.c),
            0x9a => self.sbc(self.reg.d),
            0x9b => self.sbc(self.reg.e),
            0x9c => self.sbc(self.reg.h),
            0x9d => self.sbc(self.reg.l),
            0x9e => self.sbc(self.read(self.reg.hl)),
            0x9f => self.sbc(self.reg.a),

            0xa0 => self.and(self.reg.b),
            0xa1 => self.and(self.reg.c),
            0xa2 => self.and(self.reg.d),
            0xa3 => self.and(self.reg.e),
            0xa4 => self.and(self.reg.h),
            0xa5 => self.and(self.reg.l),
            0xa6 => self.and(self.read(self.reg.hl)),
            0xa7 => self.and(self.reg.a),
            0xa8 => self.xor(self.reg.b),
            0xa9 => self.xor(self.reg.c),
            0xaa => self.xor(self.reg.d),
            0xab => self.xor(self.reg.e),
            0xac => self.xor(self.reg.h),
            0xad => self.xor(self.reg.l),
            0xae => self.xor(self.read(self.reg.hl)),
            0xaf => self.xor(self.reg.a),

            0xb0 => self.or(self.reg.b),
            0xb1 => self.or(self.reg.c),
            0xb2 => self.or(self.reg.d),
            0xb3 => self.or(self.reg.e),
            0xb4 => self.or(self.reg.h),
            0xb5 => self.or(self.reg.l),
            0xb6 => self.or(self.read(self.reg.hl)),
            0xb7 => self.or(self.reg.a),
            0xb8 => self.cp(self.reg.b),
            0xb9 => self.cp(self.reg.c),
            0xba => self.cp(self.reg.d),
            0xbb => self.cp(self.reg.e),
            0xbc => self.cp(self.reg.h),
            0xbd => self.cp(self.reg.l),
            0xbe => self.cp(self.read(self.reg.hl)),
            0xbf => self.cp(self.reg.a),

            0xc0 => self.ret_cond(!self.reg.f.zero),
            0xc1 => self.reg.bc = self.pop(),
            0xc2 => self.jump_cond(!self.reg.f.zero),
            0xc3 => self.reg.pc = self.fetch_word(),
            0xc4 => self.call_cond(!self.reg.f.zero),
            0xc5 => self.push(self.reg.bc),
            0xc6 => { let byte = self.fetch_byte(); self.add(byte); },
            0xc7 => self.call(0x00),
            0xc8 => self.ret_cond(self.reg.f.zero),
            0xc9 => self.ret(),
            0xca => self.jump_cond(self.reg.f.zero),
            0xcb => unreachable!(),
            0xcc => self.call_cond(self.reg.f.zero),
            0xcd => { let word = self.fetch_word(); self.call(word); },
            0xce => { let byte = self.fetch_byte(); self.adc(byte); },
            0xcf => self.call(0x08),

            0xd0 => self.ret_cond(!self.reg.f.carry),
            0xd1 => self.reg.de = self.pop(),
            0xd2 => self.jump_cond(!self.reg.f.carry),
            0xd3 => panic!("invalid op: {:02x}", op),
            0xd4 => self.call_cond(!self.reg.f.carry),
            0xd5 => self.push(self.reg.de),
            0xd6 => { let byte = self.fetch_byte(); self.sub(byte); },
            0xd7 => self.call(0x10),
            0xd8 => self.ret_cond(self.reg.f.carry),
            0xd9 => self.reti(),
            0xda => self.jump_cond(self.reg.f.carry),
            0xdb => panic!("invalid op: {:02x}", op),
            0xdc => self.call_cond(self.reg.f.carry),
            0xdd => panic!("invalid op: {:02x}", op),
            0xde => { let byte = self.fetch_byte(); self.sbc(byte); },
            0xdf => self.call(0x18),

            0xe0 => { let port = self.fetch_byte(); self.write_io(port, self.reg.a); },
            0xe1 => self.reg.hl = self.pop(),
            0xe2 => self.write_io(self.reg.c, self.reg.a),
            0xe3 => panic!("invalid op: {:02x}", op),
            0xe4 => panic!("invalid op: {:02x}", op),
            0xe5 => self.push(self.reg.hl),
            0xe6 => { let byte = self.fetch_byte(); self.and(byte); },
            0xe7 => self.call(0x20),
            0xe8 => { let byte = self.fetch_byte(); self.add_sp(byte); },
            // Note: `JP (HL)` is actually `JP HL`
            0xe9 => self.reg.pc = self.reg.hl,
            0xea => { let addr = self.fetch_word(); self.write(addr, self.reg.a); },
            0xeb => panic!("invalid op: {:02x}", op),
            0xec => panic!("invalid op: {:02x}", op),
            0xed => panic!("invalid op: {:02x}", op),
            0xee => { let byte = self.fetch_byte(); self.xor(byte); },
            0xef => self.call(0x28),

            0xf0 => { let port = self.fetch_byte(); self.reg.a = self.read_io(port); },
            0xf1 => { let af = self.pop() & 0xfff0; self.reg.set_af(af); },
            0xf2 => self.reg.a = self.read_io(self.reg.c),
            0xf3 => self.interrupt_master_enable = false,
            0xf4 => panic!("invalid op: {:02x}", op),
            0xf5 => self.push(self.reg.af()),
            0xf6 => { let byte = self.fetch_byte(); self.or(byte); },
            0xf7 => self.call(0x30),
            0xf8 => {
                let byte = self.fetch_byte();
                let sp = self.reg.sp;
                self.add_sp(byte);
                self.reg.hl = self.reg.sp;
                self.reg.sp = sp;
            },
            0xf9 => self.reg.sp = self.reg.hl,
            0xfa => { let addr = self.fetch_word(); self.reg.a = self.read(addr); },
            0xfb => self.interrupt_enable_delay = true,
            0xfc => panic!("invalid op: {:02x}", op),
            0xfd => panic!("invalid op: {:02x}", op),
            0xfe => { let byte = self.fetch_byte(); self.cp(byte); },
            0xff => self.call(0x38),
        };
    }

    pub fn dispatch_op_cb(&mut self, op: u8) {
        const FN_ONE: [for<'r> fn(&'r mut GameBoy, u8) -> u8; 8] = [
            GameBoy::rlc,
            GameBoy::rrc,
            GameBoy::rl,
            GameBoy::rr,
            GameBoy::sla,
            GameBoy::sra,
            GameBoy::swap,
            GameBoy::srl,
        ];
        const FN_TWO: [for<'r> fn(&'r mut GameBoy, u8, u8) -> u8; 3] =
            [GameBoy::bit, GameBoy::res, GameBoy::set];

        match op {
            0x00..=0x3f => {
                // 0b00FF_FRRR, F -> function, R -> register
                let func = FN_ONE[((op & 0b0011_1000) >> 3) as usize];
                match op & 0b0111 {
                    0 => self.reg.b = func(self, self.reg.b),
                    1 => self.reg.c = func(self, self.reg.c),
                    2 => self.reg.d = func(self, self.reg.d),
                    3 => self.reg.e = func(self, self.reg.e),
                    4 => self.reg.h = func(self, self.reg.h),
                    5 => self.reg.l = func(self, self.reg.l),
                    6 => {
                        let value = func(self, self.read(self.reg.hl));
                        self.write(self.reg.hl, value);
                    }
                    7 => self.reg.a = func(self, self.reg.a),
                    _ => unreachable!(),
                }
            }
            _ => {
                // 0bFFNN_NRRR, N -> nth bit
                let index = ((op & 0b1100_0000) >> 6) as usize - 1;
                assert!(index <= 2);
                let func = FN_TWO[index];
                let n = (op & 0b0011_1000) >> 3;

                match op & 0b0111 {
                    0 => self.reg.b = func(self, n, self.reg.b),
                    1 => self.reg.c = func(self, n, self.reg.c),
                    2 => self.reg.d = func(self, n, self.reg.d),
                    3 => self.reg.e = func(self, n, self.reg.e),
                    4 => self.reg.h = func(self, n, self.reg.h),
                    5 => self.reg.l = func(self, n, self.reg.l),
                    6 => {
                        let value = func(self, n, self.read(self.reg.hl));
                        self.write(self.reg.hl, value);
                    }
                    7 => self.reg.a = func(self, n, self.reg.a),
                    _ => unreachable!(),
                }
            }
        };
    }
}

impl GameBoy {
    #[inline]
    fn jump_relative(&mut self) {
        let offset = self.fetch_byte();
        self.reg.pc = add_relative(self.reg.pc, offset);
    }

    fn jump_relative_cond(&mut self, cond: bool) {
        let offset = self.fetch_byte();
        if cond {
            self.reg.pc = add_relative(self.reg.pc, offset);
            self.cycles += 1;
        }
    }

    fn jump_cond(&mut self, cond: bool) {
        let addr = self.fetch_word();
        if cond {
            self.reg.pc = addr;
            self.cycles += 1;
        }
    }

    fn ret(&mut self) {
        self.reg.pc = self.pop();
    }

    fn reti(&mut self) {
        self.reg.pc = self.pop();
        self.interrupt_master_enable = true;
    }

    fn ret_cond(&mut self, cond: bool) {
        if cond {
            self.ret();
            self.cycles += 3;
        }
    }

    fn call(&mut self, addr: u16) {
        self.push(self.reg.pc);
        self.reg.pc = addr;
    }

    fn call_cond(&mut self, cond: bool) {
        let addr = self.fetch_word();
        if cond {
            self.call(addr);
            self.cycles += 3;
        }
    }

    fn add(&mut self, right: u8) {
        let left = self.reg.a;
        let (new, overflow) = left.overflowing_add(right);

        self.reg.f.zero = new == 0;
        self.reg.f.substract = false;
        self.reg.f.half_carry = ((left & 0x0f) + (right & 0x0f)) > 0x0f;
        self.reg.f.carry = overflow;
        self.reg.a = new;
    }

    fn adc(&mut self, right: u8) {
        // two overflowing add
        let carry = self.reg.f.carry as u8;
        let (new, ov0) = self.reg.a.overflowing_add(right);
        let (new, ov1) = new.overflowing_add(carry);
        let half_carry = ((self.reg.a & 0x0f) + (right & 0x0f)) > (0x0f - carry);

        self.reg.f.zero = new == 0;
        self.reg.f.substract = false;
        self.reg.f.half_carry = half_carry;
        self.reg.f.carry = ov0 || ov1;
        self.reg.a = new;
    }

    fn sub(&mut self, right: u8) {
        let left = self.reg.a;
        let (new, overflow) = left.overflowing_sub(right);

        self.reg.f.zero = new == 0;
        self.reg.f.substract = true;
        self.reg.f.half_carry = (left & 0x0f) < (right & 0x0f);
        self.reg.f.carry = overflow;
        self.reg.a = new;
    }

    fn sbc(&mut self, right: u8) {
        // two overflowing sub
        let left = self.reg.a;
        let carry = self.reg.f.carry as u8;
        let (new, ov0) = left.overflowing_sub(right);
        let (new, ov1) = new.overflowing_sub(carry);

        self.reg.f.zero = new == 0;
        self.reg.f.substract = true;
        self.reg.f.half_carry = (left & 0x0f) < ((right & 0x0f) + carry);
        self.reg.f.carry = ov0 || ov1;
        self.reg.a = new;
    }

    fn and(&mut self, right: u8) {
        let new = self.reg.a & right;
        self.reg.clear_flag();
        self.reg.f.half_carry = true;
        self.reg.f.zero = new == 0;
        self.reg.a = new;
    }

    fn xor(&mut self, right: u8) {
        let new = self.reg.a ^ right;
        self.reg.clear_flag();
        self.reg.f.zero = new == 0;
        self.reg.a = new;
    }

    fn or(&mut self, right: u8) {
        let new = self.reg.a | right;
        self.reg.clear_flag();
        self.reg.f.zero = new == 0;
        self.reg.a = new;
    }

    fn cp(&mut self, right: u8) {
        let left = self.reg.a;
        self.reg.f.zero = left == right;
        self.reg.f.substract = true;
        self.reg.f.half_carry = (left & 0x0f) < (right & 0x0f);
        self.reg.f.carry = left < right;
    }

    fn inc(&mut self, value: u8) -> u8 {
        let new = value.wrapping_add(1);

        self.reg.f.zero = new == 0;
        self.reg.f.substract = false;
        self.reg.f.half_carry = (value & 0x0f) == 0x0f;
        new
    }

    fn dec(&mut self, value: u8) -> u8 {
        let new = value.wrapping_sub(1);

        self.reg.f.zero = new == 0;
        self.reg.f.substract = true;
        self.reg.f.half_carry = (value & 0x0f) == 0x00;
        new
    }

    fn rl(&mut self, value: u8) -> u8 {
        let flag_c = (value & 0x80) >> 7;
        let new = (value << 1) | self.reg.f.carry as u8;

        self.reg.clear_flag();
        self.reg.f.zero = new == 0;
        self.reg.f.carry = flag_c == 1;
        new
    }

    fn rr(&mut self, value: u8) -> u8 {
        let flag_c = value & 0x01;
        let new = (value >> 1) | (self.reg.f.carry as u8) << 7;

        self.reg.clear_flag();
        self.reg.f.zero = new == 0;
        self.reg.f.carry = flag_c == 1;
        new
    }

    fn rlc(&mut self, value: u8) -> u8 {
        let flag_c = (value & 0x80) >> 7;
        let new = (value << 1) | flag_c;

        self.reg.clear_flag();
        self.reg.f.zero = new == 0;
        self.reg.f.carry = flag_c == 1;
        new
    }

    fn rrc(&mut self, value: u8) -> u8 {
        let flag_c = value & 0x01;
        let new = (value >> 1) | (flag_c << 7);

        self.reg.clear_flag();
        self.reg.f.zero = new == 0;
        self.reg.f.carry = flag_c == 1;
        new
    }

    fn sla(&mut self, value: u8) -> u8 {
        let flag_c = (value & 0x80) >> 7;
        let new = value << 1;

        self.reg.clear_flag();
        self.reg.f.zero = new == 0;
        self.reg.f.carry = flag_c == 1;
        new
    }

    fn sra(&mut self, value: u8) -> u8 {
        let flag_c = value & 0x01;
        let new = (value & 0x80) | (value >> 1);

        self.reg.clear_flag();
        self.reg.f.zero = new == 0;
        self.reg.f.carry = flag_c == 1;
        new
    }

    fn srl(&mut self, value: u8) -> u8 {
        let flag_c = value & 0x01;
        let new = value >> 1;

        self.reg.clear_flag();
        self.reg.f.zero = new == 0;
        self.reg.f.carry = flag_c == 1;
        new
    }

    fn swap(&mut self, value: u8) -> u8 {
        let new = ((value & 0xf0) >> 4) | ((value & 0x0f) << 4);

        self.reg.clear_flag();
        self.reg.f.zero = new == 0;
        new
    }

    fn bit(&mut self, n: u8, value: u8) -> u8 {
        self.reg.f.zero = (0b01 << n) & value == 0;
        self.reg.f.substract = false;
        self.reg.f.half_carry = true;
        value
    }

    fn res(&mut self, n: u8, value: u8) -> u8 {
        value & !(0b01 << n)
    }

    fn set(&mut self, n: u8, value: u8) -> u8 {
        value | (0b01 << n)
    }

    fn add_hl(&mut self, right: u16) {
        // Note: half_carry, byte -> 3 to 4 bit, word -> 7 -> 8 bit
        let (new, overflow) = self.reg.hl.overflowing_add(right);
        let half_carry = ((self.reg.hl & 0xfff) + (right & 0xfff)) > 0xfff;

        self.reg.f.substract = false;
        self.reg.f.half_carry = half_carry;
        self.reg.f.carry = overflow;
        self.reg.hl = new;
    }

    fn add_sp(&mut self, signed: u8) {
        let old = self.reg.sp;
        let new = add_relative(old, signed);

        self.reg.sp = new;
        self.reg.clear_flag();
        self.reg.f.half_carry = (new & 0x0f) < (old & 0x0f);
        self.reg.f.carry = (new & 0xff) < (old & 0xff);
    }

    // see:
    // https://stackoverflow.com/questions/45227884/z80-daa-implementation-and-blarggs-test-rom-issues
    fn daa(&mut self) {
        let mut new = self.reg.a as u16;

        if !self.reg.f.substract {
            if self.reg.f.half_carry || ((new & 0x0f) > 9) {
                new += 0x06;
            }
            if self.reg.f.carry || new > 0x9f {
                new += 0x60;
            }
        } else {
            if self.reg.f.half_carry {
                new = new.wrapping_sub(0x06);
                if !self.reg.f.carry {
                    new &= 0xff;
                }
            }
            if self.reg.f.carry {
                new = new.wrapping_sub(0x60);
            }
        }

        self.reg.a = new as u8;
        self.reg.f.zero = self.reg.a == 0;
        self.reg.f.half_carry = false;
        self.reg.f.carry = self.reg.f.carry || (new & 0x100 != 0);
    }

    fn cpl(&mut self) {
        self.reg.a = !self.reg.a;
        self.reg.f.substract = true;
        self.reg.f.half_carry = true;
    }

    fn ccf(&mut self) {
        let carry = !self.reg.f.carry;
        self.reg.f.substract = false;
        self.reg.f.half_carry = false;
        self.reg.f.carry = carry;
    }

    fn scf(&mut self) {
        self.reg.f.substract = false;
        self.reg.f.half_carry = false;
        self.reg.f.carry = true;
    }
}

fn add_relative(l: u16, r: u8) -> u16 {
    // sign extended cast
    l.wrapping_add(r as i8 as i16 as u16)
}
