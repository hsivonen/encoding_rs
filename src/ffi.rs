// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::*;

/// Return value for `decode_*` and `encode_*` methods that indicates that
/// the input has been exhausted.
///
/// (This is zero as a micro optimization. U+0000 is never unmappable and
/// malformed sequences always have a positive length.)
pub const INPUT_EMPTY: u32 = 0;

/// Return value for `decode_*` and `encode_*` methods that indicates that
/// the output space has been exhausted.
pub const OUTPUT_FULL: u32 = 0xFFFFFFFF;

pub const ENCODING_NAME_MAX_LENGTH: usize = super::LONGEST_NAME_LENGTH;

impl WithReplacementResult {
    fn as_u32(&self) -> u32 {
        match self {
            &WithReplacementResult::InputEmpty => INPUT_EMPTY,
            &WithReplacementResult::OutputFull => OUTPUT_FULL,
        }
    }
}

impl DecoderResult {
    fn as_u32(&self) -> u32 {
        match self {
            &DecoderResult::InputEmpty => INPUT_EMPTY,
            &DecoderResult::OutputFull => OUTPUT_FULL,
            &DecoderResult::Malformed(num) => num as u32,
        }
    }
}

impl EncoderResult {
    fn as_u32(&self) -> u32 {
        match self {
            &EncoderResult::InputEmpty => INPUT_EMPTY,
            &EncoderResult::OutputFull => OUTPUT_FULL,
            &EncoderResult::Unmappable(c) => c as u32,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn encoding_for_label(label: *const u8,
                                            label_len: usize)
                                            -> Option<&'static Encoding> {
    let label_slice = ::std::slice::from_raw_parts(label, label_len);
    Encoding::for_label(label_slice)
}

#[no_mangle]
pub unsafe extern "C" fn encoding_for_label_no_replacement(label: *const u8,
                                                           label_len: usize)
                                                           -> Option<&'static Encoding> {
    let label_slice = ::std::slice::from_raw_parts(label, label_len);
    Encoding::for_label_no_replacement(label_slice)
}

#[no_mangle]
pub unsafe extern "C" fn encoding_for_name(name: *const u8,
                                           name_len: usize)
                                           -> Option<&'static Encoding> {
    let name_slice = ::std::slice::from_raw_parts(name, name_len);
    Encoding::for_name(name_slice)
}

/// Writes the name of the given `Encoding` to a caller-supplied buffer as ASCII
/// and returns the number of bytes / ASCII characters written.
///
/// The output is not null-terminated.
///
/// The caller _MUST_ ensure that `name_out` points to a buffer whose length
/// is at least `ENCODING_NAME_MAX_LENGTH` bytes.
#[no_mangle]
pub unsafe extern "C" fn encoding_name(encoding: &'static Encoding, name_out: *mut u8) -> usize {
    let bytes = encoding.name().as_bytes();
    ::std::ptr::copy_nonoverlapping(bytes.as_ptr(), name_out, bytes.len());
    bytes.len()
}

#[no_mangle]
pub unsafe extern "C" fn encoding_can_encode_everything(encoding: &'static Encoding) -> bool {
    encoding.can_encode_everything()
}

/// Allocates a new `Decoder` for the given `Encoding` on the heap and returns a
/// pointer to the newly-allocated `Decoder`.
///
/// Once the allocated `Decoder` is no longer needed, the caller _MUST_
/// deallocate it by passing the pointer returned by this function to
/// `decoder_free()`.
#[no_mangle]
pub unsafe extern "C" fn encoding_new_decoder(encoding: &'static Encoding) -> *mut Decoder {
    Box::into_raw(Box::new(encoding.new_decoder()))
}

/// Allocates a new `Encoder` for the given `Encoding` on the heap and returns a
/// pointer to the newly-allocated `Encoder`. (Exception, if the `Encoding` is
/// `replacement`, a new `Decoder` for UTF-8 is instantiated (and that
/// `Decoder` reports `UTF_8` as its `Encoding`).
///
/// Once the allocated `Encoder` is no longer needed, the caller _MUST_
/// deallocate it by passing the pointer returned by this function to
/// `encoder_free()`.
#[no_mangle]
pub unsafe extern "C" fn encoding_new_encoder(encoding: &'static Encoding) -> *mut Encoder {
    Box::into_raw(Box::new(encoding.new_encoder()))
}

