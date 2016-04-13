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

pub struct EucKrDecoder {
    lead: u8,
}

impl EucKrDecoder {
    pub fn new() -> VariantDecoder {
        VariantDecoder::EucKr(EucKrDecoder { lead: 0 })
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

    pub fn max_buffer_length_from_utf16(&self, u16_length: usize) -> usize {
        0 // TODO
    }

    pub fn max_buffer_length_from_utf8(&self, byte_length: usize) -> usize {
        0 // TODO
    }

    pub fn max_buffer_length_from_utf16_with_replacement_if_no_unmappables(&self,
                                                                           u16_length: usize)
                                                                           -> usize {
        0 // TODO
    }

    pub fn max_buffer_length_from_utf8_with_replacement_if_no_unmappables(&self,
                                                                          byte_length: usize)
                                                                          -> usize {
        0 // TODO
    }

    pub fn encode_from_utf16(&mut self,
                             src: &[u16],
                             dst: &mut [u8],
                             last: bool)
                             -> (EncoderResult, usize, usize) {
        // XXX
        (EncoderResult::InputEmpty, 0, 0)
    }

    pub fn encode_from_utf8(&mut self,
                            src: &str,
                            dst: &mut [u8],
                            last: bool)
                            -> (EncoderResult, usize, usize) {
        // XXX
        (EncoderResult::InputEmpty, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;

}
