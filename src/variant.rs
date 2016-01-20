// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module provides enums that wrap the various decoders and encoders.
//! The purpose is to make `Decoder` and `Encoder` `Sized` by writing the
//! dispatch explicitly for a finite set of specialized decoders and encoders.
//! Unfortunately, this means the compiler doesn't generate the dispatch code
//! and it has to be written here instead.
//!
//! The purpose of making `Decoder` and `Encoder` `Sized` is to allow stack
//! allocation in Rust code, including the convenience methods on `Encoding`.

use big5::*;
use super::*;

pub enum VariantDecoder {
    Big5(Big5Decoder),
}

impl VariantDecoder {
    pub fn reset(&mut self) {
        match self {
            &mut VariantDecoder::Big5(ref mut d) => {
                d.reset();
            }
        }
    }

    pub fn max_utf16_buffer_length(&self, byte_length: usize) -> usize {
        match self {
            &VariantDecoder::Big5(ref d) => d.max_utf16_buffer_length(byte_length),
        }
    }

    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        match self {
            &VariantDecoder::Big5(ref d) => d.max_utf8_buffer_length(byte_length),
        }
    }

    pub fn max_utf8_buffer_length_with_replacement(&self, byte_length: usize) -> usize {
        match self {
            &VariantDecoder::Big5(ref d) => d.max_utf8_buffer_length_with_replacement(byte_length),
        }
    }

    pub fn decode_to_utf16(&mut self,
                           src: &[u8],
                           dst: &mut [u16],
                           last: bool)
                           -> (DecoderResult, usize, usize) {
        match self {
            &mut VariantDecoder::Big5(ref mut d) => d.decode_to_utf16(src, dst, last),
        }
    }

    pub fn decode_to_utf8(&mut self,
                          src: &[u8],
                          dst: &mut [u8],
                          last: bool)
                          -> (DecoderResult, usize, usize) {
        match self {
            &mut VariantDecoder::Big5(ref mut d) => d.decode_to_utf8(src, dst, last),
        }
    }
}

pub enum VariantEncoder {
    Big5(Big5Encoder),
}

impl VariantEncoder {
    pub fn reset(&mut self) {}

    pub fn max_buffer_length_from_utf16(&self, u16_length: usize) -> usize {
        match self {
            &VariantEncoder::Big5(ref e) => e.max_buffer_length_from_utf16(u16_length),
        }
    }

    pub fn max_buffer_length_from_utf8(&self, byte_length: usize) -> usize {
        match self {
            &VariantEncoder::Big5(ref e) => e.max_buffer_length_from_utf8(byte_length),
        }
    }

    pub fn max_buffer_length_from_utf16_with_replacement(&self, u16_length: usize) -> usize {
        match self {
            &VariantEncoder::Big5(ref e) => {
                e.max_buffer_length_from_utf16_with_replacement(u16_length)
            }
        }
    }

    pub fn max_buffer_length_from_utf8_with_replacement(&self, byte_length: usize) -> usize {
        match self {
            &VariantEncoder::Big5(ref e) => {
                e.max_buffer_length_from_utf8_with_replacement(byte_length)
            }
        }
    }

    pub fn encode_from_utf16(&mut self,
                             src: &[u16],
                             dst: &mut [u8],
                             last: bool)
                             -> (EncoderResult, usize, usize) {
        match self {
            &mut VariantEncoder::Big5(ref mut e) => e.encode_from_utf16(src, dst, last),
        }
    }

    pub fn encode_from_utf8(&mut self,
                            src: &str,
                            dst: &mut [u8],
                            last: bool)
                            -> (EncoderResult, usize, usize) {
        match self {
            &mut VariantEncoder::Big5(ref mut e) => e.encode_from_utf8(src, dst, last),
        }
    }
}
