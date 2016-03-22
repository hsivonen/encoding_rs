// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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

    pub fn max_utf16_buffer_length(&self, _u16_length: usize) -> usize {
        1
    }

    pub fn max_utf8_buffer_length(&self, _byte_length: usize) -> usize {
        1 // really zero, but that might surprise callers
    }

    pub fn max_utf8_buffer_length_with_replacement(&self, _byte_length: usize) -> usize {
        3
    }

    fn decode(&mut self, src: &[u8], last: bool) -> (DecoderResult, usize, usize) {
        // Don't err if the input stream is empty. See
        // https://github.com/whatwg/encoding/issues/33
        if self.emitted || src.is_empty() {
            (DecoderResult::InputEmpty, src.len(), 0)
        } else {
            // We don't need to check if output has enough space, because
            // everything is weird anyway if the caller of the `Encoder` API
            // passes an output buffer that violates the minimum size rules.
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
    use super::super::testing::*;
    use super::super::*;

    fn decode_replacement_to_utf16(bytes: &[u8], expect: &[u16]) {
        decode_to_utf16(REPLACEMENT, bytes, expect);
    }

    fn decode_replacement_to_utf8(bytes: &[u8], expect: &str) {
        decode_to_utf8(REPLACEMENT, bytes, expect);
    }

    #[test]
    fn test_replacement_decode() {
        decode_replacement_to_utf16(b"", &[]);
        decode_replacement_to_utf16(b"A", &[0xFFFDu16]);
        decode_replacement_to_utf16(b"AB", &[0xFFFDu16]);
        decode_replacement_to_utf8(b"", "");
        decode_replacement_to_utf8(b"A", "\u{FFFD}");
        decode_replacement_to_utf8(b"AB", "\u{FFFD}");
    }

    #[test]
    fn test_replacement_encode() {
        assert_eq!(REPLACEMENT.new_encoder().encoding(), UTF_8);
    }
}
