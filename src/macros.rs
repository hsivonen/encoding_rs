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
        loop {
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
