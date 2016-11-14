// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
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
                      decode_to_utf8_raw,
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
                      decode_to_utf16_raw,
                      u16,
                      Utf16Destination);
    );
}

macro_rules! ascii_compatible_two_byte_decoder_function {
    ($lead:block,
     $trail:block,
     $slf:ident,
     $non_ascii:ident,
     $byte:ident,
     $lead_minus_offset:ident,
     $unread_handle_trail:ident,
     $source:ident,
     $handle:ident,
     $copy_ascii:ident,
     $destination_check:ident,
     $name:ident,
     $code_unit:ty,
     $dest_struct:ident,
     $ascii_punctuation:expr) => (
    pub fn $name(&mut $slf,
                 src: &[u8],
                 dst: &mut [$code_unit],
                 last: bool)
                 -> (DecoderResult, usize, usize) {
        let mut $source = ByteSource::new(src);
        let mut dest_prolog = $dest_struct::new(dst);
        let dest = if $slf.lead != 0 {
            let $lead_minus_offset = $slf.lead;
// Since we don't have `goto` we could use to jump into the trail
// handling part of the main loop, we need to repeat trail handling
// here.
            match $source.check_available() {
                Space::Full(src_consumed_prolog) => {
                    if last {
                        return (DecoderResult::Malformed(1, 0),
                                src_consumed_prolog,
                                dest_prolog.written());
                    }
                    return (DecoderResult::InputEmpty, src_consumed_prolog, dest_prolog.written());
                }
                Space::Available(source_handle_prolog) => {
                    match dest_prolog.$destination_check() {
                        Space::Full(dst_written_prolog) => {
                            return (DecoderResult::OutputFull,
                                    source_handle_prolog.consumed(),
                                    dst_written_prolog);
                        }
                        Space::Available($handle) => {
                            let ($byte, $unread_handle_trail) = source_handle_prolog.read();
// Start non-boilerplate
                            $trail
// End non-boilerplate
                        }
                    }
                }
            }
        } else {
            &mut dest_prolog
        };
        'outermost: loop {
            match dest.$copy_ascii(&mut $source) {
                CopyAsciiResult::Stop(ret) => return ret,
                CopyAsciiResult::GoOn((mut $non_ascii, mut $handle)) => {
                    'middle: loop {
                        let dest_again = {
                            let $lead_minus_offset = {
// Start non-boilerplate
                                $lead
// End non-boilerplate
                            };
                            match $source.check_available() {
                                Space::Full(src_consumed_trail) => {
                                    if last {
                                        return (DecoderResult::Malformed(1, 0),
                                                src_consumed_trail,
                                                $handle.written());
                                    }
                                    $slf.lead = $lead_minus_offset;
                                    return (DecoderResult::InputEmpty,
                                            src_consumed_trail,
                                            $handle.written());
                                }
                                Space::Available(source_handle_trail) => {
                                    let ($byte, $unread_handle_trail) = source_handle_trail.read();
// Start non-boilerplate
                                    $trail
// End non-boilerplate
                                }
                            }
                        };
                        match $source.check_available() {
                            Space::Full(src_consumed) => {
                                return (DecoderResult::InputEmpty,
                                        src_consumed,
                                        dest_again.written());
                            }
                            Space::Available(source_handle) => {
                                match dest_again.$destination_check() {
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
                                                $non_ascii = b;
                                                $handle = destination_handle;
                                                continue 'middle;
                                            }
// Testing on Haswell says that we should write the
// byte unconditionally instead of trying to unread it
// to make it part of the next SIMD stride.
                                            let dest_again_again =
                                                destination_handle.write_ascii(b);
                                            if $ascii_punctuation && b < 60 {
// We've got punctuation
                                                match source_again.check_available() {
                                                    Space::Full(src_consumed_again) => {
                                                        return (DecoderResult::InputEmpty,
                                                                src_consumed_again,
                                                                dest_again_again.written());
                                                    }
                                                    Space::Available(source_handle_again) => {
                                                        match dest_again_again.$destination_check() {
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
    });
}

macro_rules! ascii_compatible_two_byte_decoder_functions {
    ($lead:block,
     $trail:block,
     $slf:ident,
     $non_ascii:ident,
     $byte:ident,
     $lead_minus_offset:ident,
     $unread_handle_trail:ident,
     $source:ident,
     $handle:ident,
     $copy_ascii:ident,
     $destination_check:ident,
     $ascii_punctuation:expr) => (
         ascii_compatible_two_byte_decoder_function!($lead,
                                                      $trail,
                                                      $slf,
                                                      $non_ascii,
                                                      $byte,
                                                      $lead_minus_offset,
                                                      $unread_handle_trail,
                                                      $source,
                                                      $handle,
                                                      $copy_ascii,
                                                      $destination_check,
                                                      decode_to_utf8_raw,
                                                      u8,
                                                      Utf8Destination,
                                                      $ascii_punctuation);
         ascii_compatible_two_byte_decoder_function!($lead,
                                                      $trail,
                                                      $slf,
                                                      $non_ascii,
                                                      $byte,
                                                      $lead_minus_offset,
                                                      $unread_handle_trail,
                                                      $source,
                                                      $handle,
                                                      $copy_ascii,
                                                      $destination_check,
                                                      decode_to_utf16_raw,
                                                      u16,
                                                      Utf16Destination,
                                                      $ascii_punctuation);
    );
}

macro_rules! encoder_function {
    ($eof:block,
     $body:block,
     $slf:ident,
     $src_consumed:ident,
     $source:ident,
     $dest:ident,
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
        let mut $dest = ByteDestination::new(dst);
        loop {
            match $source.check_available() {
                Space::Full($src_consumed) => {
                    if last {
                        // Start non-boilerplate
                        $eof
                        // End non-boilerplate
                    }
                    return (EncoderResult::InputEmpty, $src_consumed, $dest.written());
                }
                Space::Available(source_handle) => {
                    match $dest.$destination_check() {
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
     $dest:ident,
     $c:ident,
     $destination_handle:ident,
     $unread_handle:ident,
     $destination_check:ident) => (
    encoder_function!($eof,
                      $body,
                      $slf,
                      $src_consumed,
                      $source,
                      $dest,
                      $c,
                      $destination_handle,
                      $unread_handle,
                      $destination_check,
                      encode_from_utf8_raw,
                      str,
                      Utf8Source);
    encoder_function!($eof,
                      $body,
                      $slf,
                      $src_consumed,
                      $source,
                      $dest,
                      $c,
                      $destination_handle,
                      $unread_handle,
                      $destination_check,
                      encode_from_utf16_raw,
                      [u16],
                      Utf16Source);
    );
}

macro_rules! ascii_compatible_encoder_function {
    ($bmp_body:block,
     $astral_body:block,
     $bmp:ident,
     $astral:ident,
     $slf:ident,
     $source:ident,
     $handle:ident,
     $copy_ascii:ident,
     $destination_check:ident,
     $name:ident,
     $input:ty,
     $source_struct:ident,
     $ascii_punctuation:expr) => (
    pub fn $name(&mut $slf,
                 src: &$input,
                 dst: &mut [u8],
                 _last: bool)
                 -> (EncoderResult, usize, usize) {
        let mut $source = $source_struct::new(src);
        let mut dest = ByteDestination::new(dst);
        'outermost: loop {
            match $source.$copy_ascii(&mut dest) {
                CopyAsciiResult::Stop(ret) => return ret,
                CopyAsciiResult::GoOn((mut non_ascii, mut $handle)) => {
                    'middle: loop {
                        let dest_again = match non_ascii {
                            NonAscii::BmpExclAscii($bmp) => {
// Start non-boilerplate
                                $bmp_body
// End non-boilerplate
                            }
                            NonAscii::Astral($astral) => {
// Start non-boilerplate
                                $astral_body
// End non-boilerplate
                            }
                        };
                        match $source.check_available() {
                            Space::Full(src_consumed) => {
                                return (EncoderResult::InputEmpty,
                                        src_consumed,
                                        dest_again.written());
                            }
                            Space::Available(source_handle) => {
                                match dest_again.$destination_check() {
                                    Space::Full(dst_written) => {
                                        return (EncoderResult::OutputFull,
                                                source_handle.consumed(),
                                                dst_written);
                                    }
                                    Space::Available(mut destination_handle) => {
                                        let (mut c, unread_handle) = source_handle.read_enum();
                                        let source_again = unread_handle.decommit();
                                        'innermost: loop {
                                            let ascii = match c {
                                                Unicode::NonAscii(non_ascii_again) => {
                                                    non_ascii = non_ascii_again;
                                                    $handle = destination_handle;
                                                    continue 'middle;
                                                }
                                                Unicode::Ascii(a) => a,
                                            };
// Testing on Haswell says that we should write the
// byte unconditionally instead of trying to unread it
// to make it part of the next SIMD stride.
                                            let dest_again_again =
                                                destination_handle.write_one(ascii);
                                            if $ascii_punctuation && ascii < 60 {
// We've got punctuation
                                                match source_again.check_available() {
                                                    Space::Full(src_consumed_again) => {
                                                        return (EncoderResult::InputEmpty,
                                                                src_consumed_again,
                                                                dest_again_again.written());
                                                    }
                                                    Space::Available(source_handle_again) => {
                                                        match dest_again_again.$destination_check() {
                                                            Space::Full(dst_written_again) => {
                                                                return (EncoderResult::OutputFull,
                                                                        source_handle_again.consumed(),
                                                                        dst_written_again);
                                                            }
                                                            Space::Available(destination_handle_again) => {
                                                                {
                                                                    let (c_again, _unread_handle_again) =
                                                                        source_handle_again.read_enum();
                                                                    c = c_again;
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
    });
}

macro_rules! ascii_compatible_encoder_functions {
    ($bmp_body:block,
     $astral_body:block,
     $bmp:ident,
     $astral:ident,
     $slf:ident,
     $source:ident,
     $handle:ident,
     $copy_ascii:ident,
     $destination_check:ident,
     $ascii_punctuation:expr) => (
    ascii_compatible_encoder_function!($bmp_body,
                                       $astral_body,
                                       $bmp,
                                       $astral,
                                       $slf,
                                       $source,
                                       $handle,
                                       $copy_ascii,
                                       $destination_check,
                                       encode_from_utf8_raw,
                                       str,
                                       Utf8Source,
                                       $ascii_punctuation);
    ascii_compatible_encoder_function!($bmp_body,
                                       $astral_body,
                                       $bmp,
                                       $astral,
                                       $slf,
                                       $source,
                                       $handle,
                                       $copy_ascii,
                                       $destination_check,
                                       encode_from_utf16_raw,
                                       [u16],
                                       Utf16Source,
                                       $ascii_punctuation);
     );
}

macro_rules! ascii_compatible_bmp_encoder_function {
    ($bmp_body:block,
     $bmp:ident,
     $slf:ident,
     $source:ident,
     $handle:ident,
     $copy_ascii:ident,
     $destination_check:ident,
     $name:ident,
     $input:ty,
     $source_struct:ident,
     $ascii_punctuation:expr) => (
    ascii_compatible_encoder_function!($bmp_body,
                                       {
                                           return (EncoderResult::Unmappable(astral),
                                                   $source.consumed(),
                                                   $handle.written());
                                       },
                                       $bmp,
                                       astral,
                                       $slf,
                                       $source,
                                       $handle,
                                       $copy_ascii,
                                       $destination_check,
                                       $name,
                                       $input,
                                       $source_struct,
                                       $ascii_punctuation);
     );
}

macro_rules! ascii_compatible_bmp_encoder_functions {
    ($bmp_body:block,
     $bmp:ident,
     $slf:ident,
     $source:ident,
     $handle:ident,
     $copy_ascii:ident,
     $destination_check:ident,
     $ascii_punctuation:expr) => (
    ascii_compatible_encoder_functions!($bmp_body,
                                        {
                                            return (EncoderResult::Unmappable(astral),
                                                    $source.consumed(),
                                                    $handle.written());
                                        },
                                        $bmp,
                                        astral,
                                        $slf,
                                        $source,
                                        $handle,
                                        $copy_ascii,
                                        $destination_check,
                                        $ascii_punctuation);
     );
}

macro_rules! public_decode_function{
    ($(#[$meta:meta])*,
     $decode_to_utf:ident,
     $decode_to_utf_raw:ident,
     $decode_to_utf_checking_end:ident,
     $decode_to_utf_after_one_potential_bom_byte:ident,
     $decode_to_utf_after_two_potential_bom_bytes:ident,
     $decode_to_utf_checking_end_with_offset:ident,
     $code_unit:ty) => (
    $(#[$meta])*
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
                DecoderLifeCycle::AtUtf8Start => {
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
                        _ => {
                            self.life_cycle = DecoderLifeCycle::Converting;
                            continue;
                        }
                    }
                }
                DecoderLifeCycle::AtUtf16BeStart => {
                    debug_assert!(offset == 0usize);
                    if src.is_empty() {
                        return (DecoderResult::InputEmpty, 0, 0);
                    }
                    match src[0] {
                        0xFEu8 => {
                            self.life_cycle = DecoderLifeCycle::SeenUtf16BeFirst;
                            offset += 1;
                            continue;
                        }
                        _ => {
                            self.life_cycle = DecoderLifeCycle::Converting;
                            continue;
                        }
                    }
                }
                DecoderLifeCycle::AtUtf16LeStart => {
                    debug_assert!(offset == 0usize);
                    if src.is_empty() {
                        return (DecoderResult::InputEmpty, 0, 0);
                    }
                    match src[0] {
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
            let mut out_read = 0usize;
            let (mut first_result, _, mut first_written) =
                self.variant
                    .$decode_to_utf_raw(&first[..], dst, last);
            match first_result {
                DecoderResult::InputEmpty => {
                    let (result, read, written) =
                        self.$decode_to_utf_checking_end(src, &mut dst[first_written..], last);
                    first_result = result;
                    out_read = read; // Overwrite, don't add!
                    first_written += written;
                }
                DecoderResult::Malformed(_, _) => {
                    // Wasn't read from `src`!, leave out_read to 0
                }
                DecoderResult::OutputFull => {
                    panic!("Output buffer must have been too small.");
                }
            }
            return (first_result, out_read, first_written);
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
                    .$decode_to_utf_raw(&ef_bb[..], dst, last);
            match first_result {
                DecoderResult::InputEmpty => {
                    let (result, read, written) =
                        self.$decode_to_utf_checking_end(src, &mut dst[first_written..], last);
                    first_result = result;
                    first_read = read; // Overwrite, don't add!
                    first_written += written;
                }
                DecoderResult::Malformed(_, _) => {
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
                                          .$decode_to_utf_raw(src, dst, last);
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
