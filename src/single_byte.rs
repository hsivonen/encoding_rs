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

pub struct SingleByteDecoder {
    table: &'static [u16; 128],
}

impl SingleByteDecoder {
    pub fn new(data: &'static [u16; 128]) -> VariantDecoder {
        VariantDecoder::SingleByte(SingleByteDecoder { table: data })
    }

    pub fn max_utf16_buffer_length(&self, u16_length: usize) -> usize {
        u16_length
    }

    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        byte_length * 3
    }

    pub fn max_utf8_buffer_length_with_replacement(&self, byte_length: usize) -> usize {
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

    pub fn max_buffer_length_from_utf16(&self, u16_length: usize) -> usize {
        u16_length
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
