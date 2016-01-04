// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub struct BmpHandle<'a, 'b> where 'b: 'a {
    dest: &'a mut Destination<'b>,
}

impl<'a, 'b> BmpHandle<'a, 'b> where 'b: 'a {
    #[inline(always)]
    fn new(dst: &'a mut Destination<'b>) -> BmpHandle<'a, 'b> {
        BmpHandle { dest: dst }
    }
    #[inline(always)]
    pub fn write_ascii(self, ascii: u8) {
        self.dest.write_ascii(ascii);
    }
    #[inline(always)]
    pub fn write_bmp_excl_ascii(self, bmp: u16) {
        self.dest.write_bmp_excl_ascii(bmp);
    }
}

pub struct AstralHandle<'a, 'b> where 'b: 'a {
    dest: &'a mut Destination<'b>,
}

impl<'a, 'b> AstralHandle<'a, 'b> where 'b: 'a {
    #[inline(always)]
    fn new(dst: &'a mut Destination<'b>) -> AstralHandle<'a, 'b> {
        AstralHandle { dest: dst }
    }
    #[inline(always)]
    pub fn write_char(self, c: char) {
        self.dest.write_char(c);
    }
    #[inline(always)]
    pub fn write_ascii(self, ascii: u8) {
        self.dest.write_ascii(ascii);
    }
    #[inline(always)]
    pub fn write_bmp_excl_ascii(self, bmp: u16) {
        self.dest.write_bmp_excl_ascii(bmp);
    }
    #[inline(always)]
    pub fn write_astral(self, astral: u32) {
        self.dest.write_astral(astral);
    }
}

pub struct Big5Handle<'a, 'b> where 'b: 'a {
    dest: &'a mut Destination<'b>,
}

impl<'a, 'b> Big5Handle<'a, 'b> where 'b: 'a {
    #[inline(always)]
    fn new(dst: &'a mut Destination<'b>) -> Big5Handle<'a, 'b> {
        Big5Handle { dest: dst }
    }
    #[inline(always)]
    pub fn write_char(self, c: char) {
        self.dest.write_char(c);
    }
    #[inline(always)]
    pub fn write_ascii(self, ascii: u8) {
        self.dest.write_ascii(ascii);
    }
    #[inline(always)]
    pub fn write_bmp_excl_ascii(self, bmp: u16) {
        self.dest.write_bmp_excl_ascii(bmp);
    }
    #[inline(always)]
    pub fn write_astral(self, astral: u32) {
        self.dest.write_astral(astral);
    }
    #[inline(always)]
    pub fn write_big5_combination(self, combined: u16, combining: u16) {
        self.write_big5_combination(combined, combining);
    }
}

pub struct Destination<'a> {
    slice: &'a mut [u16],
    pos: usize,
}

impl<'a> Destination<'a> {
    #[inline(always)]
    pub fn new(dst: &mut [u16]) -> Destination {
        Destination { slice: dst, pos: 0 }
    }
    #[inline(always)]
    pub fn check_space_bmp<'b>(&'b mut self) -> Option<BmpHandle<'b, 'a>> {
        if self.pos < self.slice.len() {
            Some(BmpHandle::new(self))
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn check_space_astral<'b>(&'b mut self) -> Option<AstralHandle<'b, 'a>> {
        if self.pos + 1 < self.slice.len() {
            Some(AstralHandle::new(self))
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn check_space_big5<'b>(&'b mut self) -> Option<Big5Handle<'b, 'a>> {
        if self.pos + 1 < self.slice.len() {
            Some(Big5Handle::new(self))
        } else {
            None
        }
    }
    #[inline(always)]
    fn write_char(&mut self, c: char) {
        if c <= '\u{7F}' {
            self.write_ascii(c as u8);
        } else if c <= '\u{FFFF}' {
            self.write_bmp_excl_ascii(c as u16);
        } else {
            self.write_astral(c as u32);
        }
    }
    #[inline(always)]
    fn write_ascii(&mut self, ascii: u8) {
        self.slice[self.pos] = ascii as u16;
        self.pos += 1;
    }
    #[inline(always)]
    fn write_bmp_excl_ascii(&mut self, bmp: u16) {
        self.slice[self.pos] = bmp;
        self.pos += 1;
    }
    #[inline(always)]
    fn write_astral(&mut self, astral: u32) {
        self.slice[self.pos] = (0xD7C0 + (astral >> 10)) as u16;
        self.slice[self.pos + 1] = (0xDC00 + (astral & 0x3FF)) as u16;
        self.pos += 2;
    }
    #[inline(always)]
    fn write_big5_combination(&mut self, combined: u16, combining: u16) {
        self.slice[self.pos] = combined;
        self.slice[self.pos + 1] = combining;
        self.pos += 2;
    }
}
