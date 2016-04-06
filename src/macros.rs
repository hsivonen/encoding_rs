// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

macro_rules! decoder_function {
    ($preamble:block,
     $eof:block,
     $body:block,
     $slf:ident,
     $src_consumed:ident,
     $dest:ident,
     $b:ident,
     $destination_handle:ident,
     $unread_handle:ident,
     $destination_check:ident,
     $name:ident,
     $code_unit:ty,
     $dest_struct:ident) => (
    pub fn $name(&mut $slf,
                 src: &[u8],
                 dst: &mut [$code_unit],
                 last: bool)
                 -> (DecoderResult, usize, usize) {
        let mut source = ByteSource::new(src);
        let mut $dest = $dest_struct::new(dst);
        loop { // TODO: remove this loop
            {
                // Start non-boilerplate
                $preamble
                // End non-boilerplate
            }
            loop {
                match source.check_available() {
                    Space::Full($src_consumed) => {
                        if last {
                            // Start non-boilerplate
                            $eof
                            // End non-boilerplate
                        }
                        return (DecoderResult::InputEmpty, $src_consumed, $dest.written());
                    }
                    Space::Available(source_handle) => {
                        match $dest.$destination_check() {
                            Space::Full(dst_written) => {
                                return (DecoderResult::OutputFull,
                                        source_handle.consumed(),
                                        dst_written);
                            }
                            Space::Available($destination_handle) => {
                                let ($b, $unread_handle) = source_handle.read();
                                // Start non-boilerplate
                                $body
                                // End non-boilerplate
                            }
                        }
                    }
                }
            }
        }
    });
}

macro_rules! decoder_functions {
    ($preamble:block,
     $eof:block,
     $body:block,
     $slf:ident,
     $src_consumed:ident,
     $dest:ident,
     $b:ident,
     $destination_handle:ident,
     $unread_handle:ident,
     $destination_check:ident) => (
    decoder_function!($preamble,
                      $eof,
                      $body,
                      $slf,
                      $src_consumed,
                      $dest,
                      $b,
                      $destination_handle,
                      $unread_handle,
                      $destination_check,
                      decode_to_utf8,
                      u8,
                      Utf8Destination);
    decoder_function!($preamble,
                      $eof,
                      $body,
                      $slf,
                      $src_consumed,
                      $dest,
                      $b,
                      $destination_handle,
                      $unread_handle,
                      $destination_check,
                      decode_to_utf16,
                      u16,
                      Utf16Destination);
    );
}

macro_rules! encoder_function {
    ($eof:block,
     $body:block,
     $slf:ident,
     $src_consumed:ident,
     $source:ident,
     $c:ident,
     $destination_handle:ident,
     $unread_handle:ident,
     $destination_check:ident,
     $name:ident,
     $input:ty,
     $source_struct:ident) => (
    pub fn $name(&mut $slf,
                 src: &$input,
                 dst: &mut [u8],
                 last: bool)
                 -> (EncoderResult, usize, usize) {
        let mut $source = $source_struct::new(src);
        let mut dest = ByteDestination::new(dst);
        loop {
            match $source.check_available() {
                Space::Full($src_consumed) => {
                    if last {
                        // Start non-boilerplate
                        $eof
                        // End non-boilerplate
                    }
                    return (EncoderResult::InputEmpty, $src_consumed, dest.written());
                }
                Space::Available(source_handle) => {
                    match dest.$destination_check() {
                        Space::Full(dst_written) => {
                            return (EncoderResult::OutputFull,
                                    source_handle.consumed(),
                                    dst_written);
                        }
                        Space::Available($destination_handle) => {
                            let ($c, $unread_handle) = source_handle.read();
                            // Start non-boilerplate
                            $body
                            // End non-boilerplate
                        }
                    }
                }
            }
        }
    });
}

macro_rules! encoder_functions {
    ($eof:block,
     $body:block,
     $slf:ident,
     $src_consumed:ident,
     $source:ident,
     $c:ident,
     $destination_handle:ident,
     $unread_handle:ident,
     $destination_check:ident) => (
    encoder_function!($eof,
                      $body,
                      $slf,
                      $src_consumed,
                      $source,
                      $c,
                      $destination_handle,
                      $unread_handle,
                      $destination_check,
                      encode_from_utf8,
                      str,
                      Utf8Source);
    encoder_function!($eof,
                      $body,
                      $slf,
                      $src_consumed,
                      $source,
                      $c,
                      $destination_handle,
                      $unread_handle,
                      $destination_check,
                      encode_from_utf16,
                      [u16],
                      Utf16Source);
    );
}

