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

pub struct Utf16Decoder {
    lead_surrogate: u16, // If non-zero and pending_bmp == false, a pending lead surrogate
    lead_byte: Option<u8>,
    be: bool,
    pending_bmp: bool, // if true, lead_surrogate is actually pending BMP
}

impl Utf16Decoder {
    pub fn new(big_endian: bool) -> VariantDecoder {
        VariantDecoder::Utf16(Utf16Decoder {
            lead_surrogate: 0,
            lead_byte: None,
            be: big_endian,
            pending_bmp: false,
        })
    }

    pub fn max_utf16_buffer_length(&self, byte_length: usize) -> usize {
        (byte_length + 1) / 2
    }

    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        (byte_length / 2) * 3
    }

    pub fn max_utf8_buffer_length_with_replacement(&self, byte_length: usize) -> usize {
        ((byte_length + 1) / 2) * 3
    }

    decoder_functions!({
                           if self.pending_bmp {
                               match dest.check_space_bmp() {
                                   Space::Full(_) => {
                                       return (DecoderResult::OutputFull, 0, 0);
                                   }
                                   Space::Available(destination_handle) => {
                                       destination_handle.write_bmp(self.lead_surrogate);
                                       self.pending_bmp = false;
                                       self.lead_surrogate = 0;
                                   }
                               }
                           }
                       },
                       {
                           debug_assert!(!self.pending_bmp);
                           if self.lead_surrogate != 0 {
                               self.lead_surrogate = 0;
                               match self.lead_byte {
                                   None => {
                                       return (DecoderResult::Malformed(2, 0),
                                               src_consumed,
                                               dest.written());
                                   }
                                   Some(_) => {
                                       return (DecoderResult::Malformed(3, 0),
                                               src_consumed,
                                               dest.written());
                                   }
                               }
                           }
                           match self.lead_byte {
                               None => {}
                               Some(_) => {
                                   return (DecoderResult::Malformed(1, 0),
                                           src_consumed,
                                           dest.written());
                               }
                           }
                       },
                       {
                           match self.lead_byte {
                               None => {
                                   self.lead_byte = Some(b);
                                   continue;
                               }
                               Some(lead) => {
                                   let code_unit = if self.be {
                                       (lead as u16) << 8 | b as u16
                                   } else {
                                       (b as u16) << 8 | (lead as u16)
                                   };
                                   let high_bits = code_unit & 0xFC00u16;
                                   if high_bits == 0xD800u16 {
                                       // high surrogate
                                       if self.lead_surrogate != 0 {
                                           // The previous high surrogate was in
                                           // error and this one becomes the new
                                           // pending one.
                                           self.lead_surrogate = code_unit as u16;
                                           return (DecoderResult::Malformed(2, 2),
                                                   unread_handle.consumed(),
                                                   destination_handle.written());
                                       }
                                       self.lead_surrogate = code_unit;
                                       continue;
                                   }
                                   if high_bits == 0xDC00u16 {
                                       // low surrogate
                                       if self.lead_surrogate == 0 {
                                           return (DecoderResult::Malformed(2, 0),
                                                   unread_handle.consumed(),
                                                   destination_handle.written());
                                       }
                                       destination_handle.write_surrogate_pair(self.lead_surrogate,
                                                                               code_unit);
                                       self.lead_surrogate = 0;
                                       continue;
                                   }
                                   // bmp
                                   if self.lead_surrogate != 0 {
                                       // The previous high surrogate was in
                                       // error and this code unit becomes a
                                       // pending BMP character.
                                       self.lead_surrogate = code_unit;
                                       self.pending_bmp = true;
                                       return (DecoderResult::Malformed(2, 2),
                                               unread_handle.consumed(),
                                               destination_handle.written());
                                   }
                                   destination_handle.write_bmp(code_unit);
                                   continue;
                               }
                           }
                       },
                       self,
                       src_consumed,
                       dest,
                       b,
                       destination_handle,
                       unread_handle,
                       check_space_astral);
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;

}
