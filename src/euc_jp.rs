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

pub struct EucJpDecoder {
    lead: u8,
    jis0212: bool,
}

impl EucJpDecoder {
    pub fn new() -> VariantDecoder {
        VariantDecoder::EucJp(EucJpDecoder {
            lead: 0,
            jis0212: false,
        })
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
                               if (b >= 0xA1 && b <= 0xFE) || b == 0x8E || b == 0x8F {
                                   self.lead = b;
                                   continue;
                               }
                               return (DecoderResult::Malformed(1, 0),
                                       unread_handle.consumed(),
                                       destination_handle.written());
                           }
                           let lead = self.lead as usize;
                           self.lead = 0;
                           // Comparison to 0xA1 could be hoisted, but the
                           // form below matches the spec better.
                           if lead == 0x8E && (b >= 0xA1 && b <= 0xDF) {
                               destination_handle.write_upper_bmp(0xFF61 + b as u16 - 0xA1);
                               continue;
                           }
                           if lead == 0x8F && (b >= 0xA1 && b <= 0xFE) {
                               self.lead = b;
                               self.jis0212 = true;
                               continue;
                           }
                           if (b >= 0xA1 && b <= 0xFE) && (lead >= 0xA1 && lead <= 0xFE) {
                               let pointer = (lead as usize - 0xA1) * 94usize + (b as usize - 0xA1);
                               let bmp = if self.jis0212 {
                                   self.jis0212 = false;
                                   jis0212_decode(pointer)
                               } else {
                                   jis0208_decode(pointer)
                               };
                               if bmp != 0 {
                                   destination_handle.write_bmp_excl_ascii(bmp);
                                   continue;
                               }
                           }
                           if b < 0xA1 || b == 0xFF {
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

pub struct EucJpEncoder;

impl EucJpEncoder {
    pub fn new(encoding: &'static Encoding) -> Encoder {
        Encoder::new(encoding, VariantEncoder::EucJp(EucJpEncoder))
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
                           if c == '\u{A5}' {
                               destination_handle.write_one(0x5Cu8);
                               continue;
                           }
                           if c == '\u{203E}' {
                               destination_handle.write_one(0x7Eu8);
                               continue;
                           }
                           if c >= '\u{FF61}' && c <= '\u{FF9F}' {
                               destination_handle.write_two(0x8Eu8, (c as usize - 0xFF61 + 0xA1) as u8);
                               continue;
                           }
                           if c == '\u{2212}' {
                               destination_handle.write_two(0xA1u8, 0xDDu8);
                               continue;
                           }
                           let pointer = jis0208_encode(c);
                           if pointer == usize::max_value() {
                               return (EncoderResult::Unmappable(c),
                                       unread_handle.consumed(),
                                       destination_handle.written());
                           }
                           let lead = (pointer / 94) + 0xA1;
                           let trail = (pointer % 94) + 0xA1;
                           destination_handle.write_two(lead as u8, trail as u8);
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
