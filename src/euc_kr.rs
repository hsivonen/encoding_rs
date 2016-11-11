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
    lead: u8,
}

impl EucKrDecoder {
    pub fn new() -> VariantDecoder {
        VariantDecoder::EucKr(EucKrDecoder { lead: 0 })
    }

    fn plus_one_if_lead(&self, byte_length: usize) -> usize {
        byte_length +
        if self.lead == 0 {
            0
        } else {
            1
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

    decoder_functions!({},
                       {
                           if self.lead != 0 {
                               self.lead = 0;
                               return (DecoderResult::Malformed(1, 0),
                                       src_consumed,
                                       dest.written());
                           }
                       },
                       {
                           if self.lead == 0 {
                               if b <= 0x7f {
                                   // TODO optimize ASCII run
                                   destination_handle.write_ascii(b);
                                   continue;
                               }
                               if b >= 0x81 && b <= 0xFE {
                                   self.lead = b;
                                   continue;
                               }
                               return (DecoderResult::Malformed(1, 0),
                                       unread_handle.consumed(),
                                       destination_handle.written());
                           }
                           let lead = self.lead as usize;
                           self.lead = 0;
                           if b >= 0x41 && b <= 0xFE {
                               let pointer = (lead as usize - 0x81) * 190usize +
                                             (b as usize - 0x41);
                               let bmp = euc_kr_decode(pointer);
                               if bmp != 0 {
                                   destination_handle.write_bmp_excl_ascii(bmp);
                                   continue;
                               }
                           }
                           if b <= 0x7F {
                               return (DecoderResult::Malformed(1, 0),
                                       unread_handle.unread(),
                                       destination_handle.written());
                           }
                           return (DecoderResult::Malformed(2, 0),
                                   unread_handle.consumed(),
                                   destination_handle.written());
                       },
                       self,
                       src_consumed,
                       dest,
                       b,
                       destination_handle,
                       unread_handle,
                       check_space_bmp);
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
