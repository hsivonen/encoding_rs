// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use handles::*;
use data::*;
use variant::*;
use super::*;

pub struct ShiftJisDecoder {
    lead: Option<u8>,
}

impl ShiftJisDecoder {
    pub fn new() -> VariantDecoder {
        VariantDecoder::ShiftJis(ShiftJisDecoder { lead: None })
    }

    fn plus_one_if_lead(&self, byte_length: usize) -> usize {
        byte_length +
        match self.lead {
            None => 0,
            Some(_) => 1,
        }
    }

    pub fn max_utf16_buffer_length(&self, byte_length: usize) -> usize {
        self.plus_one_if_lead(byte_length)
    }

    pub fn max_utf8_buffer_length_without_replacement(&self, byte_length: usize) -> usize {
        // worst case: 2 to 3
        let len = self.plus_one_if_lead(byte_length);
        len + (len + 1) / 2
    }

    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        self.plus_one_if_lead(byte_length) * 3
    }

    ascii_compatible_two_byte_decoder_functions!({
    // If lead is between 0x81 and 0x9F, inclusive,
    // subtract offset 0x81. Else if lead is
    // between 0xE0 and 0xFC, inclusive, subtract
    // offset 0xC1. Else if lead is between
    // 0xA1 and 0xDF, inclusive, map to half-width
    // Katakana. Else if lead is 0x80, pass through.
                                                     let mut non_ascii_minus_offset =
                                                         non_ascii.wrapping_sub(0x81);
                                                     if non_ascii_minus_offset > (0x9F - 0x81) {
                                                         let non_ascii_minus_range_start = non_ascii.wrapping_sub(0xE0);
                                                         if non_ascii_minus_range_start > (0xFC - 0xE0) {
                                                             let non_ascii_minus_half_with_katakana_start = non_ascii.wrapping_sub(0xA1);
                                                             if non_ascii_minus_half_with_katakana_start > (0xDF - 0xA1) {
                                                                 if non_ascii == 0x80 {
                                                                     handle.write_mid_bmp(0x80);
    // Not caring about optimizing subsequent non-ASCII
                                                                     continue 'outermost;
                                                                 }
                                                                 return (DecoderResult::Malformed(1, 0),
                                                                         source.consumed(),
                                                                         handle.written());
                                                             }
                                                             handle.write_upper_bmp(0xFF61 + non_ascii_minus_half_with_katakana_start as u16);
                                                             // Not caring about optimizing subsequent non-ASCII
                                                             continue 'outermost;
                                                         }
                                                         non_ascii_minus_offset = non_ascii - 0xC1;
                                                     }
                                                     non_ascii_minus_offset
                                                 },
                                                 {
    // If trail is between 0x40 and 0x7E, inclusive,
    // subtract offset 0x40. Else if trail is
    // between 0x80 and 0xFC, inclusive, subtract
    // offset 0x41.
                                                     let mut trail_minus_offset =
                                                         byte.wrapping_sub(0x40);
                                                     if trail_minus_offset > (0x7E - 0x40) {
                                                         let trail_minus_range_start =
                                                             byte.wrapping_sub(0x80);
                                                         if trail_minus_range_start > (0xFC - 0x80) {
                                                             if byte < 0x80 {
                                                                 return (DecoderResult::Malformed(1, 0),
                                                                         unread_handle_trail.unread(),
                                                                         handle.written());
                                                             }
                                                             return (DecoderResult::Malformed(2, 0),
                                                                     unread_handle_trail.consumed(),
                                                                     handle.written());
                                                         }
                                                         trail_minus_offset = byte - 0x41;
                                                     }
                                                     let pointer = lead_minus_offset as usize *
                                                                   188usize +
                                                                   trail_minus_offset as usize;
                                                     if pointer >= 8836 && pointer <= 10715 {
                                                         handle.write_upper_bmp((0xE000 - 8836 + pointer) as u16)
                                                     } else {
                                                         let bmp = jis0208_decode(pointer);
                                                         if bmp == 0 {
                                                             if byte < 0x80 {
                                                                 return (DecoderResult::Malformed(1, 0),
                                                                         unread_handle_trail.unread(),
                                                                         handle.written());
                                                             }
                                                             return (DecoderResult::Malformed(2, 0),
                                                                     unread_handle_trail.consumed(),
                                                                     handle.written());
                                                         }
                                                         handle.write_bmp_excl_ascii(bmp)
                                                     }
                                                 },
                                                 self,
                                                 non_ascii,
                                                 byte,
                                                 lead_minus_offset,
                                                 unread_handle_trail,
                                                 source,
                                                 handle,
                                                 'outermost,
                                                 copy_ascii_from_check_space_bmp,
                                                 check_space_bmp,
                                                 false);
}

pub struct ShiftJisEncoder;

impl ShiftJisEncoder {
    pub fn new(encoding: &'static Encoding) -> Encoder {
        Encoder::new(encoding, VariantEncoder::ShiftJis(ShiftJisEncoder))
    }

    pub fn max_buffer_length_from_utf16_without_replacement(&self, u16_length: usize) -> usize {
        u16_length * 2
    }

    pub fn max_buffer_length_from_utf8_without_replacement(&self, byte_length: usize) -> usize {
        byte_length
    }

