// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Byte source

pub struct ByteSource<'a> {
    slice: &'a mut [u8],
    pos: usize,
}

impl<'a> ByteSource<'a> {
    #[inline(always)]
    pub fn new(src: &mut [u8]) -> ByteSource {
        ByteSource {
            slice: src,
            pos: 0,
        }
    }
    #[inline(always)]
    pub fn check_available<'b>(&'b mut self) -> Option<ByteReadHandle<'b, 'a>> {
        if self.pos < self.slice.len() {
            Some(ByteReadHandle::new(self))
        } else {
            None
        }
    }
    #[inline(always)]
    fn read(&mut self) -> u8 {
        let ret = self.slice[self.pos];
        self.pos += 1;
        ret
    }
    #[inline(always)]
    fn unread(&mut self) {
        self.pos -= 1;
    }
}

pub struct ByteReadHandle<'a, 'b>
    where 'b: 'a
{
    source: &'a mut ByteSource<'b>,
}

impl<'a, 'b> ByteReadHandle<'a, 'b> where 'b: 'a
{
    #[inline(always)]
    fn new(src: &'a mut ByteSource<'b>) -> ByteReadHandle<'a, 'b> {
        ByteReadHandle { source: src }
    }
    #[inline(always)]
    pub fn read(self) -> (u8, ByteUnreadHandle<'a, 'b>) {
        let byte = self.source.read();
        let handle = ByteUnreadHandle::new(self.source);
        (byte, handle)
    }
}

pub struct ByteUnreadHandle<'a, 'b>
    where 'b: 'a
{
    source: &'a mut ByteSource<'b>,
}

impl<'a, 'b> ByteUnreadHandle<'a, 'b> where 'b: 'a
{
    #[inline(always)]
    fn new(src: &'a mut ByteSource<'b>) -> ByteUnreadHandle<'a, 'b> {
        ByteUnreadHandle { source: src }
    }
    #[inline(always)]
    pub fn unread(self) {}
}

// UTF-16 destination

pub struct Utf16BmpHandle<'a, 'b>
    where 'b: 'a
{
    dest: &'a mut Utf16Destination<'b>,
}

impl<'a, 'b> Utf16BmpHandle<'a, 'b> where 'b: 'a
{
    #[inline(always)]
    fn new(dst: &'a mut Utf16Destination<'b>) -> Utf16BmpHandle<'a, 'b> {
        Utf16BmpHandle { dest: dst }
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

pub struct Utf16AstralHandle<'a, 'b>
    where 'b: 'a
{
    dest: &'a mut Utf16Destination<'b>,
}

impl<'a, 'b> Utf16AstralHandle<'a, 'b> where 'b: 'a
{
    #[inline(always)]
    fn new(dst: &'a mut Utf16Destination<'b>) -> Utf16AstralHandle<'a, 'b> {
        Utf16AstralHandle { dest: dst }
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

pub struct Big5Handle<'a, 'b>
    where 'b: 'a
{
    dest: &'a mut Utf16Destination<'b>,
}

impl<'a, 'b> Big5Handle<'a, 'b> where 'b: 'a
{
    #[inline(always)]
    fn new(dst: &'a mut Utf16Destination<'b>) -> Big5Handle<'a, 'b> {
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

pub struct Utf16Destination<'a> {
    slice: &'a mut [u16],
    pos: usize,
}

impl<'a> Utf16Destination<'a> {
    #[inline(always)]
    pub fn new(dst: &mut [u16]) -> Utf16Destination {
        Utf16Destination {
            slice: dst,
            pos: 0,
        }
    }
    #[inline(always)]
    pub fn check_space_bmp<'b>(&'b mut self) -> Option<Utf16BmpHandle<'b, 'a>> {
        if self.pos < self.slice.len() {
            Some(Utf16BmpHandle::new(self))
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn check_space_astral<'b>(&'b mut self) -> Option<Utf16AstralHandle<'b, 'a>> {
        if self.pos + 1 < self.slice.len() {
            Some(Utf16AstralHandle::new(self))
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