/// Deallocates a `Decoder` previously allocated by `encoding_new_decoder()`.
#[no_mangle]
pub unsafe extern "C" fn decoder_free(decoder: *mut Decoder) {
    let _ = Box::from_raw(decoder);
}

#[no_mangle]
pub unsafe extern "C" fn decoder_encoding(decoder: &Decoder) -> &'static Encoding {
    decoder.encoding()
}

#[no_mangle]
pub unsafe extern "C" fn decoder_reset(decoder: &mut Decoder) {
    decoder.reset();
}

#[no_mangle]
pub extern "C" fn decoder_max_utf16_length(decoder: &Decoder, u16_length: usize) -> usize {
    decoder.max_utf16_buffer_length(u16_length)
}

#[no_mangle]
pub extern "C" fn decoder_max_utf8_length(decoder: &Decoder, byte_length: usize) -> usize {
    decoder.max_utf8_buffer_length(byte_length)
}

#[no_mangle]
pub extern "C" fn decoder_max_utf8_length_with_replacement(decoder: &Decoder,
                                                           byte_length: usize)
                                                           -> usize {
    decoder.max_utf8_buffer_length_with_replacement(byte_length)
}

#[no_mangle]
pub unsafe extern "C" fn decoder_decode_to_utf16(decoder: &mut Decoder,
                                                 src: *const u8,
                                                 src_len: *mut usize,
                                                 dst: *mut u16,
                                                 dst_len: *mut usize,
                                                 last: bool)
                                                 -> u32 {
    let src_slice = ::std::slice::from_raw_parts(src, *src_len);
    let dst_slice = ::std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written) = decoder.decode_to_utf16(src_slice, dst_slice, last);
    *src_len = read;
    *dst_len = written;
    result.as_u32()
}

#[no_mangle]
pub unsafe extern "C" fn decoder_decode_to_utf8(decoder: &mut Decoder,
                                                src: *const u8,
                                                src_len: *mut usize,
                                                dst: *mut u8,
                                                dst_len: *mut usize,
                                                last: bool)
                                                -> u32 {
    let src_slice = ::std::slice::from_raw_parts(src, *src_len);
    let dst_slice = ::std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written) = decoder.decode_to_utf8(src_slice, dst_slice, last);
    *src_len = read;
    *dst_len = written;
    result.as_u32()
}

#[no_mangle]
pub unsafe extern "C" fn decoder_decode_to_utf16_with_replacement(decoder: &mut Decoder,
                                                                  src: *const u8,
                                                                  src_len: *mut usize,
                                                                  dst: *mut u16,
                                                                  dst_len: *mut usize,
                                                                  last: bool,
                                                                  had_replacements: *mut bool)
                                                                  -> u32 {
    let src_slice = ::std::slice::from_raw_parts(src, *src_len);
    let dst_slice = ::std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written, replaced) = decoder.decode_to_utf16_with_replacement(src_slice,
                                                                                     dst_slice,
                                                                                     last);
    *src_len = read;
    *dst_len = written;
    *had_replacements = replaced;
    result.as_u32()
}

#[no_mangle]
pub unsafe extern "C" fn decoder_decode_to_utf8_with_replacement(decoder: &mut Decoder,
                                                                 src: *const u8,
                                                                 src_len: *mut usize,
                                                                 dst: *mut u8,
                                                                 dst_len: *mut usize,
                                                                 last: bool,
                                                                 had_replacements: *mut bool)
                                                                 -> u32 {
    let src_slice = ::std::slice::from_raw_parts(src, *src_len);
    let dst_slice = ::std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written, replaced) = decoder.decode_to_utf8_with_replacement(src_slice,
                                                                                    dst_slice,
                                                                                    last);
    *src_len = read;
    *dst_len = written;
    *had_replacements = replaced;
    result.as_u32()
}

/// Deallocates an `Encoder` previously allocated by `encoding_new_encoder()`.
#[no_mangle]
pub unsafe extern "C" fn encoder_free(encoder: *mut Encoder) {
    let _ = Box::from_raw(encoder);
}

