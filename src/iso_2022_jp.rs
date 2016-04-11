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

#[derive(Copy,Clone)]
enum Iso2022JpDecoderState {
    Ascii,
    Roman,
    Katakana,
    LeadByte,
    TrailByte,
    EscapeStart,
    Escape,
}

pub struct Iso2022JpDecoder {
    decoder_state: Iso2022JpDecoderState,
    output_state: Iso2022JpDecoderState, // only takes 1 of first 4 values
    lead: u8,
    output_flag: bool,
    pending_prepended: bool,
}

impl Iso2022JpDecoder {
    pub fn new() -> VariantDecoder {
        VariantDecoder::Iso2022Jp(Iso2022JpDecoder {
            decoder_state: Iso2022JpDecoderState::Ascii,
            output_state: Iso2022JpDecoderState::Ascii,
            lead: 0u8,
            output_flag: false,
            pending_prepended: false,
        })
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

    decoder_functions!({
                           if self.pending_prepended {
                               // lead was set in EscapeStart and "prepended"
                               // in Escape.
                               debug_assert!(self.lead == 0x24u8 || self.lead == 0x28u8);
                               match dest.check_space_bmp() {
                                   Space::Full(_) => {
                                       return (DecoderResult::OutputFull, 0, 0);
                                   }
                                   Space::Available(destination_handle) => {
                                       self.pending_prepended = false;
                                       self.output_flag = false;
                                       match self.decoder_state {
                                           Iso2022JpDecoderState::Ascii |
                                           Iso2022JpDecoderState::Roman => {
                                               destination_handle.write_ascii(self.lead);
                                               self.lead = 0x0u8;
                                           }
                                           Iso2022JpDecoderState::Katakana => {
                                               destination_handle.write_upper_bmp(self.lead as u16 -
                                                                                  0x21u16 +
                                                                                  0xFF61u16);
                                               self.lead = 0x0u8;
                                           }
                                           Iso2022JpDecoderState::LeadByte => {
                                               self.decoder_state =
                                                   Iso2022JpDecoderState::TrailByte;
                                           }
                                           _ => unreachable!(),
                                       }
                                   }
                               }
                           }
                       },
                       {
                           match self.decoder_state {
                               Iso2022JpDecoderState::TrailByte |
                               Iso2022JpDecoderState::EscapeStart => {
                                   self.decoder_state = self.output_state;
                                   return (DecoderResult::Malformed(1, 0),
                                           src_consumed,
                                           dest.written());
                               }
                               Iso2022JpDecoderState::Escape => {
                                   self.pending_prepended = true;
                                   self.decoder_state = self.output_state;
                                   return (DecoderResult::Malformed(1, 1),
                                           src_consumed,
                                           dest.written());
                               }
                               _ => {}
                           }
                       },
                       {
                           match self.decoder_state {
                               Iso2022JpDecoderState::Ascii => {
                                   if b == 0x1Bu8 {
                                       self.decoder_state = Iso2022JpDecoderState::EscapeStart;
                                       continue;
                                   }
                                   self.output_flag = false;
                                   if b > 0x7Eu8 || b == 0x0Eu8 || b == 0x0Fu8 {
                                       return (DecoderResult::Malformed(1, 0),
                                               unread_handle.consumed(),
                                               destination_handle.written());
                                   }
                                   destination_handle.write_ascii(b);
                                   continue;
                               }
                               Iso2022JpDecoderState::Roman => {
                                   if b == 0x1Bu8 {
                                       self.decoder_state = Iso2022JpDecoderState::EscapeStart;
                                       continue;
                                   }
                                   self.output_flag = false;
                                   if b == 0x5Cu8 {
                                       destination_handle.write_mid_bmp(0x00A5u16);
                                       continue;
                                   }
                                   if b == 0x7Eu8 {
                                       destination_handle.write_upper_bmp(0x203Eu16);
                                       continue;
                                   }
                                   if b > 0x7Eu8 || b == 0x0Eu8 || b == 0x0Fu8 {
                                       return (DecoderResult::Malformed(1, 0),
                                               unread_handle.consumed(),
                                               destination_handle.written());
                                   }
                                   destination_handle.write_ascii(b);
                                   continue;
                               }
                               Iso2022JpDecoderState::Katakana => {
                                   if b == 0x1Bu8 {
                                       self.decoder_state = Iso2022JpDecoderState::EscapeStart;
                                       continue;
                                   }
                                   self.output_flag = false;
                                   if b >= 0x21u8 && b <= 0x5Fu8 {
                                       destination_handle.write_upper_bmp(b as u16 - 0x21u16 +
                                                                          0xFF61u16);
                                       continue;
                                   }
                                   return (DecoderResult::Malformed(1, 0),
                                           unread_handle.consumed(),
                                           destination_handle.written());
                               }
                               Iso2022JpDecoderState::LeadByte => {
                                   if b == 0x1Bu8 {
                                       self.decoder_state = Iso2022JpDecoderState::EscapeStart;
                                       continue;
                                   }
                                   self.output_flag = false;
                                   if b >= 0x21u8 && b <= 0x7Eu8 {
                                       self.lead = b;
                                       self.decoder_state = Iso2022JpDecoderState::TrailByte;
                                       continue;
                                   }
                                   return (DecoderResult::Malformed(1, 0),
                                           unread_handle.consumed(),
                                           destination_handle.written());
                               }
                               Iso2022JpDecoderState::TrailByte => {
                                   if b == 0x1Bu8 {
                                       self.decoder_state = Iso2022JpDecoderState::EscapeStart;
                                       // The byte in error is the previous
                                       // lead byte.
                                       return (DecoderResult::Malformed(1, 1),
                                               unread_handle.consumed(),
                                               destination_handle.written());
                                   }
                                   if b >= 0x21u8 && b <= 0x7Eu8 {
                                       self.decoder_state = Iso2022JpDecoderState::LeadByte;
                                       let pointer = (self.lead as usize - 0x21usize) * 94usize +
                                                     b as usize -
                                                     0x21usize;
                                       let c = jis0208_decode(pointer);
                                       if c == 0 {
                                           return (DecoderResult::Malformed(2, 0),
                                                   unread_handle.consumed(),
                                                   destination_handle.written());
                                       }
                                       destination_handle.write_bmp_excl_ascii(c);
                                       continue;
                                   }
                                   self.decoder_state = Iso2022JpDecoderState::LeadByte;
                                   return (DecoderResult::Malformed(2, 0),
                                           unread_handle.consumed(),
                                           destination_handle.written());
                               }
                               Iso2022JpDecoderState::EscapeStart => {
                                   if b == 0x24u8 || b == 0x28u8 {
                                       self.lead = b;
                                       self.decoder_state = Iso2022JpDecoderState::Escape;
                                       continue;
                                   }
                                   self.output_flag = false;
                                   self.decoder_state = self.output_state;
                                   return (DecoderResult::Malformed(1, 0),
                                           unread_handle.unread(),
                                           destination_handle.written());
                               }
                               Iso2022JpDecoderState::Escape => {
                                   let mut state: Option<Iso2022JpDecoderState> = None;
                                   if self.lead == 0x28u8 && b == 0x42u8 {
                                       state = Some(Iso2022JpDecoderState::Ascii);
                                   } else if self.lead == 0x28u8 && b == 0x4Au8 {
                                       state = Some(Iso2022JpDecoderState::Roman);
                                   } else if self.lead == 0x28u8 && b == 0x49u8 {
                                       state = Some(Iso2022JpDecoderState::Katakana);
                                   } else if self.lead == 0x24u8 && (b == 0x40u8 || b == 0x42u8) {
                                       state = Some(Iso2022JpDecoderState::LeadByte);
                                   }
                                   match state {
                                       Some(s) => {
                                           self.lead = 0x0u8;
                                           self.decoder_state = s;
                                           self.output_state = s;
                                           let flag = self.output_flag;
                                           self.output_flag = true;
                                           if flag {
                                               // We had an escape sequence
                                               // immediately following another
                                               // escape sequence. Therefore,
                                               // the first one of these was
                                               // useless.
                                               return (DecoderResult::Malformed(3, 3),
                                                       unread_handle.consumed(),
                                                       destination_handle.written());
                                           }
                                           continue;
                                       }
                                       None => {
                                           // self.lead is still the previous
                                           // byte. It will be processed in
                                           // the preabmle upon next call.
                                           self.pending_prepended = true;
                                           self.output_flag = false;
                                           self.decoder_state = self.output_state;
                                           // The byte in error is not the
                                           // current or the previous byte but
                                           // the one before those (lone 0x1B).
                                           return (DecoderResult::Malformed(1, 1),
                                                   unread_handle.unread(),
                                                   destination_handle.written());
                                       }
                                   }
                               }
                           }
                       },
                       self,
                       src_consumed,
                       dest,
                       b,
                       destination_handle,
                       unread_handle,
                       check_space_bmp);
}

pub struct Iso2022JpEncoder;

impl Iso2022JpEncoder {
    pub fn new(encoding: &'static Encoding) -> Encoder {
        Encoder::new(encoding, VariantEncoder::Iso2022Jp(Iso2022JpEncoder))
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
