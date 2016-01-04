// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

static TEST: Encoding = Encoding {name: "foo", dom_name: "Foo"};

pub struct Encoding {
    name: &'static str,
    dom_name: &'static str,
}

impl Encoding {
    fn for_label(label: &[u8]) -> Option<&'static Encoding> {
        Some(&TEST)
    }
    fn for_label_no_replacement(label: &[u8]) -> Option<&'static Encoding> {
        Some(&TEST)
    }
    fn for_dom_name(dom_name: &[u8]) -> Option<&'static Encoding> {
        Some(&TEST)
    }
// new_decoder
// new_encoder
// encode
// decode
// encode_with_replacement
// decode_with_replacement
}

/// Result of a (potentially partial) decode operation.
pub enum DecoderResult {
   /// The input was exhausted.
   ///
   /// If this result was returned from a call where `last` was `true`, the
   /// decoding process has completed. Otherwise, the caller should call a
   /// decode method again with more input.
   Underflow,

   /// The decoder cannot produce another unit of output, because the output
   /// buffer does not have enough space left.
   ///
   /// The caller must provide more output space upon the next call and re-push
   /// the remaining input to the decoder.
   Overflow,

   /// The decoder encountered a malformed byte sequence.
   ///
   /// The caller must either treat this as a fatal error or must append one
   /// REPLACEMENT CHARACTER (U+FFFD) to the output and then re-push the
   /// the remaining input to the decoder.
   ///
   /// The wrapped integer indicates the length of the malformed byte sequence.
   /// The last byte that was consumed is the last byte of the malformed
   /// sequence. Note that the earlier bytes may have been part of an earlier
   /// input buffer.
   Malformed(u8), // u8 instead of usize to avoid uselessly bloating the enum
}

/// A converter that decodes a byte stream into Unicode according to a
/// character encoding.
///
/// The various `decode_*` methods take an input buffer (`src`) and an output
/// buffer `dst` both of which are caller-allocated. There are variants for 
/// both UTF-8 and UTF-16 output buffers.
///
/// A `decode_*` methods decode bytes from `src` into Unicode characters stored
/// into `dst` until one of the following three things happens:
///
/// 1. A malformed byte sequence is encountered.
///
/// 2. The output buffer has been filled so near capacity that the decoder
///    cannot be sure that processing an additional byte of input wouldn't
///    cause so much output that the output buffer would overflow.
///
/// 3. All the input bytes have been processed.
///
/// The `decode_*` method then returns tuple of a status indicating which one
/// of the three reasons to return happened, how many input bytes were read,
/// how many output code units (`u8` when decoding into UTF-8 and `u16`
/// when decoding to UTF-16) were written (except when decoding into `String`,
/// whose length change indicates this), and in the case of the
/// `*_with_replacement` variants, a boolean indicating whether an error was
/// replaced with the REPLACEMENT CHARACTER during the call.
///
/// In the case of the methods whose name does not end with
/// `*_with_replacement`, the status is a `DecoderResult` enumeration
/// (possibilities `Malformed`, `Overflow` and `Underflow` corresponding to the
/// three cases listed above).
///
/// In the case of methods whose name ends with `*_with_replacement`, malformed
/// sequences are automatically replaced with the REPLACEMENT CHARACTER and
/// errors do not cause the methods to return early. The return value `true`
/// signals that all input has been processed and `false` signals that more
/// output space is needed.
///
/// When decodering to UTF-8, the output buffer must have at least 5 bytes of
/// space. (Yes, 5, not 4, because Big5 is special.) When decoding to UTF-16,
/// the output buffer must have at least two UTF-16 code units (`u16`) of space.
///
/// When decoding to UTF-8 without replacement, the methods are guaranteed
/// not to return indicating that more output space is needed if the length
/// of the ouput buffer is at least the length returned by
/// `max_utf8_buffer_length()`. When decoding to UTF-8 with replacement, the
/// the length of the output buffer that guarantees the methods not to return
/// indicating that more output space is needed is given by
/// `max_utf8_buffer_length_with_replacement()`. When decoding to UTF-16 with
/// or without replacement, the length of the output buffer that guarantees
/// the methods not to return indicating that more output space is needed is
/// given by `max_utf16_buffer_length()`.
///
/// The output written into `dst` is guaranteed to be valid UTF-8 or UTF-16,
/// and the output after each `decode_*` call is guaranteed to consist of
/// complete characters. (I.e. the code unit sequence for the last character is
/// guaranteed not to be split across output buffers.)
///
/// The boolean argument `last` indicates that the end of the stream is reached
/// when all the bytes in `src` have been consumed.
///
/// A `Decoder` object can be used to incrementally decode a byte stream. The
/// decoder cannot be used for multiple streams concurrently but can be used
/// for multiple streams sequentially.
///
/// During the processing of a single stream, the caller must call `decode_*`
/// zero or more times with `last` set to `false` and then call `decode_*` at
/// least once with `last` set to `true`. If `decode_*` returns `Underflow`
/// (`true` in the `*_with_replacement` case), the processing of the stream
/// has ended. Otherwise, the caller must call `decode_*` again with `last`
/// set to `true` (or treat a `Malformed` result as a fatal error).
///
/// The decoder is ready to start processing a new stream when it has
/// returned `Underflow` (`true` in the `*_with_replacement` case) from a call
/// where `last` was set to `true`. In other cases, if the caller wishes to
/// stop processing the current stream and to start processing a new stream,
/// the caller must call `reset()` before starting processing the new stream.
///
/// When the decoder returns `Overflow` (`false` in the `*_with_replacement`
/// case) or the decoder returns `Malformed` and the caller does not wish to
/// treat it as a fatal error, the input buffer `src` may not have been
/// completely consumed. In that case, the caller must pass the unconsumed
/// contents of `src` to `decode_*` again upon the next call.
pub trait Decoder {
    /// Make the decoder ready to process a new stream.
    fn reset(&mut self);

