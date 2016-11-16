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

}
