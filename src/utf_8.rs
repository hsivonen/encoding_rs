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

pub struct Utf8Decoder {
    code_point: u32,
    bytes_seen: usize, // 1, 2 or 3: counts continuations only
    bytes_needed: usize, // 1, 2 or 3: counts continuations only
    lower_boundary: u8,
    upper_boundary: u8,
}

impl Utf8Decoder {
    pub fn new() -> VariantDecoder {
        VariantDecoder::Utf8(Utf8Decoder {
            code_point: 0,
            bytes_seen: 0,
            bytes_needed: 0,
            lower_boundary: 0x80u8,
            upper_boundary: 0xBFu8,
        })
    }

    pub fn max_utf16_buffer_length(&self, byte_length: usize) -> usize {
        byte_length + 1
    }

    pub fn max_utf8_buffer_length_without_replacement(&self, byte_length: usize) -> usize {
        byte_length + 3
    }

    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        byte_length * 3 + 3
    }

    decoder_functions!({},
                       {
                           if self.bytes_needed != 0 {
                               let bad_bytes = (self.bytes_seen + 1) as u8;
                               self.code_point = 0;
                               self.bytes_needed = 0;
                               self.bytes_seen = 0;
                               return (DecoderResult::Malformed(bad_bytes, 0),
                                       src_consumed,
                                       dest.written());
                           }
                       },
                       {
                           if self.bytes_needed == 0 {
                               if b < 0x80u8 {
                                   // XXX optimize ASCII
                                   destination_handle.write_ascii(b);
                                   continue;
                               }
                               if b < 0xC2u8 {
                                   return (DecoderResult::Malformed(1, 0),
                                           unread_handle.consumed(),
                                           destination_handle.written());
                               }
                               if b < 0xE0u8 {
                                   self.bytes_needed = 1;
                                   self.code_point = b as u32 & 0x1F;
                                   continue;
                               }
                               if b < 0xF0u8 {
                                   if b == 0xE0u8 {
                                       self.lower_boundary = 0xA0u8;
                                   } else if b == 0xEDu8 {
                                       self.upper_boundary = 0x9Fu8;
                                   }
                                   self.bytes_needed = 2;
                                   self.code_point = b as u32 & 0xF;
                                   continue;
                               }
                               if b < 0xF5u8 {
                                   if b == 0xF0u8 {
                                       self.lower_boundary = 0x90u8;
                                   } else if b == 0xF4u8 {
                                       self.upper_boundary = 0x8Fu8;
                                   }
                                   self.bytes_needed = 3;
                                   self.code_point = b as u32 & 0x7;
                                   continue;
                               }
                               return (DecoderResult::Malformed(1, 0),
                                       unread_handle.consumed(),
                                       destination_handle.written());
                           }
                           // self.bytes_needed != 0
                           if !(b >= self.lower_boundary && b <= self.upper_boundary) {
                               let bad_bytes = (self.bytes_seen + 1) as u8;
                               self.code_point = 0;
                               self.bytes_needed = 0;
                               self.bytes_seen = 0;
                               self.lower_boundary = 0x80u8;
                               self.upper_boundary = 0xBFu8;
                               return (DecoderResult::Malformed(bad_bytes, 0),
                                       unread_handle.unread(),
                                       destination_handle.written());
                           }
                           self.lower_boundary = 0x80u8;
                           self.upper_boundary = 0xBFu8;
                           self.code_point = (self.code_point << 6) | (b as u32 & 0x3F);
                           self.bytes_seen += 1;
                           if self.bytes_seen != self.bytes_needed {
                               continue;
                           }
                           if self.bytes_needed == 3 {
                               destination_handle.write_astral(self.code_point);
                           } else {
                               destination_handle.write_bmp_excl_ascii(self.code_point as u16);
                           }
                           self.code_point = 0;
                           self.bytes_needed = 0;
                           self.bytes_seen = 0;
                           continue;
                       },
                       self,
                       src_consumed,
                       dest,
                       b,
                       destination_handle,
                       unread_handle,
                       check_space_astral);
}

pub struct Utf8Encoder;

impl Utf8Encoder {
    pub fn new(encoding: &'static Encoding) -> Encoder {
        Encoder::new(encoding, VariantEncoder::Utf8(Utf8Encoder))
    }

    pub fn max_buffer_length_from_utf16_without_replacement(&self, u16_length: usize) -> usize {
        3 * u16_length
    }

    pub fn max_buffer_length_from_utf8_without_replacement(&self, byte_length: usize) -> usize {
        byte_length
    }