    /// Query the worst-case UTF-16 output size (with or without replacement).
    ///
    /// Returns the size of the output buffer in UTF-16 code units (`u16`)
    /// that will not overflow given the current state of the decoder and
    /// `byte_length` number of additional input bytes.
    ///
    /// Since the REPLACEMENT CHARACTER fits into one UTF-16 code unit, the
    /// return value of this method applies also in the
    /// `_with_replacement` case.
    ///
    /// Available via the C wrapper.
    fn max_utf16_buffer_length(&self, byte_length: usize) -> usize;

    /// Query the worst-case UTF-8 output size _without replacement_.
    ///
    /// Returns the size of the output buffer in UTF-8 code units (`u8`)
    /// that will not overflow given the current state of the decoder and
    /// `byte_length` number of additional input bytes when decoding without
    /// replacement error handling.
    ///
    /// Note that this value may be too small for the `_with_replacement` case.
    /// Use `max_utf8_buffer_length_with_replacement` for that case.
    ///
    /// Available via the C wrapper.
    fn max_utf8_buffer_length(&self, byte_length: usize) -> usize;

    /// Query the worst-case UTF-8 output size _with replacement_.
    ///
    /// Returns the size of the output buffer in UTF-8 code units (`u8`)
    /// that will not overflow given the current state of the decoder and
    /// `byte_length` number of additional input bytes when decoding with
    /// errors handled by outputting a REPLACEMENT CHARACTER for each malformed
    /// sequence.
    ///
    /// Available via the C wrapper.
    fn max_utf8_buffer_length_with_replacement(&self, byte_length: usize) -> usize;

    /// Incrementally decode a byte stream into UTF-16.
    ///
    /// See the documentation of the trait for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    fn decode_to_utf16(&mut self, src: &[u8], dst: &mut [u16], last: bool) -> (DecoderResult, usize, usize);

    /// Incrementally decode a byte stream into UTF-8.
    ///
    /// See the documentation of the trait for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    fn decode_to_utf8(&mut self, src: &[u8], dst: &mut [u8], last: bool) -> (DecoderResult, usize, usize);

