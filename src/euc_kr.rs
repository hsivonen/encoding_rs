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

pub struct EucKrDecoder {
    lead: Option<u8>,
}

impl EucKrDecoder {
    pub fn new() -> VariantDecoder {
        VariantDecoder::EucKr(EucKrDecoder { lead: None })
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
    // If lead is between 0x81 and 0xFE, inclusive,
    // subtract offset 0x81.
                                                     let non_ascii_minus_offset =
                                                         non_ascii.wrapping_sub(0x81);
                                                     if non_ascii_minus_offset > (0xFE - 0x81) {
                                                         return (DecoderResult::Malformed(1, 0),
                                                                 source.consumed(),
                                                                 handle.written());
                                                     }
                                                     non_ascii_minus_offset
                                                 },
                                                 {
                                                     if lead_minus_offset >= 0x20 {
    // Not the extension range above KS X 1001
                                                         let trail_minus_offset =
                                                             byte.wrapping_sub(0xA1);
                                                         if trail_minus_offset <= (0xFE - 0xA1) {
    // KS X 1001
                                                             let ksx_pointer = mul_94(lead_minus_offset - 0x20) + trail_minus_offset as usize;
                                                             let hangul_pointer = ksx_pointer.wrapping_sub((0x2F - 0x20) * 94);
                                                             if hangul_pointer < KSX1001_HANGUL.len() {
                                                                 let upper_bmp = KSX1001_HANGUL[hangul_pointer];
                                                                 handle.write_upper_bmp(upper_bmp)
                                                             } else if ksx_pointer < KSX1001_SYMBOLS.len() {
                                                                 let bmp = KSX1001_SYMBOLS[ksx_pointer];
                                                                 handle.write_bmp_excl_ascii(bmp)
                                                             } else {
                                                                 let hanja_pointer = ksx_pointer.wrapping_sub((0x49 - 0x20) * 94);
                                                                 if hanja_pointer < KSX1001_HANJA.len() {
                                                                     let upper_bmp = KSX1001_HANJA[hanja_pointer];
                                                                     handle.write_upper_bmp(upper_bmp)
                                                                 } else if (lead_minus_offset == 0x27) && ((trail_minus_offset as usize) < KSX1001_UPPERCASE.len()) {
                                                                     let mid_bmp = KSX1001_UPPERCASE[trail_minus_offset as usize];
                                                                     if mid_bmp == 0 {
                                                                         return (DecoderResult::Malformed(2, 0),
                                                                                 unread_handle_trail.consumed(),
                                                                                 handle.written());
                                                                     }
                                                                     handle.write_mid_bmp(mid_bmp)
                                                                 } else if (lead_minus_offset == 0x28) && ((trail_minus_offset as usize) < KSX1001_LOWERCASE.len()) {
                                                                     let mid_bmp = KSX1001_LOWERCASE[trail_minus_offset as usize];
                                                                     handle.write_mid_bmp(mid_bmp)
                                                                 } else if (lead_minus_offset == 0x25) && ((trail_minus_offset as usize) < KSX1001_BOX.len()) {
                                                                     let upper_bmp = KSX1001_BOX[trail_minus_offset as usize];
                                                                     handle.write_upper_bmp(upper_bmp)
                                                                 } else {
                                                                     let other_pointer = ksx_pointer.wrapping_sub(2 * 94);
                                                                     if other_pointer < 0x039F {
                                                                         let bmp = ksx1001_other_decode(other_pointer as u16);
    // ASCII range means unassigned
                                                                         if bmp < 0x80 {
                                                                             return (DecoderResult::Malformed(2, 0),
                                                                                     unread_handle_trail.consumed(),
                                                                                     handle.written());
                                                                         }
                                                                         handle.write_bmp_excl_ascii(bmp)
                                                                     } else {
                                                                         return (DecoderResult::Malformed(2, 0),
                                                                                 unread_handle_trail.consumed(),
                                                                                 handle.written());
                                                                     }
                                                                 }
                                                             }
                                                         } else {
    // Extension range to the left of
    // KS X 1001
                                                             let left_lead = lead_minus_offset - 0x20;
                                                             let left_trail = if byte.wrapping_sub(0x40 + 0x41) < (0x60 - 0x40) {
                                                                 byte - (12 + 0x41)
                                                             } else if byte.wrapping_sub(0x20 + 0x41) < (0x3A - 0x20) {
                                                                 byte - (6 + 0x41)
                                                             } else if byte.wrapping_sub(0x41) < 0x1A {
                                                                 byte - 0x41
                                                             } else {
                                                                 if byte < 0x80 {
                                                                     return (DecoderResult::Malformed(1, 0),
                                                                             unread_handle_trail.unread(),
                                                                             handle.written());
                                                                 }
                                                                 return (DecoderResult::Malformed(2, 0),
                                                                         unread_handle_trail.consumed(),
                                                                         handle.written());
                                                             };
                                                             let left_pointer = ((left_lead as usize) * (190 - 94 - 12)) + left_trail as usize;
                                                             if left_pointer < (0x45 - 0x20) * (190 - 94 - 12) + 0x12 {
                                                                 let upper_bmp = cp949_left_hangul_decode(left_pointer as u16);
                                                                 handle.write_upper_bmp(upper_bmp)
                                                             } else {
                                                                 if byte < 0x80 {
                                                                     return (DecoderResult::Malformed(1, 0),
                                                                             unread_handle_trail.unread(),
                                                                             handle.written());
                                                                 }
                                                                 return (DecoderResult::Malformed(2, 0),
                                                                         unread_handle_trail.consumed(),
                                                                         handle.written());
                                                             }
                                                         }
                                                     } else {
    // Extension range above KS X 1001
    // If trail is between 0x41 and 0xFE, inclusive,
    // subtract offset 0x41.
                                                     let trail_minus_offset =
                                                         byte.wrapping_sub(0x41);
                                                     if trail_minus_offset > (0xFE - 0x41) {
                                                         if byte < 0x80 {
                                                             return (DecoderResult::Malformed(1, 0),
                                                                     unread_handle_trail.unread(),
                                                                     handle.written());
                                                         }
                                                         return (DecoderResult::Malformed(2, 0),
                                                                 unread_handle_trail.consumed(),
                                                                 handle.written());
                                                     }
                                                     let pointer = lead_minus_offset as usize *
                                                                   190usize +
                                                                   trail_minus_offset as usize;
                                                     let bmp = euc_kr_decode(pointer);
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
                                                 true);
}

pub struct EucKrEncoder;

impl EucKrEncoder {
    pub fn new(encoding: &'static Encoding) -> Encoder {
        Encoder::new(encoding, VariantEncoder::EucKr(EucKrEncoder))
    }

