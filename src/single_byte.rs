// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use handles::*;
use variant::*;
use super::*;

pub struct SingleByteDecoder {
    table: &'static [u16; 128],
}

impl SingleByteDecoder {
    pub fn new(data: &'static [u16; 128]) -> VariantDecoder {
        VariantDecoder::SingleByte(SingleByteDecoder { table: data })
    }

    pub fn max_utf16_buffer_length(&self, byte_length: usize) -> usize {
        byte_length
    }

    pub fn max_utf8_buffer_length_without_replacement(&self, byte_length: usize) -> usize {
        byte_length * 3
    }

    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        byte_length * 3
    }

    decoder_functions!({},
                       {},
                       {
                           if b < 0x80 {
                               // XXX optimize ASCII
                               destination_handle.write_ascii(b);
                               continue;
                           }
                           let mapped = self.table[b as usize - 0x80usize];
                           if mapped == 0u16 {
                               return (DecoderResult::Malformed(1, 0),
                                       unread_handle.consumed(),
                                       destination_handle.written());
                           }
                           destination_handle.write_bmp_excl_ascii(mapped);
                           continue;
                       },
                       self,
                       src_consumed,
                       dest,
                       b,
                       destination_handle,
                       unread_handle,
                       check_space_bmp);
}

pub struct SingleByteEncoder {
    table: &'static [u16; 128],
}

impl SingleByteEncoder {
    pub fn new(encoding: &'static Encoding, data: &'static [u16; 128]) -> Encoder {
        Encoder::new(encoding,
                     VariantEncoder::SingleByte(SingleByteEncoder { table: data }))
    }

    pub fn max_buffer_length_from_utf16_without_replacement(&self, u16_length: usize) -> usize {
        u16_length
    }

    pub fn max_buffer_length_from_utf8_without_replacement(&self, byte_length: usize) -> usize {
        byte_length
    }

    encoder_functions!({},
                       {
                           if c <= '\u{7F}' {
                               // TODO optimize ASCII run
                               destination_handle.write_one(c as u8);
                               continue;
                           }
                           if c > '\u{FFFF}' {
                               return (EncoderResult::Unmappable(c),
                                       unread_handle.consumed(),
                                       destination_handle.written());
                           }
                           let bmp = c as u16;
                           // Loop backwards, because the lowest quarter
                           // is the least probable.
                           let mut i = 127usize;
                           loop {
                               if self.table[i] == bmp {
                                   destination_handle.write_one((i + 128) as u8);
                                   break; // i.e. continue outer loop
                               }
                               if i == 0 {
                                   return (EncoderResult::Unmappable(c),
                                           unread_handle.consumed(),
                                           destination_handle.written());
                               }
                               i -= 1;
                           }
                       },
                       self,
                       src_consumed,
                       source,
                       dest,
                       c,
                       destination_handle,
                       unread_handle,
                       check_space_one);
}

// Any copyright to the test code below this comment is dedicated to the
// Public Domain. http://creativecommons.org/publicdomain/zero/1.0/

#[cfg(test)]
mod tests {
    use super::super::data::*;
    use super::super::testing::*;
    use super::super::*;

    #[test]
    fn test_windows_1255_ca() {
        decode(WINDOWS_1255, b"\xCA", "\u{05BA}");
        encode(WINDOWS_1255, "\u{05BA}", b"\xCA");
    }

