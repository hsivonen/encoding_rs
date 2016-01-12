// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use Decoder;
use DecoderResult;
use handles::*;
use data::*;

pub struct Big5Decoder {
    lead: u8,
}

impl Big5Decoder {
    pub fn new() -> Big5Decoder {
        Big5Decoder { lead: 0 }
    }

    fn plus_one_if_lead(&self, byte_length: usize) -> usize {
        byte_length +
        if self.lead == 0 {
            0
        } else {
            1
        }
    }
}

impl Decoder for Big5Decoder {
    fn reset(&mut self) {
        self.lead = 0u8;
    }

    fn max_utf16_buffer_length(&self, byte_length: usize) -> usize {
        self.plus_one_if_lead(byte_length) + 1
    }

    fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        let len = self.plus_one_if_lead(byte_length);
        (len * 2) + (len / 2) // XXX tail
    }

    fn max_utf8_buffer_length_with_replacement(&self, byte_length: usize) -> usize {
        3 * self.plus_one_if_lead(byte_length) // XXX tail
    }

    fn decode_to_utf16(&mut self,
                       src: &[u8],
                       dst: &mut [u16],
                       last: bool)
                       -> (DecoderResult, usize, usize) {
        let mut source = ByteSource::new(src);
        let mut dest = Utf16Destination::new(dst);
        loop {
            {
                // Start non-boilerplate
                // TODO: ISO-2022-JP
                // End non-boilerplate
            }
            loop {
                match source.check_available() {
                    Space::Full(src_consumed) => {
                        if last {
                            // Start non-boilerplate
                            if self.lead != 0 {
                                self.lead = 0;
                                return (DecoderResult::Malformed(1), src_consumed, dest.written());
                            }
                            // End non-boilerplate
                        }
                        return (DecoderResult::InputEmpty, src_consumed, dest.written());
                    }
                    Space::Available(source_handle) => {
                        match dest.check_space_big5() {
                            Space::Full(dst_written) => {
                                return (DecoderResult::OutputFull,
                                        source_handle.consumed(),
                                        dst_written);
                            }
                            Space::Available(destination_handle) => {
                                let (b, unread_handle) = source_handle.read();
                                // Start non-boilerplate
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
                                    return (DecoderResult::Malformed(1),
                                            unread_handle.consumed(),
                                            destination_handle.written());
                                }
                                let lead = self.lead as usize;
                                self.lead = 0;
                                let offset = if b < 0x7F {
                                    0x40usize
                                } else {
                                    0x62usize
                                };
                                if (b >= 0x40 && b <= 0x7E) || (b >= 0xA1 && b <= 0xFE) {
                                    let pointer = (lead - 0x81usize) * 157usize +
                                                  (b as usize - offset);
                                    match pointer {
                                        1133 => {
                                            destination_handle.write_big5_combination(0x00CAu16,
                                                                                      0x0304u16);
                                            continue;
                                        }
                                        1135 => {
                                            destination_handle.write_big5_combination(0x00CAu16,
                                                                                      0x030Cu16);
                                            continue;
                                        }
                                        1164 => {
                                            destination_handle.write_big5_combination(0x00EAu16,
                                                                                      0x0304u16);
                                            continue;
                                        }
                                        1166 => {
                                            destination_handle.write_big5_combination(0x00EAu16,
                                                                                      0x030Cu16);
                                            continue;
                                        }
                                        _ => {
                                            let low_bits = big5_low_bits(pointer);
                                            if low_bits == 0 {
                                                if b <= 0x7F {
                                                    return (DecoderResult::Malformed(1),
                                                            unread_handle.unread(),
                                                            destination_handle.written());
                                                }
                                                return (DecoderResult::Malformed(2),
                                                        unread_handle.consumed(),
                                                        destination_handle.written());
                                            }
                                            if big5_is_astral(pointer) {
                                                destination_handle.write_astral(low_bits as u32 |
                                                                                0x20000u32);
                                                continue;
                                            }
                                            destination_handle.write_bmp_excl_ascii(low_bits);
                                            continue;
                                        }
                                    }
                                }
                                // pointer is null
                                if b <= 0x7F {
                                    return (DecoderResult::Malformed(1),
                                            unread_handle.unread(),
                                            destination_handle.written());
                                }
                                return (DecoderResult::Malformed(2),
                                        unread_handle.consumed(),
                                        destination_handle.written());

                                // End non-boilerplate
                            }
                        }
                    }
                }
            }
        }
    }

    fn decode_to_utf8(&mut self,
                      src: &[u8],
                      dst: &mut [u8],
                      last: bool)
                      -> (DecoderResult, usize, usize) {
        (DecoderResult::OutputFull, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;

    fn decode_big5_to_utf16(bytes: &[u8], expect: &[u16]) {
        let mut decoder = Big5Decoder::new();
        let mut dest: Vec<u16> = Vec::with_capacity(decoder.max_utf16_buffer_length(expect.len()));
        let capacity = dest.capacity();
        dest.resize(capacity, 0u16);
        let (complete, read, written, _) = decoder.decode_to_utf16_with_replacement(bytes,
                                                                                    &mut dest,
                                                                                    true);
        println!("FOO");
        println!("complete: {}, read: {}, written: {}",
                 complete.as_u32(),
                 read,
                 written);
        assert_eq!(complete, WithReplacementResult::InputEmpty);
        assert_eq!(read, bytes.len());
        assert_eq!(written, expect.len());
        dest.truncate(written);
        assert_eq!(&dest[..], expect);
    }

    #[test]
    fn test_big5_decode() {
        decode_big5_to_utf16(&[0x61u8, 0x62u8], &[0x0061u16, 0x0062u16]);
    }
}