    ascii_compatible_bmp_encoder_functions!({
                                                if bmp == 0x80 {
                                                    handle.write_one(0x80u8)
                                                } else if bmp == 0xA5 {
                                                    handle.write_one(0x5Cu8)
                                                } else if bmp == 0x203E {
                                                    handle.write_one(0x7Eu8)
                                                } else if bmp >= 0xFF61 && bmp <= 0xFF9F {
                                                    handle.write_one((bmp - (0xFF61 - 0xA1)) as u8)
                                                } else if bmp == 0x2212 {
                                                    handle.write_two(0x81u8, 0x7Cu8)
                                                } else {
                                                    let pointer = shift_jis_encode(bmp);
                                                    if pointer == usize::max_value() {
                                                        return (EncoderResult::unmappable_from_bmp(bmp),
                                                                source.consumed(),
                                                                handle.written());
                                                    }
                                                    let lead = pointer / 188;
                                                    let lead_offset = if lead < 0x1F {
                                                        0x81usize
                                                    } else {
                                                        0xC1usize
                                                    };
                                                    let trail = pointer % 188;
                                                    let trail_offset = if trail < 0x3F {
                                                        0x40usize
                                                    } else {
                                                        0x41usize
                                                    };
                                                    handle.write_two((lead + lead_offset) as u8,
                                                                     (trail + trail_offset) as u8)
                                                }
                                            },
                                            bmp,
                                            self,
                                            source,
                                            handle,
                                            copy_ascii_to_check_space_two,
                                            check_space_two,
                                            false);
}

// Any copyright to the test code below this comment is dedicated to the
// Public Domain. http://creativecommons.org/publicdomain/zero/1.0/

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::*;

    fn decode_shift_jis(bytes: &[u8], expect: &str) {
        decode(SHIFT_JIS, bytes, expect);
    }

    fn encode_shift_jis(string: &str, expect: &[u8]) {
        encode(SHIFT_JIS, string, expect);
    }

    #[test]
    fn test_shift_jis_decode() {
        // Empty
        decode_shift_jis(b"", &"");

        // ASCII
        decode_shift_jis(b"\x61\x62", "\u{0061}\u{0062}");

        // Half-width
        decode_shift_jis(b"\xA1", "\u{FF61}");
        decode_shift_jis(b"\xDF", "\u{FF9F}");
        decode_shift_jis(b"\xA0", "\u{FFFD}");
        decode_shift_jis(b"\xE0", "\u{FFFD}");
        decode_shift_jis(b"\xA0+", "\u{FFFD}+");
        decode_shift_jis(b"\xE0+", "\u{FFFD}+");

        // EUDC
        decode_shift_jis(b"\xF0\x40", "\u{E000}");
        decode_shift_jis(b"\xF9\xFC", "\u{E757}");
        decode_shift_jis(b"\xEF\xFC", "\u{FFFD}");
        decode_shift_jis(b"\xFA\x40", "\u{2170}");

        // JIS 0208
        decode_shift_jis(b"\x81\x40", "\u{3000}");
        decode_shift_jis(b"\x81\x3F", "\u{FFFD}?");
        decode_shift_jis(b"\xEE\xFC", "\u{FF02}");
        decode_shift_jis(b"\xEE\xFD", "\u{FFFD}");
        decode_shift_jis(b"\xFA\x40", "\u{2170}");
        decode_shift_jis(b"\xFA\x3F", "\u{FFFD}?");
        decode_shift_jis(b"\xFC\x4B", "\u{9ED1}");
        decode_shift_jis(b"\xFC\x4C", "\u{FFFD}L");
        //
    }

    #[test]
    fn test_shift_jis_encode() {
        // Empty
        encode_shift_jis("", b"");

        // ASCII
        encode_shift_jis("\u{0061}\u{0062}", b"\x61\x62");

        // Exceptional code points
        encode_shift_jis("\u{0080}", b"\x80");
        encode_shift_jis("\u{00A5}", b"\x5C");
        encode_shift_jis("\u{203E}", b"\x7E");
        encode_shift_jis("\u{2212}", b"\x81\x7C");

        // Half-width
        encode_shift_jis("\u{FF61}", b"\xA1");
        encode_shift_jis("\u{FF9F}", b"\xDF");

        // EUDC
        encode_shift_jis("\u{E000}", b"&#57344;");
        encode_shift_jis("\u{E757}", b"&#59223;");

        // JIS 0212
        encode_shift_jis("\u{02D8}", b"&#728;");

        // JIS 0208
        encode_shift_jis("\u{3000}", b"\x81\x40");
        encode_shift_jis("\u{FF02}", b"\xFA\x57");
        encode_shift_jis("\u{2170}", b"\xFA\x40");
        encode_shift_jis("\u{9ED1}", b"\xFC\x4B");
    }

    #[test]
    fn test_shift_jis_decode_all() {
        let input = include_bytes!("test_data/jis0208_in.txt");
        let expectation = include_str!("test_data/jis0208_in_ref.txt");
        let (cow, had_errors) = SHIFT_JIS.decode_without_bom_handling(input);
        assert!(had_errors, "Should have had errors.");
        assert_eq!(&cow[..], expectation);
    }
}
