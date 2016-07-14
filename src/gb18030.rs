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

pub struct Gb18030Decoder {
    first: u8,
    second: u8,
    third: u8,
    pending_ascii: u8,
}

impl Gb18030Decoder {
    pub fn new() -> VariantDecoder {
        VariantDecoder::Gb18030(Gb18030Decoder {
            first: 0,
            second: 0,
            third: 0,
            pending_ascii: 0,
        })
    }

    fn extra_from_state(&self, byte_length: usize) -> usize {
        byte_length +
        if self.first != 0 {
            1
        } else {
            0
        } +
        if self.second != 0 {
            1
        } else {
            0
        } +
        if self.third != 0 {
            1
        } else {
            0
        } +
        if self.pending_ascii != 0 {
            1
        } else {
            0
        }
    }

    pub fn max_utf16_buffer_length(&self, byte_length: usize) -> usize {
        // ASCII: 1 to 1 (worst case)
        // gbk: 2 to 1
        // ranges: 4 to 1 or 4 to 2
        self.extra_from_state(byte_length) + 1
    }

    pub fn max_utf8_buffer_length_without_replacement(&self, byte_length: usize) -> usize {
        // ASCII: 1 to 1
        // gbk: 2 to 2 or 2 to 3
        // ranges: 4 to 2, 4 to 3 or 4 to 4
        // 0x80: 1 to 3 (worst case)
        (self.extra_from_state(byte_length) * 3) + 1
    }

    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        (self.extra_from_state(byte_length) * 3) + 1
    }

    decoder_functions!({
                           if self.pending_ascii != 0 {
                               match dest.check_space_bmp() {
                                   Space::Full(_) => {
                                       return (DecoderResult::OutputFull, 0, 0);
                                   }
                                   Space::Available(destination_handle) => {
                                       destination_handle.write_ascii(self.pending_ascii);
                                       self.pending_ascii = 0;
                                   }
                               }
                           }
                       },
                       {
                           if self.third != 0 {
                               self.first = 0;
                               self.second = 0;
                               self.third = 0;
                               return (DecoderResult::Malformed(3, 0),
                                       src_consumed,
                                       dest.written());
                           }
                           if self.second != 0 {
                               self.first = 0;
                               self.second = 0;
                               self.third = 0;
                               return (DecoderResult::Malformed(2, 0),
                                       src_consumed,
                                       dest.written());
                           }
                           if self.first != 0 {
                               self.first = 0;
                               self.second = 0;
                               self.third = 0;
                               return (DecoderResult::Malformed(1, 0),
                                       src_consumed,
                                       dest.written());
                           }
                       },
                       {
                           if self.first == 0 {
                               debug_assert_eq!(self.second, 0);
                               debug_assert_eq!(self.third, 0);
                               if b <= 0x7f {
                                   // TODO optimize ASCII run
                                   destination_handle.write_ascii(b);
                                   continue;
                               }
                               if b == 0x80 {
                                   destination_handle.write_upper_bmp(0x20ACu16);
                                   continue;
                               }
                               if b >= 0x81 && b <= 0xFE {
                                   self.first = b;
                                   continue;
                               }
                               return (DecoderResult::Malformed(1, 0),
                                       unread_handle.consumed(),
                                       destination_handle.written());
                           }
                           if self.third != 0 {
                               let first = self.first;
                               let second = self.second;
                               let third = self.third;
                               self.first = 0;
                               self.second = 0;
                               self.third = 0;
                               if b >= 0x30 && b <= 0x39 {
                                   let pointer = ((first as usize - 0x81) * (10 * 126 * 10)) +
                                                 ((second as usize - 0x30) * (10 * 126)) +
                                                 ((third as usize - 0x81) * 10) +
                                                 (b as usize - 0x30);
                                   let c = gb18030_range_decode(pointer);
                                   if c != '\u{0}' {
                                       destination_handle.write_char_excl_ascii(c);
                                       continue;
                                   }
                               }
                               // We have an error. Let's inline what's going
                               // to happen when `second` and `third` are
                               // reprocessed. (`b` gets unread.)
                               debug_assert!(second >= 0x30 && second <= 0x39);
                               // `second` is guaranteed ASCII, so let's
                               // put it in `pending_ascii`
                               self.pending_ascii = second;
                               debug_assert!(third >= 0x81 && third <= 0xFE);
                               // `third` is guaranteed to be in the range
                               // that makes it become the new `self.first`.
                               self.first = third;
                               // Now unread `b` and designate the previous
                               // `first` as being in error.
                               return (DecoderResult::Malformed(1, 2),
                                       unread_handle.unread(),
                                       destination_handle.written());
                           }
                           if self.second != 0 {
                               debug_assert_eq!(self.third, 0);
                               if b >= 0x81 && b <= 0xFE {
                                   self.third = b;
                                   continue;
                               }
                               let second = self.second;
                               self.second = 0;
                               self.first = 0;
                               // We have an error. Let's inline what's going
                               // to happen when `second` is
                               // reprocessed. (`b` gets unread.)
                               debug_assert!(second >= 0x30 && second <= 0x39);
                               // `second` is guaranteed ASCII, so let's
                               // put it in `pending_ascii`
                               self.pending_ascii = second;
                               // Now unread `b` and designate the previous
                               // `first` as being in error.
                               return (DecoderResult::Malformed(1, 1),
                                       unread_handle.unread(),
                                       destination_handle.written());
                           }
                           // self.first != 0
                           debug_assert_eq!(self.second, 0);
                           debug_assert_eq!(self.third, 0);
                           if b >= 0x30 && b <= 0x39 {
                               self.second = b;
                               continue;
                           }
                           let lead = self.first;
                           self.first = 0;
                           let offset = if b < 0x7F {
                               0x40usize
                           } else {
                               0x41usize
                           };
                           if (b >= 0x40 && b <= 0x7E) || (b >= 0x80 && b <= 0xFE) {
                               let pointer = (lead as usize - 0x81) * 190usize +
                                             (b as usize - offset);
                               let bmp = gb18030_decode(pointer);
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
                       check_space_astral);
}

pub struct Gb18030Encoder {
    extended: bool,
}

impl Gb18030Encoder {
    pub fn new(encoding: &'static Encoding, extended_range: bool) -> Encoder {
        Encoder::new(encoding,
                     VariantEncoder::Gb18030(Gb18030Encoder { extended: extended_range }))
    }

    pub fn max_buffer_length_from_utf16_without_replacement(&self, u16_length: usize) -> usize {
        if self.extended {
            u16_length * 4
        } else {
            u16_length * 2
        }
    }

    pub fn max_buffer_length_from_utf8_without_replacement(&self, byte_length: usize) -> usize {
        if self.extended {
            // 1 to 1
            // 2 to 2
            // 3 to 2
            // 2 to 4 (worst)
            // 3 to 4
            // 4 to 4
            byte_length * 2
        } else {
            // 1 to 1
            // 2 to 2
            // 3 to 2
            byte_length
        }
    }

    encoder_functions!({},
                       {
                           if c <= '\u{7F}' {
                               // TODO optimize ASCII run
                               destination_handle.write_one(c as u8);
                               continue;
                           }
                           if c == '\u{E5E5}' {
                               return (EncoderResult::Unmappable(c),
                                       unread_handle.consumed(),
                                       destination_handle.written());
                           }
                           if !self.extended && c == '\u{20AC}' {
                               destination_handle.write_one(0x80u8);
                               continue;
                           }
                           let pointer = gb18030_encode(c);
                           if pointer != usize::max_value() {
                               let lead = (pointer / 190) + 0x81;
                               let trail = pointer % 190;
                               let offset = if trail < 0x3F {
                                   0x40
                               } else {
                                   0x41
                               };
                               destination_handle.write_two(lead as u8, (trail + offset) as u8);
                               continue;
                           }
                           if !self.extended {
                               return (EncoderResult::Unmappable(c),
                                       unread_handle.consumed(),
                                       destination_handle.written());
                           }
                           let range_pointer = gb18030_range_encode(c);
                           let first = range_pointer / (10 * 126 * 10);
                           let rem_first = range_pointer % (10 * 126 * 10);
                           let second = rem_first / (10 * 126);
                           let rem_second = rem_first % (10 * 126);
                           let third = rem_second / 10;
                           let fourth = rem_second % 10;
                           destination_handle.write_four((first + 0x81) as u8,
                                                         (second + 0x30) as u8,
                                                         (third + 0x81) as u8,
                                                         (fourth + 0x30) as u8);
                           continue;
                       },
                       self,
                       src_consumed,
                       source,
                       dest,
                       c,
                       destination_handle,
                       unread_handle,
                       check_space_four);
}

// Any copyright to the test code below this comment is dedicated to the
// Public Domain. http://creativecommons.org/publicdomain/zero/1.0/

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::*;

    fn decode_gb18030(bytes: &[u8], expect: &str) {
        decode(GB18030, bytes, expect);
    }

    fn encode_gb18030(string: &str, expect: &[u8]) {
        encode(GB18030, string, expect);
    }

    fn encode_gbk(string: &str, expect: &[u8]) {
        encode(GBK, string, expect);
    }

    #[test]
    fn test_gb18030_decode() {
        // ASCII
        decode_gb18030(b"\x61\x62", "\u{0061}\u{0062}");

        // euro
        decode_gb18030(b"\x80", "\u{20AC}");
        decode_gb18030(b"\xA2\xE3", "\u{20AC}");

        // two bytes
        decode_gb18030(b"\x81\x40", "\u{4E02}");
        decode_gb18030(b"\x81\x7E", "\u{4E8A}");
        decode_gb18030(b"\x81\x7F", "\u{FFFD}\u{007F}");
        decode_gb18030(b"\x81\x80", "\u{4E90}");
        decode_gb18030(b"\x81\xFE", "\u{4FA2}");
        decode_gb18030(b"\xFE\x40", "\u{FA0C}");
        decode_gb18030(b"\xFE\x7E", "\u{E843}");
        decode_gb18030(b"\xFE\x7F", "\u{FFFD}\u{007F}");
        decode_gb18030(b"\xFE\x80", "\u{4723}");
        decode_gb18030(b"\xFE\xFE", "\u{E4C5}");

        // The difference from the original GB18030
        decode_gb18030(b"\xA3\xA0", "\u{3000}");
        decode_gb18030(b"\xA1\xA1", "\u{3000}");

        // 0xFF
        decode_gb18030(b"\xFF\x40", "\u{FFFD}\u{0040}");

        // Four bytes
        decode_gb18030(b"\x81\x30\x81\x30", "\u{0080}");
        decode_gb18030(b"\x81\x35\xF4\x37", "\u{E7C7}");
        decode_gb18030(b"\x81\x37\xA3\x30", "\u{2603}");
        decode_gb18030(b"\x94\x39\xDA\x33", "\u{1F4A9}");
        decode_gb18030(b"\xE3\x32\x9A\x35", "\u{10FFFF}");
        decode_gb18030(b"\xE3\x32\x9A\x36\x81\x30", "\u{FFFD}\u{0032}\u{309B8}");
        decode_gb18030(b"\xE3\x32\x9A\x36\x81\x40",
                       "\u{FFFD}\u{0032}\u{FFFD}\u{0036}\u{4E02}");
        decode_gb18030(b"\xE3\x32\x9A", "\u{FFFD}"); // not \u{FFFD}\u{0032}\u{FFFD} !

    }

    #[test]
    fn test_gb18030_encode() {
        // ASCII
        encode_gb18030("\u{0061}\u{0062}", b"\x61\x62");

        // euro
        encode_gb18030("\u{20AC}", b"\xA2\xE3");

        // two bytes
        encode_gb18030("\u{4E02}", b"\x81\x40");
        encode_gb18030("\u{4E8A}", b"\x81\x7E");
        encode_gb18030("\u{4E90}", b"\x81\x80");
        encode_gb18030("\u{4FA2}", b"\x81\xFE");
        encode_gb18030("\u{FA0C}", b"\xFE\x40");
        encode_gb18030("\u{E843}", b"\xFE\x7E");
        encode_gb18030("\u{4723}", b"\xFE\x80");
        encode_gb18030("\u{E4C5}", b"\xFE\xFE");

        // The difference from the original GB18030
        encode_gb18030("\u{E5E5}", b"&#58853;");
        encode_gb18030("\u{3000}", b"\xA1\xA1");

        // Four bytes
        encode_gb18030("\u{0080}", b"\x81\x30\x81\x30");
        encode_gb18030("\u{E7C7}", b"\x81\x35\xF4\x37");
        encode_gb18030("\u{2603}", b"\x81\x37\xA3\x30");
        encode_gb18030("\u{1F4A9}", b"\x94\x39\xDA\x33");
        encode_gb18030("\u{10FFFF}", b"\xE3\x32\x9A\x35");
    }

    #[test]
    fn test_gbk_encode() {
        // ASCII
        encode_gbk("\u{0061}\u{0062}", b"\x61\x62");

        // euro
        encode_gbk("\u{20AC}", b"\x80");

        // two bytes
        encode_gbk("\u{4E02}", b"\x81\x40");
        encode_gbk("\u{4E8A}", b"\x81\x7E");
        encode_gbk("\u{4E90}", b"\x81\x80");
        encode_gbk("\u{4FA2}", b"\x81\xFE");
        encode_gbk("\u{FA0C}", b"\xFE\x40");
        encode_gbk("\u{E843}", b"\xFE\x7E");
        encode_gbk("\u{4723}", b"\xFE\x80");
        encode_gbk("\u{E4C5}", b"\xFE\xFE");

        // The difference from the original gb18030
        encode_gbk("\u{E5E5}", b"&#58853;");
        encode_gbk("\u{3000}", b"\xA1\xA1");

        // Four bytes
        encode_gbk("\u{0080}", b"&#128;");
        encode_gbk("\u{E7C7}", b"&#59335;");
        encode_gbk("\u{2603}", b"&#9731;");
        encode_gbk("\u{1F4A9}", b"&#128169;");
        encode_gbk("\u{10FFFF}", b"&#1114111;");
    }
}
