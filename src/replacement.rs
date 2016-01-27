// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use handles::*;
use variant::*;
use super::*;

pub struct ReplacementDecoder {
    emitted: bool,
}

impl ReplacementDecoder {
    pub fn new(encoding: &'static Encoding) -> Decoder {
        Decoder::new(encoding,
                     VariantDecoder::Replacement(ReplacementDecoder { emitted: false }))
    }

    pub fn reset(&mut self) {
        self.emitted = false;
    }

    pub fn max_utf16_buffer_length(&self, u16_length: usize) -> usize {
        1
    }

    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        1 // really zero, but that might surprise callers
    }

    pub fn max_utf8_buffer_length_with_replacement(&self, byte_length: usize) -> usize {
        3
    }

    fn decode(&mut self, src: &[u8], last: bool) -> (DecoderResult, usize, usize) {
        // Don't err if the input stream is empty. See
        // https://github.com/whatwg/encoding/issues/33
        if self.emitted || src.is_empty() {
            if last {
                // The API says the caller doesn't need to reset after
                // the decoder returns `InputEmpty` with `last` set to `true`.
                self.emitted = false;
            }
            (DecoderResult::InputEmpty, src.len(), 0)
        } else {
            self.emitted = true;
            (DecoderResult::Malformed(1u8), 1, 0)
        }
    }

    pub fn decode_to_utf16(&mut self,
                           src: &[u8],
                           _dst: &mut [u16],
                           last: bool)
                           -> (DecoderResult, usize, usize) {
        self.decode(src, last)
    }

    pub fn decode_to_utf8(&mut self,
                          src: &[u8],
                          _dst: &mut [u8],
                          last: bool)
                          -> (DecoderResult, usize, usize) {
        self.decode(src, last)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;

}
