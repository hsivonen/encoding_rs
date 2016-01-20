// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::*;

const INPUT_EMPTY: u32 = 0xFFFFFFFF;

const OUTPUT_FULL: u32 = 0xFFFFFFFE;

impl WithReplacementResult {
    fn as_u32(&self) -> u32 {
        match self {
            &WithReplacementResult::InputEmpty => INPUT_EMPTY,
            &WithReplacementResult::OutputFull => OUTPUT_FULL,
        }
    }
}

impl DecoderResult {
    fn as_u32(&self) -> u32 {
        match self {
            &DecoderResult::InputEmpty => INPUT_EMPTY,
            &DecoderResult::OutputFull => OUTPUT_FULL,
            &DecoderResult::Malformed(num) => num as u32,
        }
    }
}

impl EncoderResult {
    fn as_u32(&self) -> u32 {
        match self {
            &EncoderResult::InputEmpty => INPUT_EMPTY,
            &EncoderResult::OutputFull => OUTPUT_FULL,
            &EncoderResult::Unmappable(c) => c as u32,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn decoder_decode_to_utf16(decoder: &mut Decoder,
                                                 src: *const u8,
                                                 src_len: *mut usize,
                                                 dst: *mut u16,
                                                 dst_len: *mut usize,
                                                 last: bool)
                                                 -> u32 {
    let src_slice = ::std::slice::from_raw_parts(src, *src_len);
    let dst_slice = ::std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written) = decoder.decode_to_utf16(src_slice, dst_slice, last);
    *src_len = read;
    *dst_len = written;
    result.as_u32()
}

#[no_mangle]
pub unsafe extern "C" fn decoder_decode_to_utf8(decoder: &mut Decoder,
                                                src: *const u8,
                                                src_len: *mut usize,
                                                dst: *mut u8,
                                                dst_len: *mut usize,
                                                last: bool)
                                                -> u32 {
    let src_slice = ::std::slice::from_raw_parts(src, *src_len);
    let dst_slice = ::std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written) = decoder.decode_to_utf8(src_slice, dst_slice, last);
    *src_len = read;
    *dst_len = written;
    result.as_u32()
}

#[no_mangle]
pub unsafe extern "C" fn decoder_decode_to_utf16_with_replacement(decoder: &mut Decoder,
                                                                  src: *const u8,
                                                                  src_len: *mut usize,
                                                                  dst: *mut u16,
                                                                  dst_len: *mut usize,
                                                                  last: bool,
                                                                  had_replacements: *mut bool)
                                                                  -> u32 {
    let src_slice = ::std::slice::from_raw_parts(src, *src_len);
    let dst_slice = ::std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written, replaced) = decoder.decode_to_utf16_with_replacement(src_slice,
                                                                                     dst_slice,
                                                                                     last);
    *src_len = read;
    *dst_len = written;
    *had_replacements = replaced;
    result.as_u32()
}

#[no_mangle]
pub unsafe extern "C" fn decoder_decode_to_utf8_with_replacement(decoder: &mut Decoder,
                                                                 src: *const u8,
                                                                 src_len: *mut usize,
                                                                 dst: *mut u8,
                                                                 dst_len: *mut usize,
                                                                 last: bool,
                                                                 had_replacements: *mut bool)
                                                                 -> u32 {
    let src_slice = ::std::slice::from_raw_parts(src, *src_len);
    let dst_slice = ::std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written, replaced) = decoder.decode_to_utf8_with_replacement(src_slice,
                                                                                    dst_slice,
                                                                                    last);
    *src_len = read;
    *dst_len = written;
    *had_replacements = replaced;
    result.as_u32()
}

#[no_mangle]
pub extern "C" fn decoder_max_utf16_length(decoder: &Decoder, byte_length: usize) -> usize {
    decoder.max_utf16_buffer_length(byte_length)
}

#[no_mangle]
pub extern "C" fn decoder_max_utf8_length(decoder: &Decoder, byte_length: usize) -> usize {
    decoder.max_utf8_buffer_length(byte_length)
}

#[no_mangle]
pub extern "C" fn decoder_max_utf8_length_with_replacement(decoder: &Decoder,
                                                           byte_length: usize)
                                                           -> usize {
    decoder.max_utf8_buffer_length_with_replacement(byte_length)
}
