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

pub struct Utf8Decoder;

impl Utf8Decoder {
    pub fn new(encoding: &'static Encoding) -> Decoder {
        Decoder::new(encoding, VariantDecoder::Utf8(Utf8Decoder))
    }

    pub fn reset(&mut self) {}

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

pub struct Utf8Encoder;

impl Utf8Encoder {
    pub fn new(encoding: &'static Encoding) -> Encoder {
        Encoder::new(encoding, VariantEncoder::Utf8(Utf8Encoder))
    }

    pub fn max_buffer_length_from_utf16(&self, u16_length: usize) -> usize {
        3 * u16_length
    }

    pub fn max_buffer_length_from_utf8(&self, byte_length: usize) -> usize {
        byte_length
    }

    pub fn encode_from_utf16(&mut self,
                             src: &[u16],
                             dst: &mut [u8],
                             _last: bool)
                             -> (EncoderResult, usize, usize) {
        let mut source = Utf16Source::new(src);
        let mut dest = Utf8Destination::new(dst);
        loop {
            match source.check_available() {
                Space::Full(src_consumed) => {
                    return (EncoderResult::InputEmpty, src_consumed, dest.written());
                }
                Space::Available(source_handle) => {
                    match dest.check_space_astral() {
                        Space::Full(dst_written) => {
                            return (EncoderResult::OutputFull,
                                    source_handle.consumed(),
                                    dst_written);
                        }
                        Space::Available(destination_handle) => {
                            let (c, _) = source_handle.read();
                            // Start non-boilerplate
                            destination_handle.write_char(c);
                            // End non-boilerplate
                        }
                    }
                }
            }
        }
    }

    pub fn encode_from_utf8(&mut self,
                            src: &str,
                            dst: &mut [u8],
                            _last: bool)
                            -> (EncoderResult, usize, usize) {
        let mut to_write = src.len();
        if to_write <= dst.len() {
            unsafe {
                ::std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), to_write);
            }
            return (EncoderResult::InputEmpty, to_write, to_write);
        }
        to_write = dst.len();
        // Move back until we find a UTF-8 sequence boundary.
        let bytes = src.as_bytes();
        while (bytes[to_write] & 0xC0) == 0x80 {
            to_write -= 1;
        }
        unsafe {
            ::std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), to_write);
        }
        return (EncoderResult::OutputFull, to_write, to_write);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;

}
