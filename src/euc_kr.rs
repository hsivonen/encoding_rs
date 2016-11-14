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

pub struct EucKrDecoder {
    lead: u8,
}

impl EucKrDecoder {
    pub fn new() -> VariantDecoder {
        VariantDecoder::EucKr(EucKrDecoder { lead: 0 })
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

    pub fn max_utf8_buffer_length_without_replacement(&self, byte_length: usize) -> usize {
        // worst case: 2 to 3
        let len = self.plus_one_if_lead(byte_length);
        len + (len + 1) / 2
    }

    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        self.plus_one_if_lead(byte_length) * 3
    }

    pub fn decode_to_utf8_raw(&mut self,
                              src: &[u8],
                              dst: &mut [u8],
                              last: bool)
                              -> (DecoderResult, usize, usize) {
        let mut source = ByteSource::new(src);
        let mut dest_prolog = Utf8Destination::new(dst);
        let dest = if self.lead != 0 {
            let lead_minus_offset = self.lead;
            // Since we don't have `goto` we could use to jump into the trail
            // handling part of the main loop, we need to repeat trail handling
            // here.
            match source.check_available() {
                Space::Full(src_consumed_prolog) => {
                    if last {
                        return (DecoderResult::Malformed(1, 0),
                                src_consumed_prolog,
                                dest_prolog.written());
                    }
                    return (DecoderResult::InputEmpty, src_consumed_prolog, dest_prolog.written());
                }
                Space::Available(source_handle_prolog) => {
                    match dest_prolog.check_space_bmp() {
                        Space::Full(dst_written_prolog) => {
                            return (DecoderResult::OutputFull,
                                    source_handle_prolog.consumed(),
                                    dst_written_prolog);
                        }
                        Space::Available(handle) => {
                            let (byte, unread_handle_trail) = source_handle_prolog.read();
                            // Start non-boilerplate
                            // If trail is between 0x41 and 0xFE, inclusive,
                            // subtract offset 0x41.
                            let trail_minus_offset = byte.wrapping_sub(0x41);
                            if trail_minus_offset > (0xFE - 0x41) {
                                if byte <= 0x7F {
                                    return (DecoderResult::Malformed(1, 0),
                                            unread_handle_trail.unread(),
                                            handle.written());
                                }
                                return (DecoderResult::Malformed(2, 0),
                                        unread_handle_trail.consumed(),
                                        handle.written());
                            }
                            let pointer = lead_minus_offset as usize * 190usize +
                                          trail_minus_offset as usize;
                            let bmp = euc_kr_decode(pointer);
                            if bmp == 0 {
                                if byte <= 0x7F {
                                    return (DecoderResult::Malformed(1, 0),
                                            unread_handle_trail.unread(),
                                            handle.written());
                                }
                                return (DecoderResult::Malformed(2, 0),
                                        unread_handle_trail.consumed(),
                                        handle.written());
                            }
                            handle.write_bmp_excl_ascii(bmp)
                            // End non-boilerplate
                        }
                    }
                }
            }
        } else {
            &mut dest_prolog
        };
        'outermost: loop {
            match dest.copy_ascii_from_check_space_bmp(&mut source) {
                CopyAsciiResult::Stop(ret) => return ret,
                CopyAsciiResult::GoOn((mut non_ascii, mut handle)) => {
                    'middle: loop {
                        let dest_again = {
                            let lead_minus_offset = {
                                // Start non-boilerplate
                                // If lead is between 0x81 and 0xFE, inclusive,
                                // subtract offset 0x81.
                                let non_ascii_minus_offset = non_ascii.wrapping_sub(0x81);
                                if non_ascii_minus_offset > (0xFE - 0x81) {
                                    return (DecoderResult::Malformed(1, 0),
                                            source.consumed(),
                                            handle.written());
                                }
                                non_ascii_minus_offset
                                // End non-boilerplate
                            };
                            match source.check_available() {
                                Space::Full(src_consumed_trail) => {
                                    if last {
                                        return (DecoderResult::Malformed(1, 0),
                                                src_consumed_trail,
                                                handle.written());
                                    }
                                    self.lead = lead_minus_offset;
                                    return (DecoderResult::InputEmpty,
                                            src_consumed_trail,
                                            handle.written());
                                }
                                Space::Available(source_handle_trail) => {
                                    let (byte, unread_handle_trail) = source_handle_trail.read();
                                    // Start non-boilerplate
                                    // If trail is between 0x41 and 0xFE, inclusive,
                                    // subtract offset 0x41.
                                    let trail_minus_offset = byte.wrapping_sub(0x41);
                                    if trail_minus_offset > (0xFE - 0x41) {
                                        if byte <= 0x7F {
                                            return (DecoderResult::Malformed(1, 0),
                                                    unread_handle_trail.unread(),
                                                    handle.written());
                                        }
                                        return (DecoderResult::Malformed(2, 0),
                                                unread_handle_trail.consumed(),
                                                handle.written());
                                    }
                                    let pointer = lead_minus_offset as usize * 190usize +
                                                  trail_minus_offset as usize;
                                    let bmp = euc_kr_decode(pointer);
                                    if bmp == 0 {
                                        if byte <= 0x7F {
                                            return (DecoderResult::Malformed(1, 0),
                                                    unread_handle_trail.unread(),
                                                    handle.written());
                                        }
                                        return (DecoderResult::Malformed(2, 0),
                                                unread_handle_trail.consumed(),
                                                handle.written());
                                    }
                                    handle.write_bmp_excl_ascii(bmp)
                                    // End non-boilerplate
                                }
                            }
                        };
                        match source.check_available() {
                            Space::Full(src_consumed) => {
                                return (DecoderResult::InputEmpty,
                                        src_consumed,
                                        dest_again.written());
                            }
                            Space::Available(source_handle) => {
                                match dest_again.check_space_bmp() {
                                    Space::Full(dst_written) => {
                                        return (DecoderResult::OutputFull,
                                                source_handle.consumed(),
                                                dst_written);
                                    }
                                    Space::Available(mut destination_handle) => {
                                        let (mut b, unread_handle) = source_handle.read();
                                        let source_again = unread_handle.decommit();
                                        'innermost: loop {
                                            if b > 127 {
                                                non_ascii = b;
                                                handle = destination_handle;
                                                continue 'middle;
                                            }
                                            // Testing on Haswell says that we should write the
                                            // byte unconditionally instead of trying to unread it
                                            // to make it part of the next SIMD stride.
                                            let dest_again_again =
                                                destination_handle.write_ascii(b);
                                            if b < 60 {
                                                // We've got punctuation
                                                match source_again.check_available() {
                                                    Space::Full(src_consumed_again) => {
                                                        return (DecoderResult::InputEmpty,
                                                                src_consumed_again,
                                                                dest_again_again.written());
                                                    }
                                                    Space::Available(source_handle_again) => {
                                                        match dest_again_again.check_space_bmp() {
                                                            Space::Full(dst_written_again) => {
                                                                return (DecoderResult::OutputFull,
                                                                        source_handle_again.consumed(),
                                                                        dst_written_again);
                                                            }
                                                            Space::Available(destination_handle_again) => {
                                                                {
                                                                    let (b_again, _unread_handle_again) =
                                                                        source_handle_again.read();
                                                                    b = b_again;
                                                                    destination_handle = destination_handle_again;
                                                                    continue 'innermost;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            // We've got markup or ASCII text
                                            continue 'outermost;
                                        }
                                    }
                                }
                            }
                        }
                        unreachable!("Should always continue earlier.");
                    }
                }
            }
            unreachable!("Should always continue earlier.");
        }
    }

    decoder_function!({},
                      {
                          if self.lead != 0 {
                              self.lead = 0;
                              return (DecoderResult::Malformed(1, 0), src_consumed, dest.written());
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
                              let pointer = (lead as usize - 0x81) * 190usize + (b as usize - 0x41);
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
                      check_space_bmp,
                      decode_to_utf16_raw,
                      u16,
                      Utf16Destination);
}

pub struct EucKrEncoder;

impl EucKrEncoder {
    pub fn new(encoding: &'static Encoding) -> Encoder {
        Encoder::new(encoding, VariantEncoder::EucKr(EucKrEncoder))
    }

    pub fn max_buffer_length_from_utf16_without_replacement(&self, u16_length: usize) -> usize {
        u16_length * 2
    }

    pub fn max_buffer_length_from_utf8_without_replacement(&self, byte_length: usize) -> usize {
        byte_length
    }

    ascii_compatible_bmp_encoder_functions!({
                                                let pointer = euc_kr_encode(bmp);
                                                if pointer == usize::max_value() {
                                                    return (EncoderResult::unmappable_from_bmp(bmp),
                                                            source.consumed(),
                                                            handle.written());
                                                }
                                                let lead = (pointer / 190) + 0x81;
                                                let trail = (pointer % 190) + 0x41;
                                                handle.write_two(lead as u8, trail as u8)
                                            },
                                            bmp,
                                            self,
                                            source,
                                            handle,
                                            copy_ascii_to_check_space_two,
                                            check_space_two,
                                            true);
}

// Any copyright to the test code below this comment is dedicated to the
// Public Domain. http://creativecommons.org/publicdomain/zero/1.0/

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::*;

    fn decode_euc_kr(bytes: &[u8], expect: &str) {
        decode(EUC_KR, bytes, expect);
    }

    fn encode_euc_kr(string: &str, expect: &[u8]) {
        encode(EUC_KR, string, expect);
    }

    #[test]
    fn test_euc_kr_decode() {
        // Empty
        decode_euc_kr(b"", &"");

        // ASCII
        decode_euc_kr(b"\x61\x62", "\u{0061}\u{0062}");

        decode_euc_kr(b"\x81\x41", "\u{AC02}");
        decode_euc_kr(b"\x81\x5B", "\u{FFFD}\x5B");
        decode_euc_kr(b"\xFD\xFE", "\u{8A70}");
        decode_euc_kr(b"\xFE\x41", "\u{FFFD}\x41");
        decode_euc_kr(b"\xFF\x41", "\u{FFFD}\x41");
        decode_euc_kr(b"\x80\x41", "\u{FFFD}\x41");
    }

    #[test]
    fn test_euc_kr_encode() {
        // Empty
        encode_euc_kr("", b"");

        // ASCII
        encode_euc_kr("\u{0061}\u{0062}", b"\x61\x62");

        encode_euc_kr("\u{AC02}", b"\x81\x41");
        encode_euc_kr("\u{8A70}", b"\xFD\xFE");
    }

}