    /// Incrementally decode a byte stream into UTF-8 with type system signaling
    /// of UTF-8 validity.
    ///
    /// This methods calls `decode_to_utf8` and then zeroes out up to three
    /// bytes that aren't logically part of the write in order to retain the
    /// UTF-8 validity even for the unwritten part of the buffer.
    ///
    /// See the documentation of the trait for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    fn decode_to_str(&mut self, src: &[u8], dst: &mut str, last: bool) -> (DecoderResult, usize, usize) {
        let bytes: &mut [u8] = unsafe { std::mem::transmute(dst) };
        let (result, read, written) = self.decode_to_utf8(src, bytes, last);
        let len = bytes.len();
        let mut trail = written;
        while trail < len && ((bytes[trail] & 0xC0) == 0x80) {
            bytes[trail] = 0;
            trail += 1;
        }
        (result, read, written)
    }

    /// Incrementally decode a byte stream into UTF-8 using a `String` receiver.
    ///
    /// Like the others, this method follows the logic that the output buffer is
    /// caller-allocated. This method treats the capacity of the `String` as
    /// the output limit. That is, this method guarantees not to cause a
    /// reallocation of the backing buffer of `String`.
    ///
    /// The return value is a pair that contains the `DecoderResult` and the
    /// number of bytes read. The number of bytes written is signaled via
    /// the length of the `String` changing.
    ///
    /// See the documentation of the trait for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    fn decode_to_string(&mut self, src: &[u8], dst: &mut String, last: bool) -> (DecoderResult, usize) {
        unsafe {
            let vec = dst.as_mut_vec();
            let old_len = vec.len();
            let capacity = vec.capacity();
            vec.set_len(capacity);
            let (result, read, written) = self.decode_to_utf8(src, &mut vec[old_len..], last);
            vec.set_len(old_len + written);
            (result, read)
        }
    }

    /// Incrementally decode a byte stream into UTF-16 with malformed sequences
    /// replaced with the REPLACEMENT CHARACTER.
    ///
    /// See the documentation of the trait for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    fn decode_to_utf16_with_replacement(&mut self, src: &[u8], dst: &mut [u16], last: bool) -> (bool, usize, usize, bool) {
        // XXX
        (true, 0, 0, false)
    }

    /// Incrementally decode a byte stream into UTF-8 with malformed sequences
    /// replaced with the REPLACEMENT CHARACTER.
    ///
    /// See the documentation of the trait for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    fn decode_to_utf8_with_replacement(&mut self, src: &[u8], dst: &mut [u8], last: bool) -> (bool, usize, usize, bool) {
        // XXX
        (true, 0, 0, false)
    }

    /// Incrementally decode a byte stream into UTF-8 with malformed sequences
    /// replaced with the REPLACEMENT CHARACTER with type system signaling
    /// of UTF-8 validity.
    ///
    /// This methods calls `decode_to_utf8_with_replacement` and then zeroes
    /// out up to three bytes that aren't logically part of the write in order
    /// to retain the UTF-8 validity even for the unwritten part of the buffer.
    ///
    /// See the documentation of the trait for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    fn decode_to_str_with_replacement(&mut self, src: &[u8], dst: &mut str, last: bool) -> (bool, usize, usize, bool) {
        let bytes: &mut [u8] = unsafe { std::mem::transmute(dst) };
        let (result, read, written, replaced) = self.decode_to_utf8_with_replacement(src, bytes, last);
        let len = bytes.len();
        let mut trail = written;
        while trail < len && ((bytes[trail] & 0xC0) == 0x80) {
            bytes[trail] = 0;
            trail += 1;
        }
        (result, read, written, replaced)
    }

    /// Incrementally decode a byte stream into UTF-8 with malformed sequences
    /// replaced with the REPLACEMENT CHARACTER using a `String` receiver.
    ///
    /// Like the others, this method follows the logic that the output buffer is
    /// caller-allocated. This method treats the capacity of the `String` as
    /// the output limit. That is, this method guarantees not to cause a
    /// reallocation of the backing buffer of `String`.
    ///
    /// The return value is a tuple that contains the `DecoderResult`, the
    /// number of bytes read and a boolean indicating whether replacements
    /// were done. The number of bytes written is signaled via the length of
    /// the `String` changing.
    ///
    /// See the documentation of the trait for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    fn decode_to_string_with_replacement(&mut self, src: &[u8], dst: &mut String, last: bool) -> (bool, usize, bool) {
        unsafe {
            let vec = dst.as_mut_vec();
            let old_len = vec.len();
            let capacity = vec.capacity();
            vec.set_len(capacity);
            let (result, read, written, replaced) = self.decode_to_utf8_with_replacement(src, &mut vec[old_len..], last);
            vec.set_len(old_len + written);
            (result, read, replaced)
        }
    }

}