macro_rules! public_decode_function{
    ($decode_to_utf:ident,
     $decode_to_utf_checking_end:ident,
     $decode_to_utf_after_one_potential_bom_byte:ident,
     $decode_to_utf_after_two_potential_bom_bytes:ident,
     $decode_to_utf_checking_end_with_offset:ident,
     $code_unit:ty) => (
    pub fn $decode_to_utf(&mut self,
                           src: &[u8],
                           dst: &mut [$code_unit],
                           last: bool)
                           -> (DecoderResult, usize, usize) {
        let mut offset = 0usize;
        loop {
            match self.life_cycle {
                // The common case. (Post-sniffing.)
                DecoderLifeCycle::Converting => {
                    return self.$decode_to_utf_checking_end(src, dst, last);
                }
                // The rest is all BOM sniffing!
                DecoderLifeCycle::AtStart => {
                    debug_assert!(offset == 0usize);
                    if src.is_empty() {
                        return (DecoderResult::InputEmpty, 0, 0);
                    }
                    match src[0] {
                        0xEFu8 => {
                            self.life_cycle = DecoderLifeCycle::SeenUtf8First;
                            offset += 1;
                            continue;
                        }
                        0xFEu8 => {
                            self.life_cycle = DecoderLifeCycle::SeenUtf16BeFirst;
                            offset += 1;
                            continue;
                        }
                        0xFFu8 => {
                            self.life_cycle = DecoderLifeCycle::SeenUtf16LeFirst;
                            offset += 1;
                            continue;
                        }
                        _ => {
                            self.life_cycle = DecoderLifeCycle::Converting;
                            continue;
                        }
                    }
                }
                DecoderLifeCycle::SeenUtf8First => {
                    if offset >= src.len() {
                        if last {
                            return self.$decode_to_utf_after_one_potential_bom_byte(src,
                                                                                    dst,
                                                                                    last,
                                                                                    offset,
                                                                                    0xEFu8);
                        }
                        return (DecoderResult::InputEmpty, offset, 0);
                    }
                    if src[offset] == 0xBBu8 {
                        self.life_cycle = DecoderLifeCycle::SeenUtf8Second;
                        offset += 1;
                        continue;
                    }
                    return self.$decode_to_utf_after_one_potential_bom_byte(src,
                                                                            dst,
                                                                            last,
                                                                            offset,
                                                                            0xEFu8);
                }
                DecoderLifeCycle::SeenUtf8Second => {
                    if offset >= src.len() {
                        if last {
                            return self.$decode_to_utf_after_two_potential_bom_bytes(src,
                                                                                     dst,
                                                                                     last,
                                                                                     offset);
                        }
                        return (DecoderResult::InputEmpty, offset, 0);
                    }
                    if src[offset] == 0xBFu8 {
                        self.life_cycle = DecoderLifeCycle::Converting;
                        offset += 1;
                        if self.encoding != UTF_8 {
                            self.encoding = UTF_8;
                            self.variant = UTF_8.new_variant_decoder();
                        }
                        return self.$decode_to_utf_checking_end_with_offset(src,
                                                                            dst,
                                                                            last,
                                                                            offset);
                    }
                    return self.$decode_to_utf_after_two_potential_bom_bytes(src,
                                                                             dst,
                                                                             last,
                                                                             offset);
                }
                DecoderLifeCycle::SeenUtf16BeFirst => {
                    if offset >= src.len() {
                        if last {
                            return self.$decode_to_utf_after_one_potential_bom_byte(src,
                                                                                    dst,
                                                                                    last,
                                                                                    offset,
                                                                                    0xFEu8);
                        }
                        return (DecoderResult::InputEmpty, offset, 0);
                    }
                    if src[offset] == 0xFFu8 {
                        self.life_cycle = DecoderLifeCycle::Converting;
                        offset += 1;
                        if self.encoding != UTF_16BE {
                            self.encoding = UTF_16BE;
                            self.variant = UTF_16BE.new_variant_decoder();
                        }
                        return self.$decode_to_utf_checking_end_with_offset(src,
                                                                            dst,
                                                                            last,
                                                                            offset);
                    }
                    return self.$decode_to_utf_after_one_potential_bom_byte(src,
                                                                            dst,
                                                                            last,
                                                                            offset,
                                                                            0xFEu8);
                }
                DecoderLifeCycle::SeenUtf16LeFirst => {
                    if offset >= src.len() {
                        if last {
                            return self.$decode_to_utf_after_one_potential_bom_byte(src,
                                                                                    dst,
                                                                                    last,
                                                                                    offset,
                                                                                    0xFFu8);
                        }
                        return (DecoderResult::InputEmpty, offset, 0);
                    }
                    if src[offset] == 0xFEu8 {
                        self.life_cycle = DecoderLifeCycle::Converting;
                        offset += 1;
                        if self.encoding != UTF_16LE {
                            self.encoding = UTF_16LE;
                            self.variant = UTF_16LE.new_variant_decoder();
                        }
                        return self.$decode_to_utf_checking_end_with_offset(src,
                                                                            dst,
                                                                            last,
                                                                            offset);
                    }
                    return self.$decode_to_utf_after_one_potential_bom_byte(src,
                                                                            dst,
                                                                            last,
                                                                            offset,
                                                                            0xFFu8);
                }
                DecoderLifeCycle::ConvertingWithPendingBB => {
                    debug_assert!(offset == 0usize);
                    return self.$decode_to_utf_after_one_potential_bom_byte(src,
                                                                            dst,
                                                                            last,
                                                                            0usize,
                                                                            0xBBu8);
                }
                DecoderLifeCycle::Finished => panic!("Must not use a decoder that has finished."),
            }
        }
    }
                           
    fn $decode_to_utf_after_one_potential_bom_byte(&mut self,
                                                   src: &[u8],
                                                   dst: &mut [$code_unit],
                                                   last: bool,
                                                   offset: usize,
                                                   first_byte: u8)
                                                   -> (DecoderResult, usize, usize) {
        self.life_cycle = DecoderLifeCycle::Converting;
        if offset == 0usize {
            // First byte was seen previously.
            let first = [first_byte];
            let (mut first_result, mut first_read, mut first_written) =
                self.variant
                    .$decode_to_utf(&first[..], dst, last);
            match first_result {
                DecoderResult::InputEmpty => {
                    let (result, read, written) =
                        self.$decode_to_utf_checking_end(src, &mut dst[first_written..], last);
                    first_result = result;
                    first_read = read; // Overwrite, don't add!
                    first_written += written;
                }
                DecoderResult::Malformed(_) => {
                    first_read = 0usize; // Wasn't read from `src`!
                }
                DecoderResult::OutputFull => {
                    panic!("Output buffer must have been too small.");
                }
            }
            return (first_result, first_read, first_written);
        }
        debug_assert!(offset == 1usize);
        // The first byte is in `src`, so no need to push it separately.
        return self.$decode_to_utf_checking_end(src, dst, last);
    }

    fn $decode_to_utf_after_two_potential_bom_bytes(&mut self,
                                                    src: &[u8],
                                                    dst: &mut [$code_unit],
                                                    last: bool,
                                                    offset: usize)
                                                    -> (DecoderResult, usize, usize) {
        self.life_cycle = DecoderLifeCycle::Converting;
        if offset == 0usize {
            // The first two bytes are not in the current buffer..
            let ef_bb = [0xEFu8, 0xBBu8];
            let (mut first_result, mut first_read, mut first_written) =
                self.variant
                    .$decode_to_utf(&ef_bb[..], dst, last);
            match first_result {
                DecoderResult::InputEmpty => {
                    let (result, read, written) =
                        self.$decode_to_utf_checking_end(src, &mut dst[first_written..], last);
                    first_result = result;
                    first_read = read; // Overwrite, don't add!
                    first_written += written;
                }
                DecoderResult::Malformed(_) => {
                    if first_read == 1usize {
                        // The first byte was malformed. We need to handle
                        // the second one, which isn't in `src`, later.
                        self.life_cycle = DecoderLifeCycle::ConvertingWithPendingBB;
                    }
                    first_read = 0usize; // Wasn't read from `src`!
                }
                DecoderResult::OutputFull => {
                    panic!("Output buffer must have been too small.");
                }
            }
            return (first_result, first_read, first_written);
        }
        if offset == 1usize {
            // The first byte isn't in the current buffer but the second one
            // is.
            return self.$decode_to_utf_after_one_potential_bom_byte(src,
                                                                    dst,
                                                                    last,
                                                                    0usize,
                                                                    0xEFu8);

        }
        debug_assert!(offset == 2usize);
        // The first two bytes are in `src`, so no need to push them separately.
        return self.$decode_to_utf_checking_end(src, dst, last);
    }

    /// Calls `$decode_to_utf_checking_end` with `offset` bytes omitted from
    /// the start of `src` but adjusting the return values to show those bytes
    /// as having been consumed.
    fn $decode_to_utf_checking_end_with_offset(&mut self,
                                               src: &[u8],
                                               dst: &mut [$code_unit],
                                               last: bool,
                                               offset: usize)
                                               -> (DecoderResult, usize, usize) {
        debug_assert!(self.life_cycle == DecoderLifeCycle::Converting);
        let (result, read, written) = self.$decode_to_utf_checking_end(&src[offset..], dst, last);
        return (result, read + offset, written);
    }

    /// Calls through to the delegate and adjusts life cycle iff `last` is
    /// `true` and result is `DecoderResult::InputEmpty`.
    fn $decode_to_utf_checking_end(&mut self,
                                   src: &[u8],
                                   dst: &mut [$code_unit],
                                   last: bool)
                                   -> (DecoderResult, usize, usize) {
        debug_assert!(self.life_cycle == DecoderLifeCycle::Converting);
        let (result, read, written) = self.variant
                                          .$decode_to_utf(src, dst, last);
        if last {
            match result {
                DecoderResult::InputEmpty => {
                    self.life_cycle = DecoderLifeCycle::Finished;
                }
                _ => {}
            }
        }
        return (result, read, written);
    });
}
