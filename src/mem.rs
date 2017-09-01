// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub fn is_ascii(buffer: &[u8]) -> bool {
    true
}

pub fn is_basic_latin(buffer: &[u16]) -> bool {
    true
}

pub fn is_utf8_latin1(buffer: &str) -> bool {
    true
}

pub fn is_utf16_latin1(buffer: &[u16]) -> bool {
    true
}

pub fn convert_utf8_to_utf16(src: &[u8], dst: &mut [u16]) -> usize {
    0
}

pub fn convert_str_to_utf16(src: &str, dst: &mut [u16]) -> usize {
    0
}

pub fn convert_utf16_to_utf8(src: &[u16], dst: &mut [u8]) -> usize {
    0
}

pub fn convert_utf16_to_str(src: &[u16], dst: &mut str) -> usize {
    0
}

pub fn convert_latin1_to_utf16(src: &[u8], dst: &mut [u16]) -> usize {
    0
}

pub fn convert_latin1_to_utf8(src: &[u8], dst: &mut [u8]) -> usize {
    0
}

pub fn convert_latin1_to_str(src: &[u8], dst: &mut str) -> usize {
    0
}

pub fn convert_utf8_to_latin1_lossy(src: &[u8], dst: &mut [u8]) -> usize {
    0
}

pub fn convert_utf16_to_latin1_lossy(src: &[u16], dst: &mut [u8]) -> usize {
    0
}

pub fn utf16_valid_up_to(buffer: &[u16]) -> usize {
    0
}

pub fn ensure_utf16_validity(buffer: &mut[u16]) {

}

pub fn copy_ascii_to_ascii(src: &[u8], dst: &mut [u8]) -> usize {
    0
}

pub fn copy_ascii_to_basic_latin(src: &[u8], dst: &mut [u16]) -> usize {
    0
}

pub fn copy_basic_latin_to_ascii(src: &[u16], dst: &mut [u8]) -> usize {
    0
}