/// Result of a (potentially partial) encode operation.
pub enum EncoderResult {
   /// The input was exhausted.
   ///
   /// If this result was returned from a call where `last` was `true`, the
   /// decoding process has completed. Otherwise, the caller should call a
   /// decode method again with more input.
   Underflow,

   /// The encoder cannot produce another unit of output, because the output
   /// buffer does not have enough space left.
   ///
   /// The caller must provide more output space upon the next call and re-push
   /// the remaining input to the decoder.
   Overflow,

   /// The encoder encountered an unmappable character.
   ///
   /// The caller must either treat this as a fatal error or must append
   /// a placeholder to the output and then re-push the the remaining input to
   /// the encoder.
   Unmappable(char),
}

/// A converter that encodes a Unicode stream into bytes according to a
/// character encoding.
///
/// The various `encode_*` methods take an input buffer (`src`) and an output
/// buffer `dst` both of which are caller-allocated. There are variants for 
/// both UTF-8 and UTF-16 input buffers.
///
/// A `encode_*` methods encode characters from `src` into bytes characters
/// stored into `dst` until one of the following three things happens:
///
/// 1. An unmappable character is encountered.
///
/// 2. The output buffer has been filled so near capacity that the decoder
///    cannot be sure that processing an additional character of input wouldn't
///    cause so much output that the output buffer would overflow.
///
/// 3. All the input characters have been processed.
///
/// The `encode_*` method then returns tuple of a status indicating which one
/// of the three reasons to return happened, how many input code units (`u8`
/// when encoding from UTF-8 and `u16` when encoding from UTF-16) were read,
/// how many output bytes were written (except when encoding into `Vec<u8>`,
/// whose length change indicates this), and in the case of the
/// `*_with_replacement` variants, a boolean indicating whether an unmappable
/// character was replaced with a numeric character reference during the call.
///
/// In the case of the methods whose name does not end with
/// `*_with_replacement`, the status is an `EncoderResult` enumeration
/// (possibilities `Unmappable`, `Overflow` and `Underflow` corresponding to the
/// three cases listed above).
///
/// In the case of methods whose name ends with `*_with_replacement`, unmappable
/// characters are automatically replaced with the corresponding numeric
/// character references and unmappable characters do not cause the methods to
/// return early. The return value `true` signals that all input has been
/// processed and `false` signals that more output space is needed.
///
/// XXX: When decoding to UTF-8 without replacement, the methods are guaranteed
/// not to return indicating that more output space is needed if the length
/// of the ouput buffer is at least the length returned by
/// `max_utf8_buffer_length()`. When decoding to UTF-8 with replacement, the
/// the length of the output buffer that guarantees the methods not to return
/// indicating that more output space is needed is given by
/// `max_utf8_buffer_length_with_replacement()`. When decoding to UTF-16 with
/// or without replacement, the length of the output buffer that guarantees
/// the methods not to return indicating that more output space is needed is
/// given by `max_utf16_buffer_length()`.
///
/// When encoding from UTF-8, each `src` buffer _must_ be valid UTF-8. (When
/// calling from Rust, the type system takes care of this.) When encoding from
/// UTF-16, unpaired surrogates in the input are treated as U+FFFD REPLACEMENT
/// CHARACTERS. Therefore, in order for astral characters not to turn into a
/// pair of REPLACEMENT CHARACTERS, the caller must ensure that surrogate pairs
/// are not split across input buffer boundaries.
///
/// Except in the case of ISO-2022-JP, the output of each `encode_*` call is
/// guaranteed to consist of a valid byte sequence of complete characters.
/// (I.e. the code unit sequence for the last character is guaranteed not to be
/// split across output buffers.)
///
/// The boolean argument `last` indicates that the end of the stream is reached
/// when all the characters in `src` have been consumed. This argument is needed
/// for ISO-2022-JP and is ignored for other encodings.
///
/// An `Encoder` object can be used to incrementally encode a byte stream. An
/// ISO-2022-JP encoder cannot be used for multiple streams concurrently but
/// can be used for multiple streams sequentially. (The other encoders are
/// stateless.)
///
/// During the processing of a single stream, the caller must call `encode_*`
/// zero or more times with `last` set to `false` and then call `encode_*` at
/// least once with `last` set to `true`. If `encode_*` returns `Underflow`
/// (`true` in the `*_with_replacement` case), the processing of the stream
/// has ended. Otherwise, the caller must call `encode_*` again with `last`
/// set to `true` (or treat an `Unmappable` result as a fatal error). (If you
/// know that the encoder is not an ISO-2022-JP encoder, you may ignore this
/// paragraph and treat the encoder as stateless.)
///
/// The encoder is ready to start processing a new stream when it has
/// returned `Underflow` (`true` in the `*_with_replacement` case) from a call
/// where `last` was set to `true`. In other cases, if the caller wishes to
/// stop processing the current stream and to start processing a new stream,
/// the caller must call `reset()` before starting processing the new stream.
/// (If you know that the encoder is not an ISO-2022-JP encoder, you may ignore
/// this paragraph and treat the encoder as stateless.)
///
/// When the encoder returns `Overflow` (`false` in the `*_with_replacement`
/// case) or the encoder returns `Unmappable` and the caller does not wish to
/// treat it as a fatal error, the input buffer `src` may not have been
/// completely consumed. In that case, the caller must pass the unconsumed
/// contents of `src` to `encode_*` again upon the next call.
pub trait Encoder {
    /// Make the encoder ready to process a new stream. (No-op for all encoders
    /// other than the ISO-2022-JP encoder.)
    fn reset(&mut self) {
    }