#[no_mangle]
pub unsafe extern "C" fn encoder_encoding(encoder: &Encoder) -> &'static Encoding {
    encoder.encoding()
}

#[no_mangle]
pub unsafe extern "C" fn encoder_reset(encoder: &mut Encoder) {
    encoder.reset();
}

#[no_mangle]
pub extern "C" fn encoder_max_buffer_length_from_utf16(encoder: &Encoder,
                                                       u16_length: usize)
                                                       -> usize {
    encoder.max_buffer_length_from_utf16(u16_length)
}

#[no_mangle]
pub extern "C" fn encoder_max_buffer_length_from_utf8(encoder: &Encoder,
                                                      byte_length: usize)
                                                      -> usize {
    encoder.max_buffer_length_from_utf8(byte_length)
}


#[no_mangle]
pub extern "C" fn encoder_max_buffer_length_from_utf16_with_replacement_if_no_unmappables
    (encoder: &Encoder,
     u16_length: usize)
     -> usize {
    encoder.max_buffer_length_from_utf16_with_replacement_if_no_unmappables(u16_length)
}

#[no_mangle]
pub extern "C" fn encoder_max_buffer_length_from_utf8_with_replacement_if_no_unmappables
    (encoder: &Encoder,
     byte_length: usize)
     -> usize {
    encoder.max_buffer_length_from_utf8_with_replacement_if_no_unmappables(byte_length)
}

#[no_mangle]
pub unsafe extern "C" fn encoder_encode_from_utf16(encoder: &mut Encoder,
                                                   src: *const u16,
                                                   src_len: *mut usize,
                                                   dst: *mut u8,
                                                   dst_len: *mut usize,
                                                   last: bool)
                                                   -> u32 {
    let src_slice = ::std::slice::from_raw_parts(src, *src_len);
    let dst_slice = ::std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written) = encoder.encode_from_utf16(src_slice, dst_slice, last);
    *src_len = read;
    *dst_len = written;
    result.as_u32()
}

#[no_mangle]
pub unsafe extern "C" fn encoder_encode_from_utf8(encoder: &mut Encoder,
                                                  src: *const u8,
                                                  src_len: *mut usize,
                                                  dst: *mut u8,
                                                  dst_len: *mut usize,
                                                  last: bool)
                                                  -> u32 {
    let src_slice = ::std::slice::from_raw_parts(src, *src_len);
    let string = ::std::str::from_utf8_unchecked(src_slice);
    let dst_slice = ::std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written) = encoder.encode_from_utf8(string, dst_slice, last);
    *src_len = read;
    *dst_len = written;
    result.as_u32()
}

#[no_mangle]
pub unsafe extern "C" fn encoder_encode_from_utf16_with_replacement(encoder: &mut Encoder,
                                                                    src: *const u16,
                                                                    src_len: *mut usize,
                                                                    dst: *mut u8,
                                                                    dst_len: *mut usize,
                                                                    last: bool,
                                                                    had_replacements: *mut bool)
                                                                    -> u32 {
    let src_slice = ::std::slice::from_raw_parts(src, *src_len);
    let dst_slice = ::std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written, replaced) = encoder.encode_from_utf16_with_replacement(src_slice,
                                                                                       dst_slice,
                                                                                       last);
    *src_len = read;
    *dst_len = written;
    *had_replacements = replaced;
    result.as_u32()
}

#[no_mangle]
pub unsafe extern "C" fn encoder_encode_from_utf8_with_replacement(encoder: &mut Encoder,
                                                                   src: *const u8,
                                                                   src_len: *mut usize,
                                                                   dst: *mut u8,
                                                                   dst_len: *mut usize,
                                                                   last: bool,
                                                                   had_replacements: *mut bool)
                                                                   -> u32 {
    let src_slice = ::std::slice::from_raw_parts(src, *src_len);
    let string = ::std::str::from_utf8_unchecked(src_slice);
    let dst_slice = ::std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written, replaced) = encoder.encode_from_utf8_with_replacement(string,
                                                                                      dst_slice,
                                                                                      last);
    *src_len = read;
    *dst_len = written;
    *had_replacements = replaced;
    result.as_u32()
}