    pub fn max_buffer_length_from_utf16_without_replacement(&self, u16_length: usize) -> usize {
        u16_length * 2
    }

    pub fn max_buffer_length_from_utf8_without_replacement(&self, byte_length: usize) -> usize {
        byte_length
    }

    ascii_compatible_bmp_encoder_functions!({
                                                let pointer = euc_kr_encode(bmp);
                                                if pointer == usize::max_value() {
                                                    return (EncoderResult::unmappable_from_bmp(bmp),
                                                            source.consumed(),
                                                            handle.written());
                                                }
                                                let lead = (pointer / 190) + 0x81;
                                                let trail = (pointer % 190) + 0x41;
                                                handle.write_two(lead as u8, trail as u8)
                                            },
                                            bmp,
                                            self,
                                            source,
                                            handle,
                                            copy_ascii_to_check_space_two,
                                            check_space_two,
                                            true);
}

// Any copyright to the test code below this comment is dedicated to the
// Public Domain. http://creativecommons.org/publicdomain/zero/1.0/

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::*;

    fn decode_euc_kr(bytes: &[u8], expect: &str) {
        decode(EUC_KR, bytes, expect);
    }

    fn encode_euc_kr(string: &str, expect: &[u8]) {
        encode(EUC_KR, string, expect);
    }

    #[test]
    fn test_euc_kr_decode() {
        // Empty
        decode_euc_kr(b"", &"");

        // ASCII
        decode_euc_kr(b"\x61\x62", "\u{0061}\u{0062}");

        decode_euc_kr(b"\x81\x41", "\u{AC02}");
        decode_euc_kr(b"\x81\x5B", "\u{FFFD}\x5B");
        decode_euc_kr(b"\xFD\xFE", "\u{8A70}");
        decode_euc_kr(b"\xFE\x41", "\u{FFFD}\x41");
        decode_euc_kr(b"\xFF\x41", "\u{FFFD}\x41");
        decode_euc_kr(b"\x80\x41", "\u{FFFD}\x41");
        decode_euc_kr(b"\xA1\xFF", "\u{FFFD}");
    }

    #[test]
    fn test_euc_kr_encode() {
        // Empty
        encode_euc_kr("", b"");

        // ASCII
        encode_euc_kr("\u{0061}\u{0062}", b"\x61\x62");

        encode_euc_kr("\u{AC02}", b"\x81\x41");
        encode_euc_kr("\u{8A70}", b"\xFD\xFE");
    }

    #[test]
    fn test_euc_kr_decode_all() {
        let input = include_bytes!("test_data/euc_kr_in.txt");
        let expectation = include_str!("test_data/euc_kr_in_ref.txt");
        let (cow, had_errors) = EUC_KR.decode_without_bom_handling(input);
        assert!(had_errors, "Should have had errors.");
        assert_eq!(&cow[..], expectation);
    }

    #[test]
    fn test_euc_kr_encode_all() {
        let input = include_str!("test_data/euc_kr_out.txt");
        let expectation = include_bytes!("test_data/euc_kr_out_ref.txt");
        let (cow, encoding, had_errors) = EUC_KR.encode(input);
        assert!(!had_errors, "Should not have had errors.");
        assert_eq!(encoding, EUC_KR);
        assert_eq!(&cow[..], &expectation[..]);
    }
}