    /// Query the worst-case output size when encoding from UTF-16 without
    /// replacement.
    ///
    /// Returns the size of the output buffer in bytes that will not overflow
    /// given the current state of the encoder and `u16_length` number of
    /// additional input code units.
    ///
    /// Available via the C wrapper.
    fn max_buffer_length_from_utf16(&self, u16_length: usize) -> usize;

    /// Query the worst-case output size when encoding from UTF-8 without
    /// replacement.
    ///
    /// Returns the size of the output buffer in bytes that will not overflow
    /// given the current state of the encoder and `byte_length` number of
    /// additional input code units.
    ///
    /// Available via the C wrapper.
    fn max_buffer_length_from_utf8(&self, byte_length: usize) -> usize;

    /// Query the worst-case output size when encoding from UTF-16 with
    /// replacement.
    ///
    /// Returns the size of the output buffer in bytes that will not overflow
    /// given the current state of the encoder and `u16_length` number of
    /// additional input code units.
    ///
    /// Available via the C wrapper.
    fn max_buffer_length_from_utf16_with_replacement(&self, u16_length: usize) -> usize;

    /// Query the worst-case output size when encoding from UTF-8 with
    /// replacement.
    ///
    /// Returns the size of the output buffer in bytes that will not overflow
    /// given the current state of the encoder and `byte_length` number of
    /// additional input code units.
    ///
    /// Available via the C wrapper.
    fn max_buffer_length_from_utf8_with_replacement(&self, byte_length: usize) -> usize;