    pub const HIGH_BYTES: &'static [u8; 128] = &[0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
                                                 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F,
                                                 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
                                                 0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F,
                                                 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7,
                                                 0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF,
                                                 0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7,
                                                 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF,
                                                 0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7,
                                                 0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF,
                                                 0xD0, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7,
                                                 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF,
                                                 0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7,
                                                 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF,
                                                 0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7,
                                                 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF];

    fn decode_single_byte(encoding: &'static Encoding, data: &'static [u16; 128]) {
        let mut with_replacement = [0u16; 128];
        let mut it = data.iter().enumerate();
        loop {
            match it.next() {
                Some((i, code_point)) => {
                    if *code_point == 0 {
                        with_replacement[i] = 0xFFFD;
                    } else {
                        with_replacement[i] = *code_point;
                    }
                }
                None => {
                    break;
                }
            }
        }

        decode_to_utf16(encoding, HIGH_BYTES, &with_replacement[..]);
    }

    fn encode_single_byte(encoding: &'static Encoding, data: &'static [u16; 128]) {
        let mut with_zeros = [0u8; 128];
        let mut it = data.iter().enumerate();
        loop {
            match it.next() {
                Some((i, code_point)) => {
                    if *code_point == 0 {
                        with_zeros[i] = 0;
                    } else {
                        with_zeros[i] = HIGH_BYTES[i];
                    }
                }
                None => {
                    break;
                }
            }
        }

        encode_from_utf16(encoding, data, &with_zeros[..]);
    }

    // These tests are so self-referential that they are pretty useless.

    // BEGIN GENERATED CODE. PLEASE DO NOT EDIT.
    // Instead, please regenerate using generate-encoding-data.py

    #[test]
    fn test_single_byte_decode() {
        decode_single_byte(IBM866, IBM866_DATA);
        decode_single_byte(ISO_8859_10, ISO_8859_10_DATA);
        decode_single_byte(ISO_8859_13, ISO_8859_13_DATA);
        decode_single_byte(ISO_8859_14, ISO_8859_14_DATA);
        decode_single_byte(ISO_8859_15, ISO_8859_15_DATA);
        decode_single_byte(ISO_8859_16, ISO_8859_16_DATA);
        decode_single_byte(ISO_8859_2, ISO_8859_2_DATA);
        decode_single_byte(ISO_8859_3, ISO_8859_3_DATA);
        decode_single_byte(ISO_8859_4, ISO_8859_4_DATA);
        decode_single_byte(ISO_8859_5, ISO_8859_5_DATA);
        decode_single_byte(ISO_8859_6, ISO_8859_6_DATA);
        decode_single_byte(ISO_8859_7, ISO_8859_7_DATA);
        decode_single_byte(ISO_8859_8, ISO_8859_8_DATA);
        decode_single_byte(KOI8_R, KOI8_R_DATA);
        decode_single_byte(KOI8_U, KOI8_U_DATA);
        decode_single_byte(MACINTOSH, MACINTOSH_DATA);
        decode_single_byte(WINDOWS_1250, WINDOWS_1250_DATA);
        decode_single_byte(WINDOWS_1251, WINDOWS_1251_DATA);
        decode_single_byte(WINDOWS_1252, WINDOWS_1252_DATA);
        decode_single_byte(WINDOWS_1253, WINDOWS_1253_DATA);
        decode_single_byte(WINDOWS_1254, WINDOWS_1254_DATA);
        decode_single_byte(WINDOWS_1255, WINDOWS_1255_DATA);
        decode_single_byte(WINDOWS_1256, WINDOWS_1256_DATA);
        decode_single_byte(WINDOWS_1257, WINDOWS_1257_DATA);
        decode_single_byte(WINDOWS_1258, WINDOWS_1258_DATA);
        decode_single_byte(WINDOWS_874, WINDOWS_874_DATA);
        decode_single_byte(X_MAC_CYRILLIC, X_MAC_CYRILLIC_DATA);
    }

    #[test]
    fn test_single_byte_encode() {
        encode_single_byte(IBM866, IBM866_DATA);
        encode_single_byte(ISO_8859_10, ISO_8859_10_DATA);
        encode_single_byte(ISO_8859_13, ISO_8859_13_DATA);
        encode_single_byte(ISO_8859_14, ISO_8859_14_DATA);
        encode_single_byte(ISO_8859_15, ISO_8859_15_DATA);
        encode_single_byte(ISO_8859_16, ISO_8859_16_DATA);
        encode_single_byte(ISO_8859_2, ISO_8859_2_DATA);
        encode_single_byte(ISO_8859_3, ISO_8859_3_DATA);
        encode_single_byte(ISO_8859_4, ISO_8859_4_DATA);
        encode_single_byte(ISO_8859_5, ISO_8859_5_DATA);
        encode_single_byte(ISO_8859_6, ISO_8859_6_DATA);
        encode_single_byte(ISO_8859_7, ISO_8859_7_DATA);
        encode_single_byte(ISO_8859_8, ISO_8859_8_DATA);
        encode_single_byte(KOI8_R, KOI8_R_DATA);
        encode_single_byte(KOI8_U, KOI8_U_DATA);
        encode_single_byte(MACINTOSH, MACINTOSH_DATA);
        encode_single_byte(WINDOWS_1250, WINDOWS_1250_DATA);
        encode_single_byte(WINDOWS_1251, WINDOWS_1251_DATA);
        encode_single_byte(WINDOWS_1252, WINDOWS_1252_DATA);
        encode_single_byte(WINDOWS_1253, WINDOWS_1253_DATA);
        encode_single_byte(WINDOWS_1254, WINDOWS_1254_DATA);
        encode_single_byte(WINDOWS_1255, WINDOWS_1255_DATA);
        encode_single_byte(WINDOWS_1256, WINDOWS_1256_DATA);
        encode_single_byte(WINDOWS_1257, WINDOWS_1257_DATA);
        encode_single_byte(WINDOWS_1258, WINDOWS_1258_DATA);
        encode_single_byte(WINDOWS_874, WINDOWS_874_DATA);
        encode_single_byte(X_MAC_CYRILLIC, X_MAC_CYRILLIC_DATA);
    }
    // END GENERATED CODE

}
