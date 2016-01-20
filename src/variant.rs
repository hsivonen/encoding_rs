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
