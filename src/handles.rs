// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module provides structs that use lifetimes to couple bounds checking
//! and space availability checking and deteching those from actual slice
//! reading/writing.
//!
//! At present, the internals of the implementation are safe code, so the
//! bound checks currently also happen on read/write. Once this code works,
//! the plan is to replace the internals with unsafe code that omits the
//! bound check at the read/write time.

pub enum Space<T> {
    Available(T),
    Full(usize),
}

// Byte source

pub struct ByteSource<'a> {
    slice: &'a [u8],
    pos: usize,
}

impl<'a> ByteSource<'a> {
    #[inline(always)]
    pub fn new(src: &[u8]) -> ByteSource {
        ByteSource {
            slice: src,
            pos: 0,
        }
    }
    #[inline(always)]
    pub fn check_available<'b>(&'b mut self) -> Space<ByteReadHandle<'b, 'a>> {
        if self.pos < self.slice.len() {
            Space::Available(ByteReadHandle::new(self))
        } else {
            Space::Full(self.consumed())
        }
    }
    #[inline(always)]
    fn read(&mut self) -> u8 {
        let ret = self.slice[self.pos];
        self.pos += 1;
        ret
    }
    #[inline(always)]
    fn unread(&mut self) -> usize {
        self.pos -= 1;
        self.pos
    }
    #[inline(always)]
    fn consumed(&self) -> usize {
        self.pos
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
    #[inline(always)]
    pub fn consumed(&self) -> usize {
        self.source.consumed()
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
    pub fn unread(self) -> usize {
        self.source.unread()
    }
    #[inline(always)]
    pub fn consumed(&self) -> usize {
        self.source.consumed()
    }
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
    pub fn written(&self) -> usize {
        self.dest.written()
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
    pub fn written(&self) -> usize {
        self.dest.written()
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

pub struct Utf16Big5Handle<'a, 'b>
    where 'b: 'a
{
    dest: &'a mut Utf16Destination<'b>,
}

impl<'a, 'b> Utf16Big5Handle<'a, 'b> where 'b: 'a
{
    #[inline(always)]
    fn new(dst: &'a mut Utf16Destination<'b>) -> Utf16Big5Handle<'a, 'b> {
        Utf16Big5Handle { dest: dst }
    }
    #[inline(always)]
    pub fn written(&self) -> usize {
        self.dest.written()
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
        self.dest.write_big5_combination(combined, combining);
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
    pub fn check_space_bmp<'b>(&'b mut self) -> Space<Utf16BmpHandle<'b, 'a>> {
        if self.pos < self.slice.len() {
            Space::Available(Utf16BmpHandle::new(self))
        } else {
            Space::Full(self.written())
        }
    }
    #[inline(always)]
    pub fn check_space_astral<'b>(&'b mut self) -> Space<Utf16AstralHandle<'b, 'a>> {
        if self.pos + 1 < self.slice.len() {
            Space::Available(Utf16AstralHandle::new(self))
        } else {
            Space::Full(self.written())
        }
    }
    #[inline(always)]
    pub fn check_space_big5<'b>(&'b mut self) -> Space<Utf16Big5Handle<'b, 'a>> {
        if self.pos + 1 < self.slice.len() {
            Space::Available(Utf16Big5Handle::new(self))
        } else {
            Space::Full(self.written())
        }
    }
    #[inline(always)]
    pub fn written(&self) -> usize {
        self.pos
    }
    #[inline(always)]
    fn write_code_unit(&mut self, u: u16) {
        self.slice[self.pos] = u;
        self.pos += 1;
    }
    #[inline(always)]
    fn write_char(&mut self, c: char) {
        if c <= '\u{FFFF}' {
            self.write_code_unit(c as u16);
        } else {
            self.write_astral(c as u32);
        }
    }
    #[inline(always)]
    fn write_ascii(&mut self, ascii: u8) {
        debug_assert!(ascii < 0x80);
        self.write_code_unit(ascii as u16);
    }
    #[inline(always)]
    fn write_bmp_excl_ascii(&mut self, bmp: u16) {
        debug_assert!(bmp >= 0x80);
        self.write_code_unit(bmp);
    }
    #[inline(always)]
    fn write_astral(&mut self, astral: u32) {
        debug_assert!(astral > 0xFFFF);
        debug_assert!(astral <= 0x10FFFF);
        self.write_code_unit((0xD7C0 + (astral >> 10)) as u16);
        self.write_code_unit((0xDC00 + (astral & 0x3FF)) as u16);
    }
    #[inline(always)]
    fn write_big5_combination(&mut self, combined: u16, combining: u16) {
        self.write_bmp_excl_ascii(combined);
        self.write_bmp_excl_ascii(combining);
    }
}

// UTF-8 destination

pub struct Utf8BmpHandle<'a, 'b>
    where 'b: 'a
{
    dest: &'a mut Utf8Destination<'b>,
}

impl<'a, 'b> Utf8BmpHandle<'a, 'b> where 'b: 'a
{
    #[inline(always)]
    fn new(dst: &'a mut Utf8Destination<'b>) -> Utf8BmpHandle<'a, 'b> {
        Utf8BmpHandle { dest: dst }
    }
    #[inline(always)]
    pub fn written(&self) -> usize {
        self.dest.written()
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

pub struct Utf8AstralHandle<'a, 'b>
    where 'b: 'a
{
    dest: &'a mut Utf8Destination<'b>,
}

impl<'a, 'b> Utf8AstralHandle<'a, 'b> where 'b: 'a
{
    #[inline(always)]
    fn new(dst: &'a mut Utf8Destination<'b>) -> Utf8AstralHandle<'a, 'b> {
        Utf8AstralHandle { dest: dst }
    }
    #[inline(always)]
    pub fn written(&self) -> usize {
        self.dest.written()
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

pub struct Utf8Big5Handle<'a, 'b>
    where 'b: 'a
{
    dest: &'a mut Utf8Destination<'b>,
}

impl<'a, 'b> Utf8Big5Handle<'a, 'b> where 'b: 'a
{
    #[inline(always)]
    fn new(dst: &'a mut Utf8Destination<'b>) -> Utf8Big5Handle<'a, 'b> {
        Utf8Big5Handle { dest: dst }
    }
    #[inline(always)]
    pub fn written(&self) -> usize {
        self.dest.written()
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
        self.dest.write_big5_combination(combined, combining);
    }
}

pub struct Utf8Destination<'a> {
    slice: &'a mut [u8],
    pos: usize,
}

impl<'a> Utf8Destination<'a> {
    #[inline(always)]
    pub fn new(dst: &mut [u8]) -> Utf8Destination {
        Utf8Destination {
            slice: dst,
            pos: 0,
        }
    }
    #[inline(always)]
    pub fn check_space_bmp<'b>(&'b mut self) -> Space<Utf8BmpHandle<'b, 'a>> {
        if self.pos < self.slice.len() {
            Space::Available(Utf8BmpHandle::new(self))
        } else {
            Space::Full(self.written())
        }
    }
    #[inline(always)]
    pub fn check_space_astral<'b>(&'b mut self) -> Space<Utf8AstralHandle<'b, 'a>> {
        if self.pos + 1 < self.slice.len() {
            Space::Available(Utf8AstralHandle::new(self))
        } else {
            Space::Full(self.written())
        }
    }
    #[inline(always)]
    pub fn check_space_big5<'b>(&'b mut self) -> Space<Utf8Big5Handle<'b, 'a>> {
        if self.pos + 1 < self.slice.len() {
            Space::Available(Utf8Big5Handle::new(self))
        } else {
            Space::Full(self.written())
        }
    }
    #[inline(always)]
    pub fn written(&self) -> usize {
        self.pos
    }
    #[inline(always)]
    fn write_code_unit(&mut self, u: u8) {
        self.slice[self.pos] = u;
        self.pos += 1;
    }
    #[inline(always)]
    fn write_char(&mut self, c: char) {
        if c <= '\u{7F}' {
            self.write_ascii(c as u8);
        } else if c <= '\u{0800}' {
            self.write_mid_bmp(c as u16);
        } else if c <= '\u{FFFF}' {
            self.write_upper_bmp(c as u16);
        } else {
            self.write_astral(c as u32);
        }
    }
    #[inline(always)]
    fn write_ascii(&mut self, ascii: u8) {
        debug_assert!(ascii < 0x80);
        self.write_code_unit(ascii);
    }
    #[inline(always)]
    fn write_mid_bmp(&mut self, mid_bmp: u16) {
        debug_assert!(mid_bmp >= 0x80);
        debug_assert!(mid_bmp < 0x800);
        self.write_code_unit(((mid_bmp as u32 >> 6) | 0xC0u32) as u8);
        self.write_code_unit(((mid_bmp as u32 & 0x3Fu32) | 0x80u32) as u8);
    }
    #[inline(always)]
    fn write_upper_bmp(&mut self, upper_bmp: u16) {
        debug_assert!(upper_bmp >= 0x800);
        self.write_code_unit(((upper_bmp as u32 >> 12) | 0xE0u32) as u8);
        self.write_code_unit((((upper_bmp as u32 & 0xFC0u32) >> 6) | 0x80u32) as u8);
        self.write_code_unit(((upper_bmp as u32 & 0x3Fu32) | 0x80u32) as u8);
    }
    #[inline(always)]
    fn write_bmp_excl_ascii(&mut self, bmp: u16) {
        if bmp < 0x800u16 {
            self.write_mid_bmp(bmp);
        } else {
            self.write_upper_bmp(bmp);
        }
    }
    #[inline(always)]
    fn write_astral(&mut self, astral: u32) {
        debug_assert!(astral > 0xFFFF);
        debug_assert!(astral <= 0x10FFFF);
        self.write_code_unit(((astral >> 18) | 0xF0u32) as u8);
        self.write_code_unit((((astral & 0x3F000u32) >> 12) | 0x80u32) as u8);
        self.write_code_unit((((astral & 0xFC0u32) >> 6) | 0x80u32) as u8);
        self.write_code_unit(((astral & 0x3Fu32) | 0x80u32) as u8);
    }
    #[inline(always)]
    fn write_big5_combination(&mut self, combined: u16, combining: u16) {
        self.write_mid_bmp(combined);
        self.write_upper_bmp(combining);
    }
}
