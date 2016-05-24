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

pub struct ShiftJisDecoder {
    lead: u8,
}

impl ShiftJisDecoder {
    pub fn new() -> VariantDecoder {
        VariantDecoder::ShiftJis(ShiftJisDecoder { lead: 0 })
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

    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        // worst case: 2 to 3
        let len = self.plus_one_if_lead(byte_length);
        len + (len + 1) / 2
    }

    pub fn max_utf8_buffer_length_with_replacement(&self, byte_length: usize) -> usize {
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
                               if b == 0x80 {
                                   destination_handle.write_mid_bmp(b as u16);
                                   continue;
                               }
                               if b >= 0xA1 && b <= 0xDF {
                                   destination_handle.write_upper_bmp(0xFF61 + b as u16 - 0xA1);
                                   continue;
                               }
                               if (b >= 0x81 && b <= 0x9F) || (b >= 0xE0 && b <= 0xFC) {
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
                               0x41usize
                           };
                           let lead_offset = if lead < 0xA0 {
                               0x81usize
                           } else {
                               0xC1usize
                           };
                           if (b >= 0x40 && b <= 0x7E) || (b >= 0x80 && b <= 0xFC) {
                               let pointer = (lead as usize - lead_offset) * 188usize +
                                             (b as usize - offset);
                               let bmp = jis0208_decode(pointer);
                               if bmp != 0 {
                                   destination_handle.write_bmp_excl_ascii(bmp);
                                   continue;
                               } else if pointer >= 8836 && pointer <= 10528 {
                                   destination_handle.write_upper_bmp((0xE000 + pointer - 8836) as u16);
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

pub struct ShiftJisEncoder;

impl ShiftJisEncoder {
    pub fn new(encoding: &'static Encoding) -> Encoder {
        Encoder::new(encoding, VariantEncoder::ShiftJis(ShiftJisEncoder))
    }

    pub fn max_buffer_length_from_utf16(&self, u16_length: usize) -> usize {
        u16_length * 2
    }

    pub fn max_buffer_length_from_utf8(&self, byte_length: usize) -> usize {
        byte_length
    }

    encoder_functions!({},
                       {
                           if c <= '\u{7F}' {
                               // TODO optimize ASCII run
                               destination_handle.write_one(c as u8);
                               continue;
                           }
                           if c == '\u{80}' {
                               destination_handle.write_one(0x80u8);
                               continue;
                           }
                           if c == '\u{A5}' {
                               destination_handle.write_one(0x5Cu8);
                               continue;
                           }
                           if c == '\u{203E}' {
                               destination_handle.write_one(0x7Eu8);
                               continue;
                           }
                           if c >= '\u{FF61}' && c <= '\u{FF9F}' {
                               destination_handle.write_one((c as usize - 0xFF61 + 0xA1) as u8);
                               continue;
                           }
                           if c == '\u{2212}' {
                               destination_handle.write_two(0x81u8, 0x7Cu8);
                               continue;
                           }
                           let pointer = jis0208_encode(c);
                           if pointer == usize::max_value() {
                               return (EncoderResult::Unmappable(c),
                                       unread_handle.consumed(),
                                       destination_handle.written());
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
                           destination_handle.write_two((lead + lead_offset) as u8,
                                                        (trail + trail_offset) as u8);
                           continue;
                       },
                       self,
                       src_consumed,
                       source,
                       dest,
                       c,
                       destination_handle,
                       unread_handle,
                       check_space_two);
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;

}