    /// Incrementally encode into byte stream from UTF-16.
    ///
    /// See the documentation of the trait for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    fn encode_from_utf16(&mut self, src: &[u16], dst: &mut [u8], last: bool) -> (EncoderResult, usize, usize);

    /// Incrementally encode into byte stream from UTF-8.
    ///
    /// See the documentation of the trait for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    fn encode_from_utf8(&mut self, src: &str, dst: &mut [u8], last: bool) -> (EncoderResult, usize, usize);

    /// Incrementally encode into byte stream from UTF-16 with replacement.
    ///
    /// See the documentation of the trait for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    fn encode_from_utf16_with_replacement(&mut self, src: &[u16], dst: &mut [u8], last: bool) -> (bool, usize, usize, bool) {
        // XXX
        (true, 0, 0, false)
    }

    /// Incrementally encode into byte stream from UTF-8 with replacement.
    ///
    /// See the documentation of the trait for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    fn encode_from_utf8_with_replacement(&mut self, src: &str, dst: &mut [u8], last: bool) -> (bool, usize, usize, bool) {
        // XXX
        (true, 0, 0, false)
    }

    // XXX: _to_vec variants for all these?
}


// ############## C API ###############

#[no_mangle]
pub unsafe extern fn decoder_decode_to_utf16(decoder: &mut Decoder, src: *const u8, src_len: *mut usize, dst: *mut u16, dst_len: *mut usize, last: bool) -> DecoderResult {
    let src_slice = std::slice::from_raw_parts(src, *src_len);
    let dst_slice = std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written) = decoder.decode_to_utf16(src_slice, dst_slice, last);
    *src_len = read;
    *dst_len = written;
    result
}

#[no_mangle]
pub unsafe extern fn decoder_decode_to_utf8(decoder: &mut Decoder, src: *const u8, src_len: *mut usize, dst: *mut u8, dst_len: *mut usize, last: bool) -> DecoderResult {
    let src_slice = std::slice::from_raw_parts(src, *src_len);
    let dst_slice = std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written) = decoder.decode_to_utf8(src_slice, dst_slice, last);
    *src_len = read;
    *dst_len = written;
    result
}

#[no_mangle]
pub unsafe extern fn decoder_decode_to_utf16_with_replacement(decoder: &mut Decoder, src: *const u8, src_len: *mut usize, dst: *mut u16, dst_len: *mut usize, last: bool, had_replacements: *mut bool) -> bool {
    let src_slice = std::slice::from_raw_parts(src, *src_len);
    let dst_slice = std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written, replaced) = decoder.decode_to_utf16_with_replacement(src_slice, dst_slice, last);
    *src_len = read;
    *dst_len = written;
    *had_replacements = replaced;
    result
}

#[no_mangle]
pub unsafe extern fn decoder_decode_to_utf8_with_replacement(decoder: &mut Decoder, src: *const u8, src_len: *mut usize, dst: *mut u8, dst_len: *mut usize, last: bool, had_replacements: *mut bool) -> bool {
    let src_slice = std::slice::from_raw_parts(src, *src_len);
    let dst_slice = std::slice::from_raw_parts_mut(dst, *dst_len);
    let (result, read, written, replaced) = decoder.decode_to_utf8_with_replacement(src_slice, dst_slice, last);
    *src_len = read;
    *dst_len = written;
    *had_replacements = replaced;
    result
}

#[no_mangle]
pub extern fn decoder_max_utf16_length(decoder: &Decoder, byte_length: usize) -> usize {
    decoder.max_utf16_buffer_length(byte_length)
}

#[no_mangle]
pub extern fn decoder_max_utf8_length(decoder: &Decoder, byte_length: usize) -> usize {
    decoder.max_utf8_buffer_length(byte_length)
}

#[no_mangle]
pub extern fn decoder_max_utf8_length_with_replacement(decoder: &Decoder, byte_length: usize) -> usize {
    decoder.max_utf8_buffer_length_with_replacement(byte_length)
}

// ############## TESTS ###############

#[test]
fn it_works() {
}
