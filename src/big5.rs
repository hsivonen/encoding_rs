// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use handles::*;
use data::*;
use variant::*;
use super::*;

pub struct Big5Decoder {
    lead: u8,
}

impl Big5Decoder {
    pub fn new() -> VariantDecoder {
        VariantDecoder::Big5(Big5Decoder { lead: 0 })
    }

    fn plus_one_if_lead(&self, byte_length: usize) -> usize {
        byte_length +
        if self.lead == 0 {
            0
        } else {
            1
        }
    }

    pub fn max_utf16_buffer_length(&self, u16_length: usize) -> usize {
        // If there is a lead but the next byte isn't a valid trail, an
        // error is generated for the lead (+1). Then another iteration checks
        // space, which needs +1 to account for the possibility of astral
        // output or combining pair.
        self.plus_one_if_lead(u16_length) + 1
    }

    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        // No need to account for REPLACEMENT CHARACTERS.
        // Cases:
        // ASCII: 1 to 1
        // Valid pair: 2 to 2, 2 to 3 or 2 to 4, i.e. worst case 2 to 4
        // lead set and first byte is trail: 1 to 4 worst case
        //
        // When checking for space for the last byte:
        // no lead: the last byte must be ASCII (or fatal error): 1 to 1
        // lead set: space for 4 bytes was already checked when reading the
        // lead, hence the last lead and the last trail together are worst
        // case 2 to 4.
        //
        // If lead set and the input is a single trail byte, the worst-case
        // output is 4, so we need to add one before multiplying if lead is
        // set.
        let len = self.plus_one_if_lead(byte_length);
        (len * 2)
    }

    pub fn max_utf8_buffer_length_with_replacement(&self, byte_length: usize) -> usize {
        // If there is a lead but the next byte isn't a valid trail, an
        // error is generated for the lead (+(1*3)). Then another iteration
        // checks space, which needs +3 to account for the possibility of astral
        // output or combining pair. In between start and end, the worst case
        // is that every byte is bad: *3.
        3 * self.plus_one_if_lead(byte_length) + 3
    }

    decoder_functions!({},
                       {
                           if self.lead != 0 {
                               self.lead = 0;
                               return (DecoderResult::Malformed(1, 0), src_consumed, dest.written());
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
                           let offset = if b < 0x7F {
                               0x40usize
                           } else {
                               0x62usize
                           };
                           if (b >= 0x40 && b <= 0x7E) || (b >= 0xA1 && b <= 0xFE) {
                               let pointer = (lead - 0x81usize) * 157usize + (b as usize - offset);
                               match pointer {
                                   1133 => {
                                       destination_handle.write_big5_combination(0x00CAu16,
                                                                                 0x0304u16);
                                       continue;
                                   }
                                   1135 => {
                                       destination_handle.write_big5_combination(0x00CAu16,
                                                                                 0x030Cu16);
                                       continue;
                                   }
                                   1164 => {
                                       destination_handle.write_big5_combination(0x00EAu16,
                                                                                 0x0304u16);
                                       continue;
                                   }
                                   1166 => {
                                       destination_handle.write_big5_combination(0x00EAu16,
                                                                                 0x030Cu16);
                                       continue;
                                   }
                                   _ => {
                                       let low_bits = big5_low_bits(pointer);
                                       if low_bits == 0 {
                                           if b <= 0x7F {
                                               return (DecoderResult::Malformed(1, 0),
                                                       unread_handle.unread(),
                                                       destination_handle.written());
                                           }
                                           return (DecoderResult::Malformed(2, 0),
                                                   unread_handle.consumed(),
                                                   destination_handle.written());
                                       }
                                       if big5_is_astral(pointer) {
                                           destination_handle.write_astral(low_bits as u32 |
                                                                           0x20000u32);
                                           continue;
                                       }
                                       destination_handle.write_bmp_excl_ascii(low_bits);
                                       continue;
                                   }
                               }
                           }
                           // pointer is null
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
                       check_space_astral);
}

pub struct Big5Encoder;

impl Big5Encoder {
    pub fn new(encoding: &'static Encoding) -> Encoder {
        Encoder::new(encoding, VariantEncoder::Big5(Big5Encoder))
    }

    pub fn max_buffer_length_from_utf16(&self, u16_length: usize) -> usize {
        // Astral: 2 to 2
        // ASCII: 1 to 1
        // Other: 1 to 2
        2 * u16_length
    }

    pub fn max_buffer_length_from_utf8(&self, byte_length: usize) -> usize {
        // Astral: 4 to 2
        // Upper BMP: 3 to 2
        // Lower BMP: 2 to 2
        // ASCII: 1 to 1
        byte_length
    }

    encoder_functions!({},
                       {
                           if c <= '\u{7F}' {
                               // TODO optimize ASCII run
                               destination_handle.write_one(c as u8);
                               continue;
                           }
                           let high_bits = c as u32 & 0xFF0000;
                           let (low_bits, is_astral) = if high_bits == 0 {
                               (c as u16, false)
                           } else if high_bits == 0x20000 {
                               ((c as u32 & 0xFFFF) as u16, true)
                           } else {
                               // Only BMP and Plane 2 are potentially mappable.
                               return (EncoderResult::Unmappable(c),
                                       unread_handle.consumed(),
                                       destination_handle.written());
                           };
                           let pointer = big5_find_pointer(low_bits, is_astral);
                           if pointer == 0 {
                               return (EncoderResult::Unmappable(c),
                                       unread_handle.consumed(),
                                       destination_handle.written());
                           }
                           let lead = pointer / 157 + 0x81;
                           let remainder = pointer % 157;
                           let trail = if remainder < 0x3F {
                               remainder + 0x40
                           } else {
                               remainder + 0x62
                           };
                           destination_handle.write_two(lead as u8, trail as u8);
                           continue;
                       },
                       self,
                       src_consumed,
                       source,
                       c,
                       destination_handle,
                       unread_handle,
                       check_space_two);
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::*;

    fn decode_big5_to_utf16(bytes: &[u8], expect: &[u16]) {
        decode_to_utf16(BIG5, bytes, expect);
    }

    fn decode_big5_to_utf8(bytes: &[u8], expect: &str) {
        decode_to_utf8(BIG5, bytes, expect);
    }

    fn encode_big5_from_utf16(string: &[u16], expect: &[u8]) {
        encode_from_utf16(BIG5, string, expect);
    }

    fn encode_big5_from_utf8(string: &str, expect: &[u8]) {
        encode_from_utf8(BIG5, string, expect);
    }

    #[test]
    fn test_big5_decode() {
        // ASCII
        decode_big5_to_utf16(&[0x61u8, 0x62u8], &[0x0061u16, 0x0062u16]);
        // Edge cases
        decode_big5_to_utf16(&[0x87u8, 0x40u8], &[0x43F0u16]);
        decode_big5_to_utf16(&[0xFEu8, 0xFEu8], &[0x79D4u16]);
        decode_big5_to_utf16(&[0xFEu8, 0xFDu8], &[0xD864u16, 0xDD0Du16]);
        decode_big5_to_utf16(&[0x88u8, 0x62u8], &[0x00CAu16, 0x0304u16]);
        decode_big5_to_utf16(&[0x88u8, 0x64u8], &[0x00CAu16, 0x030Cu16]);
        decode_big5_to_utf16(&[0x88u8, 0x66u8], &[0x00CAu16]);
        decode_big5_to_utf16(&[0x88u8, 0xA3u8], &[0x00EAu16, 0x0304u16]);
        decode_big5_to_utf16(&[0x88u8, 0xA5u8], &[0x00EAu16, 0x030Cu16]);
        decode_big5_to_utf16(&[0x88u8, 0xA7u8], &[0x00EAu16]);
        decode_big5_to_utf16(&[0x99u8, 0xD4u8], &[0x8991u16]);
        decode_big5_to_utf16(&[0x99u8, 0xD5u8], &[0xD85Eu16, 0xDD67u16]);
        decode_big5_to_utf16(&[0x99u8, 0xD6u8], &[0x8A29u16]);
        // Edge cases surrounded with ASCII
        decode_big5_to_utf16(&[0x61u8, 0x87u8, 0x40u8, 0x62u8],
                             &[0x0061u16, 0x43F0u16, 0x0062u16]);
        decode_big5_to_utf16(&[0x61u8, 0xFEu8, 0xFEu8, 0x62u8],
                             &[0x0061u16, 0x79D4u16, 0x0062u16]);
        decode_big5_to_utf16(&[0x61u8, 0xFEu8, 0xFDu8, 0x62u8],
                             &[0x0061u16, 0xD864u16, 0xDD0Du16, 0x0062u16]);
        decode_big5_to_utf16(&[0x61u8, 0x88u8, 0x62u8, 0x62u8],
                             &[0x0061u16, 0x00CAu16, 0x0304u16, 0x0062u16]);
        decode_big5_to_utf16(&[0x61u8, 0x88u8, 0x64u8, 0x62u8],
                             &[0x0061u16, 0x00CAu16, 0x030Cu16, 0x0062u16]);
        decode_big5_to_utf16(&[0x61u8, 0x88u8, 0x66u8, 0x62u8],
                             &[0x0061u16, 0x00CAu16, 0x0062u16]);
        decode_big5_to_utf16(&[0x61u8, 0x88u8, 0xA3u8, 0x62u8],
                             &[0x0061u16, 0x00EAu16, 0x0304u16, 0x0062u16]);
        decode_big5_to_utf16(&[0x61u8, 0x88u8, 0xA5u8, 0x62u8],
                             &[0x0061u16, 0x00EAu16, 0x030Cu16, 0x0062u16]);
        decode_big5_to_utf16(&[0x61u8, 0x88u8, 0xA7u8, 0x62u8],
                             &[0x0061u16, 0x00EAu16, 0x0062u16]);
        decode_big5_to_utf16(&[0x61u8, 0x99u8, 0xD4u8, 0x62u8],
                             &[0x0061u16, 0x8991u16, 0x0062u16]);
        decode_big5_to_utf16(&[0x61u8, 0x99u8, 0xD5u8, 0x62u8],
                             &[0x0061u16, 0xD85Eu16, 0xDD67u16, 0x0062u16]);
        decode_big5_to_utf16(&[0x61u8, 0x99u8, 0xD6u8, 0x62u8],
                             &[0x0061u16, 0x8A29u16, 0x0062u16]);
        // Bad sequences
        decode_big5_to_utf16(&[0x80u8, 0x61u8], &[0xFFFDu16, 0x0061u16]);
        decode_big5_to_utf16(&[0xFFu8, 0x61u8], &[0xFFFDu16, 0x0061u16]);
        decode_big5_to_utf16(&[0xFEu8, 0x39u8], &[0xFFFDu16, 0x0039u16]);
        decode_big5_to_utf16(&[0x87u8, 0x66u8], &[0xFFFDu16, 0x0066u16]);
        decode_big5_to_utf16(&[0x81u8, 0x40u8], &[0xFFFDu16, 0x0040u16]);
        decode_big5_to_utf16(&[0x61u8, 0x81u8], &[0x0061u16, 0xFFFDu16]);

        // ASCII
        decode_big5_to_utf8(&[0x61u8, 0x62u8], &"\u{0061}\u{0062}");
        // Edge cases
        decode_big5_to_utf8(&[0x87u8, 0x40u8], &"\u{43F0}");
        decode_big5_to_utf8(&[0xFEu8, 0xFEu8], &"\u{79D4}");
        decode_big5_to_utf8(&[0xFEu8, 0xFDu8], &"\u{2910D}");
        decode_big5_to_utf8(&[0x88u8, 0x62u8], &"\u{00CA}\u{0304}");
        decode_big5_to_utf8(&[0x88u8, 0x64u8], &"\u{00CA}\u{030C}");
        decode_big5_to_utf8(&[0x88u8, 0x66u8], &"\u{00CA}");
        decode_big5_to_utf8(&[0x88u8, 0xA3u8], &"\u{00EA}\u{0304}");
        decode_big5_to_utf8(&[0x88u8, 0xA5u8], &"\u{00EA}\u{030C}");
        decode_big5_to_utf8(&[0x88u8, 0xA7u8], &"\u{00EA}");
        decode_big5_to_utf8(&[0x99u8, 0xD4u8], &"\u{8991}");
        decode_big5_to_utf8(&[0x99u8, 0xD5u8], &"\u{27967}");
        decode_big5_to_utf8(&[0x99u8, 0xD6u8], &"\u{8A29}");
        // Edge cases surrounded with ASCII
        decode_big5_to_utf8(&[0x61u8, 0x87u8, 0x40u8, 0x62u8],
                            &"\u{0061}\u{43F0}\u{0062}");
        decode_big5_to_utf8(&[0x61u8, 0xFEu8, 0xFEu8, 0x62u8],
                            &"\u{0061}\u{79D4}\u{0062}");
        decode_big5_to_utf8(&[0x61u8, 0xFEu8, 0xFDu8, 0x62u8],
                            &"\u{0061}\u{2910D}\u{0062}");
        decode_big5_to_utf8(&[0x61u8, 0x88u8, 0x62u8, 0x62u8],
                            &"\u{0061}\u{00CA}\u{0304}\u{0062}");
        decode_big5_to_utf8(&[0x61u8, 0x88u8, 0x64u8, 0x62u8],
                            &"\u{0061}\u{00CA}\u{030C}\u{0062}");
        decode_big5_to_utf8(&[0x61u8, 0x88u8, 0x66u8, 0x62u8],
                            &"\u{0061}\u{00CA}\u{0062}");
        decode_big5_to_utf8(&[0x61u8, 0x88u8, 0xA3u8, 0x62u8],
                            &"\u{0061}\u{00EA}\u{0304}\u{0062}");
        decode_big5_to_utf8(&[0x61u8, 0x88u8, 0xA5u8, 0x62u8],
                            &"\u{0061}\u{00EA}\u{030C}\u{0062}");
        decode_big5_to_utf8(&[0x61u8, 0x88u8, 0xA7u8, 0x62u8],
                            &"\u{0061}\u{00EA}\u{0062}");
        decode_big5_to_utf8(&[0x61u8, 0x99u8, 0xD4u8, 0x62u8],
                            &"\u{0061}\u{8991}\u{0062}");
        decode_big5_to_utf8(&[0x61u8, 0x99u8, 0xD5u8, 0x62u8],
                            &"\u{0061}\u{27967}\u{0062}");
        decode_big5_to_utf8(&[0x61u8, 0x99u8, 0xD6u8, 0x62u8],
                            &"\u{0061}\u{8A29}\u{0062}");
        // Bad sequences
        decode_big5_to_utf8(&[0x80u8, 0x61u8], &"\u{FFFD}\u{0061}");
        decode_big5_to_utf8(&[0xFFu8, 0x61u8], &"\u{FFFD}\u{0061}");
        decode_big5_to_utf8(&[0xFEu8, 0x39u8], &"\u{FFFD}\u{0039}");
        decode_big5_to_utf8(&[0x87u8, 0x66u8], &"\u{FFFD}\u{0066}");
        decode_big5_to_utf8(&[0x81u8, 0x40u8], &"\u{FFFD}\u{0040}");
        decode_big5_to_utf8(&[0x61u8, 0x81u8], &"\u{0061}\u{FFFD}");
    }

    #[test]
    fn test_big5_encode() {
        // ASCII
        encode_big5_from_utf16(&[0x0061u16, 0x0062u16], b"\x61\x62");
        // Edge cases
        encode_big5_from_utf16(&[0x9EA6u16, 0x0061u16], b"&#40614;\x61");
        encode_big5_from_utf16(&[0xD858u16, 0xDE6Bu16, 0x0061u16], b"&#156267;\x61");
        encode_big5_from_utf16(&[0x3000u16], b"\xA1\x40");
        encode_big5_from_utf16(&[0x20ACu16], b"\xA3\xE1");
        encode_big5_from_utf16(&[0x4E00u16], b"\xA4\x40");
        encode_big5_from_utf16(&[0xD85Du16, 0xDE07u16], b"\xC8\xA4");
        encode_big5_from_utf16(&[0xFFE2u16], b"\xC8\xCD");
        encode_big5_from_utf16(&[0x79D4u16], b"\xFE\xFE");
        // Not in index
        encode_big5_from_utf16(&[0x2603u16, 0x0061u16], b"&#9731;\x61");
        // duplicate low bits
        encode_big5_from_utf16(&[0xD840u16, 0xDFB5u16], b"\xFD\x6A");
        // prefer last
        encode_big5_from_utf16(&[0x2550u16], b"\xF9\xF9");

        // ASCII
        encode_big5_from_utf8("\u{0061}\u{0062}", b"\x61\x62");
        // Edge cases
        encode_big5_from_utf8("\u{9EA6}\u{0061}", b"&#40614;\x61");
        encode_big5_from_utf8("\u{2626B}\u{0061}", b"&#156267;\x61");
        encode_big5_from_utf8("\u{3000}", b"\xA1\x40");
        encode_big5_from_utf8("\u{20AC}", b"\xA3\xE1");
        encode_big5_from_utf8("\u{4E00}", b"\xA4\x40");
        encode_big5_from_utf8("\u{27607}", b"\xC8\xA4");
        encode_big5_from_utf8("\u{FFE2}", b"\xC8\xCD");
        encode_big5_from_utf8("\u{79D4}", b"\xFE\xFE");
        // Not in index
        encode_big5_from_utf8("\u{2603}\u{0061}", b"&#9731;\x61");
        // duplicate low bits
        encode_big5_from_utf8("\u{203B5}", b"\xFD\x6A");
        // prefer last
        encode_big5_from_utf8("\u{2550}", b"\xF9\xF9");
    }
}