    pub fn encode_from_utf16_raw(&mut self,
                                 src: &[u16],
                                 dst: &mut [u8],
                                 _last: bool)
                                 -> (EncoderResult, usize, usize) {
        let mut source = Utf16Source::new(src);
        let mut dest = Utf8Destination::new(dst);
        loop {
            match source.check_available() {
                Space::Full(src_consumed) => {
                    return (EncoderResult::InputEmpty, src_consumed, dest.written());
                }
                Space::Available(source_handle) => {
                    match dest.check_space_astral() {
                        Space::Full(dst_written) => {
                            return (EncoderResult::OutputFull,
                                    source_handle.consumed(),
                                    dst_written);
                        }
                        Space::Available(destination_handle) => {
                            let (c, _) = source_handle.read();
                            // Start non-boilerplate
                            destination_handle.write_char(c);
                            // End non-boilerplate
                        }
                    }
                }
            }
        }
    }

    pub fn encode_from_utf8_raw(&mut self,
                                src: &str,
                                dst: &mut [u8],
                                _last: bool)
                                -> (EncoderResult, usize, usize) {
        let mut to_write = src.len();
        if to_write <= dst.len() {
            unsafe {
                ::std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), to_write);
            }
            return (EncoderResult::InputEmpty, to_write, to_write);
        }
        to_write = dst.len();
        // Move back until we find a UTF-8 sequence boundary.
        let bytes = src.as_bytes();
        while (bytes[to_write] & 0xC0) == 0x80 {
            to_write -= 1;
        }
        unsafe {
            ::std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), to_write);
        }
        return (EncoderResult::OutputFull, to_write, to_write);
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::*;

    fn decode_utf8_to_utf16(bytes: &[u8], expect: &[u16]) {
        decode_to_utf16_without_replacement(UTF_8, bytes, expect);
    }

    fn decode_utf8_to_utf8(bytes: &[u8], expect: &str) {
        decode_to_utf8_without_replacement(UTF_8, bytes, expect);
    }

    fn decode_valid_utf8(string: &str) {
        decode_utf8_to_utf8(string.as_bytes(), string);
    }

    fn encode_utf8_from_utf16(string: &[u16], expect: &[u8]) {
        encode_from_utf16_without_replacement(UTF_8, string, expect);
    }

    fn encode_utf8_from_utf8(string: &str, expect: &[u8]) {
        encode_from_utf8_without_replacement(UTF_8, string, expect);
    }

    #[test]
    fn test_utf8_decode() {
        decode_valid_utf8("ab");
        decode_valid_utf8("a\u{E4}b");
        decode_valid_utf8("a\u{2603}b");
        decode_valid_utf8("a\u{1F4A9}b");
        decode_utf8_to_utf8(b"a\xC3b", "a\u{FFFD}b");
        decode_utf8_to_utf8(b"a\xC3", "a\u{FFFD}");
        decode_utf8_to_utf8(b"a\xE2\x98b", "a\u{FFFD}b");
        decode_utf8_to_utf8(b"a\xE2\x98", "a\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x9F\x92b", "a\u{FFFD}b");
        decode_utf8_to_utf8(b"a\xF0\x9F\x92", "a\u{FFFD}");
        decode_utf8_to_utf8(b"a\x80b", "a\u{FFFD}b");
        decode_utf8_to_utf8(b"a\x80", "a\u{FFFD}");
        decode_utf8_to_utf8(b"a\xBFb", "a\u{FFFD}b");
        decode_utf8_to_utf8(b"a\xBF", "a\u{FFFD}");
        decode_utf8_to_utf8(b"a\x80\x80b", "a\u{FFFD}\u{FFFD}b");
        decode_utf8_to_utf8(b"a\x80\x80", "a\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xBF\xBFb", "a\u{FFFD}\u{FFFD}b");
        decode_utf8_to_utf8(b"a\xBF\xBF", "a\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xC3\xA4\x80b", "a\u{E4}\u{FFFD}b");
        decode_utf8_to_utf8(b"a\xC3\xA4\x80", "a\u{E4}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xC3\xA4\xBFb", "a\u{E4}\u{FFFD}b");
        decode_utf8_to_utf8(b"a\xC3\xA4\xBF", "a\u{E4}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xE2\x98\x83\x80b", "a\u{2603}\u{FFFD}b");
        decode_utf8_to_utf8(b"a\xE2\x98\x83\x80", "a\u{2603}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xE2\x98\x83\xBFb", "a\u{2603}\u{FFFD}b");
        decode_utf8_to_utf8(b"a\xE2\x98\x83\xBF", "a\u{2603}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x9F\x92\xA9\x80b", "a\u{1F4A9}\u{FFFD}b");
        decode_utf8_to_utf8(b"a\xF0\x9F\x92\xA9\x80", "a\u{1F4A9}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x9F\x92\xA9\xBFb", "a\u{1F4A9}\u{FFFD}b");
        decode_utf8_to_utf8(b"a\xF0\x9F\x92\xA9\xBF", "a\u{1F4A9}\u{FFFD}");
    }

}
