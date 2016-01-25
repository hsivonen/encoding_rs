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

pub struct ReplacementDecoder;

impl ReplacementDecoder {
    pub fn new(encoding: &'static Encoding) -> Decoder {
        Decoder::new(encoding, VariantDecoder::Replacement(ReplacementDecoder))
    }

    pub fn reset(&mut self) {}

    pub fn max_utf16_buffer_length(&self, byte_length: usize) -> usize {
        byte_length
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
                           } else {
                               return (DecoderResult::Malformed(1),
                                       unread_handle.consumed(),
                                       destination_handle.written());

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


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;

}
