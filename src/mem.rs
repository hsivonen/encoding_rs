// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Functions for converting between different in-RAM representations of text.
//!
//! By using slices for output, the functions here seek to enable by-register
//! (ALU register or SIMD register as available) operations in order to
//! outperform iterator-based conversions available in the Rust standard
//! library.
//!
//! _Note:_ "Latin1" in this module refers to the Unicode range from U+0000 to
//! U+00FF, inclusive, and does not refer to the windows-1252 range. This
//! in-memory encoding is sometimes used as a storage optimization of text
//! when UTF-16 indexing and length semantics are exposed.

use ascii::*;
use super::in_inclusive_range8;
use super::DecoderResult;
use super::EncoderResult;
use utf_8::Utf8Decoder;
use utf_8::Utf8Encoder;

// `as` truncates, so works on 32-bit, too.
const LATIN1_MASK: usize = 0xFF00FF00_FF00FF00u64 as usize;

#[allow(unused_macros)]
macro_rules! by_unit_check_alu {
    ($name:ident,
     $unit:ty,
     $mask:ident) => (
    #[inline(always)]
    fn $name(buffer: &[$unit]) -> bool {
        let unit_size = ::std::mem::size_of::<$unit>();
        let src = buffer.as_ptr();
        let len = buffer.len();
        let mut offset = 0usize;
        let mut until_alignment = ((ALIGNMENT - ((src as usize) & ALIGNMENT_MASK)) &
                                   ALIGNMENT_MASK) / unit_size;
        let mut accu = 0usize;
        if until_alignment + ALIGNMENT / unit_size <= len {
            while until_alignment != 0 {
                accu |= buffer[offset] as usize;
                offset += 1;
                until_alignment -= 1;
            }
            let len_minus_stride = len - ALIGNMENT / unit_size;
            loop {
                accu |= unsafe { *(src.offset(offset as isize) as *const usize) };
                offset += ALIGNMENT / unit_size;
                if offset > len_minus_stride {
                    break;
                }
            }
        }
        while offset < len {
            accu |= buffer[offset] as usize;
            offset += 1;
        }
        accu & $mask == 0
    })
}

#[allow(unused_macros)]
macro_rules! by_unit_check_simd {
    ($name:ident,
     $unit:ty,
     $splat:expr,
     $simd_ty:ty,
     $mask:ident,
     $func:ident) => (
    #[inline(always)]
    fn $name(buffer: &[$unit]) -> bool {
        let unit_size = ::std::mem::size_of::<$unit>();
        let src = buffer.as_ptr();
        let len = buffer.len();
        let mut offset = 0usize;
        let mut until_alignment = ((SIMD_ALIGNMENT - ((src as usize) & SIMD_ALIGNMENT_MASK)) &
                                   SIMD_ALIGNMENT_MASK) / unit_size;
        let mut accu = 0usize;
        if until_alignment + STRIDE_SIZE / unit_size <= len {
            while until_alignment != 0 {
                accu |= buffer[offset] as usize;
                offset += 1;
                until_alignment -= 1;
            }
            let len_minus_stride = len - STRIDE_SIZE / unit_size;
            let mut simd_accu = $splat;
            loop {
                simd_accu = simd_accu | unsafe { *(src.offset(offset as isize) as *const $simd_ty) };
                offset += STRIDE_SIZE / unit_size;
                if offset > len_minus_stride {
                    break;
                }
            }
            // TODO: Could optimize away this branch on aarch64 and arm by
            // ORing the horizontal max into accu.
            if !$func(simd_accu) {
                return false;
            }
        }
        while offset < len {
            accu |= buffer[offset] as usize;
            offset += 1;
        }
        accu & $mask == 0
    })
}

cfg_if!{
    if #[cfg(all(feature = "simd-accel", any(target_feature = "sse2", all(target_endian = "little", target_arch = "aarch64"))))] {
        use simd_funcs::*;
        use simd::u8x16;
        use simd::u16x8;

        const SIMD_ALIGNMENT: usize = 16;

        const SIMD_ALIGNMENT_MASK: usize = 15;

        by_unit_check_simd!(is_ascii_impl, u8, u8x16::splat(0), u8x16, ASCII_MASK, simd_is_ascii);
        by_unit_check_simd!(is_basic_latin_impl, u16, u16x8::splat(0), u16x8, BASIC_LATIN_MASK, simd_is_basic_latin);
        by_unit_check_simd!(is_utf16_latin1_impl, u16, u16x8::splat(0), u16x8, LATIN1_MASK, simd_is_latin1);

        #[inline(always)]
        fn utf16_valid_up_to_impl(buffer: &[u16]) -> usize {
            // This function is a mess, because it simultaneously tries to do
            // only aligned SIMD (perhaps misguidedly) and needs to deal with
            // the last code unit in a SIMD stride being part of a valid
            // surrogate pair.
            let unit_size = ::std::mem::size_of::<u16>();
            let src = buffer.as_ptr();
            let len = buffer.len();
            let mut offset = 0usize;
            'outer: loop {
                let until_alignment = ((SIMD_ALIGNMENT - ((unsafe { src.offset(offset as isize) } as usize) & SIMD_ALIGNMENT_MASK)) &
                                        SIMD_ALIGNMENT_MASK) / unit_size;
                let offset_plus_until_alignment = offset + until_alignment;
                let offset_plus_until_alignment_plus_one = offset_plus_until_alignment + 1;
                if offset_plus_until_alignment_plus_one + STRIDE_SIZE / unit_size > len {
                    break;
                }
                let (up_to, last_valid_low) = utf16_valid_up_to_alu(&buffer[offset..offset_plus_until_alignment_plus_one]);
                if up_to != until_alignment + 1 {
                    return offset + up_to;
                }
                if last_valid_low {
                    offset = offset_plus_until_alignment_plus_one;
                    continue;
                }
                let len_minus_stride = len - STRIDE_SIZE / unit_size;
                offset = offset_plus_until_alignment;
                'inner: loop {
                    let offset_plus_stride = offset + STRIDE_SIZE / unit_size;
                    if contains_surrogates(unsafe { *(src.offset(offset as isize) as *const u16x8) }) {
                        if offset_plus_stride == len {
                            break 'outer;
                        }
                        let offset_plus_stride_plus_one = offset_plus_stride + 1;
                        let (up_to, last_valid_low) = utf16_valid_up_to_alu(&buffer[offset..offset_plus_stride_plus_one]);
                        if up_to != STRIDE_SIZE / unit_size + 1 {
                            return offset + up_to;
                        }
                        if last_valid_low {
                            offset = offset_plus_stride_plus_one;
                            continue 'outer;
                        }
                    }
                    offset = offset_plus_stride;
                    if offset > len_minus_stride {
                        break 'outer;
                    }
                }
            }
            let (up_to, _) = utf16_valid_up_to_alu(&buffer[offset..]);
            offset + up_to
        }
    } else {
        by_unit_check_alu!(is_ascii_impl, u8, ASCII_MASK);
        by_unit_check_alu!(is_basic_latin_impl, u16, BASIC_LATIN_MASK);
        by_unit_check_alu!(is_utf16_latin1_impl, u16, LATIN1_MASK);

        #[inline(always)]
        fn utf16_valid_up_to_impl(buffer: &[u16]) -> usize {
            let (up_to, _) = utf16_valid_up_to_alu(buffer);
            up_to
        }
    }
}

/// The second return value is true iff the last code unit examined is a low
/// surrogate that is part of a valid pair.
#[inline(always)]
fn utf16_valid_up_to_alu(buffer: &[u16]) -> (usize, bool) {
    let len = buffer.len();
    if len == 0 {
        return (0, false);
    }
    let mut offset = 0usize;
    loop {
        let unit = buffer[offset];
        let next = offset + 1;
        let unit_minus_surrogate_start = unit.wrapping_sub(0xD800);
        if unit_minus_surrogate_start > (0xDFFF - 0xD800) {
            // Not a surrogate
            offset = next;
            if offset == len {
                return (offset, false);
            }
            continue;
        }
        if unit_minus_surrogate_start <= (0xDBFF - 0xD800) {
            // high surrogate
            if next < len {
                let second = buffer[next];
                let second_minus_low_surrogate_start = second.wrapping_sub(0xDC00);
                if second_minus_low_surrogate_start <= (0xDFFF - 0xDC00) {
                    // The next code unit is a low surrogate. Advance position.
                    offset = next + 1;
                    if offset == len {
                        return (offset, true);
                    }
                    continue;
                }
                // The next code unit is not a low surrogate. Don't advance
                // position and treat the high surrogate as unpaired.
                // fall through
            }
            // Unpaired, fall through
        }
        // Unpaired surrogate
        return (offset, false);
    }
}

/// Checks whether the buffer is all-ASCII.
///
/// May read the entire buffer even if it isn't all-ASCII. (I.e. the function
/// is not guaranteed to fail fast.)
#[inline]
pub fn is_ascii(buffer: &[u8]) -> bool {
    is_ascii_impl(buffer)
}

/// Checks whether the buffer is all-Basic Latin (i.e. UTF-16 representing
/// only ASCII characters).
///
/// May read the entire buffer even if it isn't all-ASCII. (I.e. the function
/// is not guaranteed to fail fast.)
#[inline]
pub fn is_basic_latin(buffer: &[u16]) -> bool {
    is_basic_latin_impl(buffer)
}

/// Checks whether the buffer is valid UTF-8 representing only code points
/// less than or equal to U+00FF.
///
/// Fails fast. (I.e. returns before having read the whole buffer if UTF-8
/// invalidity or code points above U+00FF are discovered.
#[inline]
pub fn is_utf8_latin1(buffer: &[u8]) -> bool {
    let mut bytes = buffer;
    loop {
        if let Some((byte, offset)) = validate_ascii(bytes) {
            if in_inclusive_range8(byte, 0xC2, 0xC3) {
                let next = offset + 1;
                if next == bytes.len() {
                    return false;
                }
                if bytes[next] & 0xC0 != 0x80 {
                    return false;
                }
                bytes = &bytes[offset + 2..];
            } else {
                return false;
            }
        } else {
            return true;
        }
    }
}

/// Checks whether the buffer represents only code point less than or equal
/// to U+00FF.
///
/// Fails fast. (I.e. returns before having read the whole buffer if code
/// points above U+00FF are discovered.
#[inline]
pub fn is_str_latin1(buffer: &str) -> bool {
    let mut bytes = buffer.as_bytes();
    loop {
        if let Some((byte, offset)) = validate_ascii(bytes) {
            if byte > 0xC3 {
                return false;
            }
            bytes = &bytes[offset + 2..];
        } else {
            return true;
        }
    }
}

/// Checks whether the buffer represents only code point less than or equal
/// to U+00FF.
///
/// May read the entire buffer even if it isn't all-Latin1. (I.e. the function
/// is not guaranteed to fail fast.)
#[inline]
pub fn is_utf16_latin1(buffer: &[u16]) -> bool {
    is_utf16_latin1_impl(buffer)
}

/// Converts potentially-invalid UTF-8 to valid UTF-16 with errors replaced
/// with the REPLACEMENT CHARACTER.
///
/// The length of the destination buffer must be at least the length of the
/// source buffer _plus one_.
///
/// Returns the number of `u16`s written.
///
/// # Panics
///
/// Panics if the destination buffer is shorter than stated above.
#[inline]
pub fn convert_utf8_to_utf16(src: &[u8], dst: &mut [u16]) -> usize {
    // TODO: Can the + 1 be eliminated?
    assert!(dst.len() >= src.len() + 1);
    let mut decoder = Utf8Decoder::new_inner();
    let mut total_read = 0usize;
    let mut total_written = 0usize;
    loop {
        let (result, read, written) =
            decoder.decode_to_utf16_raw(&src[total_read..], &mut dst[total_written..], true);
        total_read += read;
        total_written += written;
        match result {
            DecoderResult::InputEmpty => {
                return total_written;
            }
            DecoderResult::OutputFull => {
                unreachable!("The assert at the top of the function should have caught this.");
            }
            DecoderResult::Malformed(_, _) => {
                // There should always be space for the U+FFFD, because
                // otherwise we'd have gotten OutputFull already.
                dst[total_written] = 0xFFFD;
                total_written += 1;
            }
        }
    }
}

/// Converts valid UTF-8 to valid UTF-16 with errors replaced with the
/// REPLACEMENT CHARACTER.
///
/// The length of the destination buffer must be at least the length of the
/// source buffer.
///
/// Returns the number of `u16`s written.
///
/// # Panics
///
/// Panics if the destination buffer is shorter than stated above.
#[inline]
pub fn convert_str_to_utf16(src: &str, dst: &mut [u16]) -> usize {
    assert!(
        dst.len() >= src.len(),
        "Destination must not be shorter than the source."
    );
    let bytes = src.as_bytes();
    let src_len = src.len();
    let src_ptr = src.as_ptr();
    let dst_ptr = dst.as_mut_ptr();
    let mut total_read = 0usize;
    let mut total_written = 0usize;
    loop {
        // dst can't advance more than src
        let src_left = src_len - total_read;
        if let Some((non_ascii, consumed)) =
            unsafe {
                ascii_to_basic_latin(
                    src_ptr.offset(total_read as isize),
                    dst_ptr.offset(total_written as isize),
                    src_left,
                )
            } {
            total_read += consumed;
            total_written += consumed;

            let unit = non_ascii as u32;
            if unit < 0xE0u32 {
                let point = ((unit & 0x1Fu32) << 6) | (bytes[total_read + 1] as u32 & 0x3Fu32);
                total_read += 2;
                dst[total_written] = point as u16;
                total_written += 1;
                continue;
            }
            if unit < 0xF0u32 {
                let point = ((unit & 0xFu32) << 12) |
                            ((bytes[total_read + 1] as u32 & 0x3Fu32) << 6) |
                            (bytes[total_read + 2] as u32 & 0x3Fu32);
                total_read += 3;
                dst[total_written] = point as u16;
                total_written += 1;
                continue;
            }
            let point = ((unit & 0x7u32) << 18) | ((bytes[total_read + 1] as u32 & 0x3Fu32) << 12) |
                        ((bytes[total_read + 2] as u32 & 0x3Fu32) << 6) |
                        (bytes[total_read + 3] as u32 & 0x3Fu32);
            total_read += 4;
            dst[total_written] = (0xD7C0 + (point >> 10)) as u16;
            total_written += 1;
            dst[total_written] = (0xDC00 + (point & 0x3FF)) as u16;
            total_written += 1;
            continue;
        }
        return total_written + src_left;
    }
}

/// Converts potentially-invalid UTF-16 to valid UTF-8 with errors replaced
/// with the REPLACEMENT CHARACTER.
///
/// The length of the destination buffer must be at least the length of the
/// source buffer times three _plus one_.
///
/// Returns the number of bytes written.
///
/// # Panics
///
/// Panics if the destination buffer is shorter than stated above.
///
/// # Safety
///
/// Note that this function may write garbage beyond the number of bytes
/// indicated by the return value, so using a `&mut str` interpreted as
/// `&mut [u8]` as the destination is not safe. If you want to convert into
/// a `&mut str`, use `convert_utf16_to_str()` instead of this function.
#[inline]
pub fn convert_utf16_to_utf8(src: &[u16], dst: &mut [u8]) -> usize {
    assert!(dst.len() >= src.len() * 3 + 1);
    let mut encoder = Utf8Encoder;
    let (result, _, written) = encoder.encode_from_utf16_raw(src, dst, true);
    debug_assert!(result == EncoderResult::InputEmpty);
    written
}

/// Converts potentially-invalid UTF-16 to valid UTF-8 with errors replaced
/// with the REPLACEMENT CHARACTER such that the validity of the output is
/// signaled using the Rust type system.
///
/// The length of the destination buffer must be at least the length of the
/// source buffer times three _plus one_.
///
/// Returns the number of bytes written.
///
/// # Panics
///
/// Panics if the destination buffer is shorter than stated above.
#[inline]
pub fn convert_utf16_to_str(src: &[u16], dst: &mut str) -> usize {
    let bytes: &mut [u8] = unsafe { ::std::mem::transmute(dst) };
    let written = convert_utf16_to_utf8(src, bytes);
    let len = bytes.len();
    let mut trail = written;
    let max = ::std::cmp::min(len, trail + STRIDE_SIZE);
    while trail < max {
        bytes[trail] = 0;
        trail += 1;
    }
    while trail < len && ((bytes[trail] & 0xC0) == 0x80) {
        bytes[trail] = 0;
        trail += 1;
    }
    written
}

/// Converts bytes whose unsigned value is interpreted as Unicode code point
/// (i.e. U+0000 to U+00FF, inclusive) to UTF-16.
///
/// The length of the destination buffer must be at least the length of the
/// source buffer.
///
/// The number of `u16`s written equals the length of the source buffer.
///
/// # Panics
///
/// Panics if the destination buffer is shorter than stated above.
#[inline]
pub fn convert_latin1_to_utf16(src: &[u8], dst: &mut [u16]) {
    assert!(
        dst.len() >= src.len(),
        "Destination must not be shorter than the source."
    );
    unsafe {
        unpack_latin1(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
}

/// Converts bytes whose unsigned value is interpreted as Unicode code point
/// (i.e. U+0000 to U+00FF, inclusive) to UTF-8.
///
/// The length of the destination buffer must be at least the length of the
/// source buffer times two.
///
/// Returns the number of bytes written.
///
/// # Panics
///
/// Panics if the destination buffer is shorter than stated above.
///
/// # Safety
///
/// Note that this function may write garbage beyond the number of bytes
/// indicated by the return value, so using a `&mut str` interpreted as
/// `&mut [u8]` as the destination is not safe. If you want to convert into
/// a `&mut str`, use `convert_utf16_to_str()` instead of this function.
#[inline]
pub fn convert_latin1_to_utf8(src: &[u8], dst: &mut [u8]) -> usize {
    assert!(
        dst.len() >= src.len() * 2,
        "Destination must not be shorter than the source times two."
    );
    let src_len = src.len();
    let src_ptr = src.as_ptr();
    let dst_ptr = dst.as_mut_ptr();
    let mut total_read = 0usize;
    let mut total_written = 0usize;
    loop {
        // src can't advance more than dst
        let src_left = src_len - total_read;
        if let Some((non_ascii, consumed)) =
            unsafe {
                ascii_to_ascii(
                    src_ptr.offset(total_read as isize),
                    dst_ptr.offset(total_written as isize),
                    src_left,
                )
            } {
            total_read += consumed + 1;
            total_written += consumed;

            let code_point = non_ascii as u32;
            dst[total_written] = ((code_point >> 6) | 0xC0u32) as u8;
            total_written += 1;
            dst[total_written] = ((code_point as u32 & 0x3Fu32) | 0x80u32) as u8;
            total_written += 1;
            continue;
        }
        return total_written + src_left;
    }
}

/// Converts bytes whose unsigned value is interpreted as Unicode code point
/// (i.e. U+0000 to U+00FF, inclusive) to UTF-8 such that the validity of the
/// output is signaled using the Rust type system.
///
/// The length of the destination buffer must be at least the length of the
/// source buffer times two.
///
/// Returns the number of bytes written.
///
/// # Panics
///
/// Panics if the destination buffer is shorter than stated above.
#[inline]
pub fn convert_latin1_to_str(src: &[u8], dst: &mut str) -> usize {
    let bytes: &mut [u8] = unsafe { ::std::mem::transmute(dst) };
    let written = convert_latin1_to_utf8(src, bytes);
    let len = bytes.len();
    let mut trail = written;
    let max = ::std::cmp::min(len, trail + STRIDE_SIZE);
    while trail < max {
        bytes[trail] = 0;
        trail += 1;
    }
    while trail < len && ((bytes[trail] & 0xC0) == 0x80) {
        bytes[trail] = 0;
        trail += 1;
    }
    written
}

/// If the input is valid UTF-8 representing only Unicode code points from
/// U+0000 to U+00FF, inclusive, converts the input into output that
/// represents the value of each code point as the unsigned byte value of
/// each output byte.
///
/// If the input does not fulfill the condition stated above, this function
/// does something that is memory-safe without any promises about any
/// properties of the output. In particular, callers shouldn't assume the
/// output to be the same across crate versions or CPU architectures and
/// should not assume that non-ASCII input can't map to ASCII output.
///
/// The length of the destination buffer must be at least the length of the
/// source buffer.
///
/// Returns the number of bytes written.
///
/// # Panics
///
/// Panics if the destination buffer is shorter than stated above.
#[inline]
pub fn convert_utf8_to_latin1_lossy(src: &[u8], dst: &mut [u8]) -> usize {
    assert!(
        dst.len() >= src.len(),
        "Destination must not be shorter than the source."
    );
    let src_len = src.len();
    let src_ptr = src.as_ptr();
    let dst_ptr = dst.as_mut_ptr();
    let mut total_read = 0usize;
    let mut total_written = 0usize;
    loop {
        // dst can't advance more than src
        let src_left = src_len - total_read;
        if let Some((non_ascii, consumed)) =
            unsafe {
                ascii_to_ascii(
                    src_ptr.offset(total_read as isize),
                    dst_ptr.offset(total_written as isize),
                    src_left,
                )
            } {
            total_read += consumed + 1;
            total_written += consumed;

            if total_read == src_len {
                return total_written;
            }

            let trail = src[total_read];
            total_read += 1;

            dst[total_written] = (((non_ascii as u32 & 0x1Fu32) << 6) |
                                  (trail as u32 & 0x3Fu32)) as u8;
            total_written += 1;
            continue;
        }
        return total_written + src_left;
    }
}

/// If the input is valid UTF-16 representing only Unicode code points from
/// U+0000 to U+00FF, inclusive, converts the input into output that
/// represents the value of each code point as the unsigned byte value of
/// each output byte.
///
/// If the input does not fulfill the condition stated above, this function
/// does something that is memory-safe without any promises about any
/// properties of the output. In particular, callers shouldn't assume the
/// output to be the same across crate versions or CPU architectures and
/// should not assume that non-Basic Latin input can't map to ASCII output.
///
/// The length of the destination buffer must be at least the length of the
/// source buffer.
///
/// The number of bytes written equals the length of the source buffer.
///
/// # Panics
///
/// Panics if the destination buffer is shorter than stated above.
#[inline]
pub fn convert_utf16_to_latin1_lossy(src: &[u16], dst: &mut [u8]) {
    assert!(
        dst.len() >= src.len(),
        "Destination must not be shorter than the source."
    );
    unsafe {
        pack_latin1(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
}

/// Returns the index of the first unpaired surrogate or, if the input is
/// valid UTF-16 in its entirety, the length of the input.
#[inline]
pub fn utf16_valid_up_to(buffer: &[u16]) -> usize {
    utf16_valid_up_to_impl(buffer)
}

/// Replaces unpaired surrogates in the input with the REPLACEMENT CHARACTER.
#[inline]
pub fn ensure_utf16_validity(buffer: &mut [u16]) {
    let mut offset = 0;
    loop {
        offset += utf16_valid_up_to(&buffer[offset..]);
        if offset == buffer.len() {
            return;
        }
        buffer[offset] = 0xFFFD;
        offset += 1;
    }
}

/// Copies ASCII from source to destination up to the first non-ASCII byte
/// (or the end of the input if it is ASCII in its entirety).
///
/// The length of the destination buffer must be at least the length of the
/// source buffer.
///
/// Returns the number of bytes written.
///
/// # Panics
///
/// Panics if the destination buffer is shorter than stated above.
#[inline]
pub fn copy_ascii_to_ascii(src: &[u8], dst: &mut [u8]) -> usize {
    assert!(
        dst.len() >= src.len(),
        "Destination must not be shorter than the source."
    );
    if let Some((_, consumed)) =
        unsafe { ascii_to_ascii(src.as_ptr(), dst.as_mut_ptr(), src.len()) } {
        consumed
    } else {
        src.len()
    }
}

/// Copies ASCII from source to destination zero-extending it to UTF-16 up to
/// the first non-ASCII byte (or the end of the input if it is ASCII in its
/// entirety).
///
/// The length of the destination buffer must be at least the length of the
/// source buffer.
///
/// Returns the number of `u16`s written.
///
/// # Panics
///
/// Panics if the destination buffer is shorter than stated above.
#[inline]
pub fn copy_ascii_to_basic_latin(src: &[u8], dst: &mut [u16]) -> usize {
    assert!(
        dst.len() >= src.len(),
        "Destination must not be shorter than the source."
    );
    if let Some((_, consumed)) =
        unsafe { ascii_to_basic_latin(src.as_ptr(), dst.as_mut_ptr(), src.len()) } {
        consumed
    } else {
        src.len()
    }
}

/// Copies Basic Latin from source to destination narrowing it to ASCII up to
/// the first non-Basic Latin code unit (or the end of the input if it is
/// Basic Latin in its entirety).
///
/// The length of the destination buffer must be at least the length of the
/// source buffer.
///
/// Returns the number of bytes written.
///
/// # Panics
///
/// Panics if the destination buffer is shorter than stated above.
#[inline]
pub fn copy_basic_latin_to_ascii(src: &[u16], dst: &mut [u8]) -> usize {
    assert!(
        dst.len() >= src.len(),
        "Destination must not be shorter than the source."
    );
    if let Some((_, consumed)) =
        unsafe { basic_latin_to_ascii(src.as_ptr(), dst.as_mut_ptr(), src.len()) } {
        consumed
    } else {
        src.len()
    }
}

// Any copyright to the test code below this comment is dedicated to the
// Public Domain. http://creativecommons.org/publicdomain/zero/1.0/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ascii_success() {
        let mut src: Vec<u8> = Vec::with_capacity(128);
        src.resize(128, 0);
        for i in 0..src.len() {
            src[i] = i as u8;
        }
        for i in 0..src.len() {
            assert!(is_ascii(&src[i..]));
        }
    }

    #[test]
    fn test_is_ascii_fail() {
        let mut src: Vec<u8> = Vec::with_capacity(128);
        src.resize(128, 0);
        for i in 0..src.len() {
            src[i] = i as u8;
        }
        for i in 0..src.len() {
            let tail = &mut src[i..];
            for j in 0..tail.len() {
                tail[j] = 0xA0;
                assert!(!is_ascii(tail));
            }
        }
    }

    #[test]
    fn test_is_basic_latin_success() {
        let mut src: Vec<u16> = Vec::with_capacity(128);
        src.resize(128, 0);
        for i in 0..src.len() {
            src[i] = i as u16;
        }
        for i in 0..src.len() {
            assert!(is_basic_latin(&src[i..]));
        }
    }

    #[test]
    fn test_is_basic_latin_fail() {
        let mut src: Vec<u16> = Vec::with_capacity(128);
        src.resize(128, 0);
        for i in 0..src.len() {
            src[i] = i as u16;
        }
        for i in 0..src.len() {
            let tail = &mut src[i..];
            for j in 0..tail.len() {
                tail[j] = 0xA0;
                assert!(!is_basic_latin(tail));
            }
        }
    }

    #[test]
    fn test_is_utf16_latin1_success() {
        let mut src: Vec<u16> = Vec::with_capacity(256);
        src.resize(256, 0);
        for i in 0..src.len() {
            src[i] = i as u16;
        }
        for i in 0..src.len() {
            assert!(is_utf16_latin1(&src[i..]));
        }
    }

    #[test]
    fn test_is_utf16_latin1_fail() {
        let mut src: Vec<u16> = Vec::with_capacity(256);
        src.resize(256, 0);
        for i in 0..src.len() {
            src[i] = i as u16;
        }
        for i in 0..src.len() {
            let tail = &mut src[i..];
            for j in 0..tail.len() {
                tail[j] = 0x100 + j as u16;
                assert!(!is_utf16_latin1(tail));
            }
        }
    }

    #[test]
    fn test_is_str_latin1_success() {
        let mut src: Vec<u16> = Vec::with_capacity(256);
        src.resize(256, 0);
        for i in 0..src.len() {
            src[i] = i as u16;
        }
        for i in 0..src.len() {
            let s = String::from_utf16(&src[i..]).unwrap();
            assert!(is_str_latin1(&s[..]));
        }
    }

    #[test]
    fn test_is_str_latin1_fail() {
        let mut src: Vec<u16> = Vec::with_capacity(256);
        src.resize(256, 0);
        for i in 0..src.len() {
            src[i] = i as u16;
        }
        for i in 0..src.len() {
            let tail = &mut src[i..];
            for j in 0..tail.len() {
                tail[j] = 0x100 + j as u16;
                let s = String::from_utf16(tail).unwrap();
                assert!(!is_str_latin1(&s[..]));
            }
        }
    }

    #[test]
    fn test_is_utf8_latin1_success() {
        let mut src: Vec<u16> = Vec::with_capacity(256);
        src.resize(256, 0);
        for i in 0..src.len() {
            src[i] = i as u16;
        }
        for i in 0..src.len() {
            let s = String::from_utf16(&src[i..]).unwrap();
            assert!(is_utf8_latin1(s.as_bytes()));
        }
    }

    #[test]
    fn test_is_utf8_latin1_fail() {
        let mut src: Vec<u16> = Vec::with_capacity(256);
        src.resize(256, 0);
        for i in 0..src.len() {
            src[i] = i as u16;
        }
        for i in 0..src.len() {
            let tail = &mut src[i..];
            for j in 0..tail.len() {
                tail[j] = 0x100 + j as u16;
                let s = String::from_utf16(tail).unwrap();
                assert!(!is_utf8_latin1(s.as_bytes()));
            }
        }
    }

    #[test]
    fn test_is_utf8_latin1_invalid() {
        assert!(!is_utf8_latin1(b"\xC3"));
        assert!(!is_utf8_latin1(b"a\xC3"));
        assert!(!is_utf8_latin1(b"\xFF"));
        assert!(!is_utf8_latin1(b"a\xFF"));
        assert!(!is_utf8_latin1(b"\xC3\xFF"));
        assert!(!is_utf8_latin1(b"a\xC3\xFF"));
    }

    #[test]
    fn test_convert_utf8_to_utf16() {
        let src = "abcdefghijklmnopqrstu\u{1F4A9}v\u{2603}w\u{00B6}xyzz";
        let mut dst: Vec<u16> = Vec::with_capacity(src.len() + 1);
        dst.resize(src.len() + 1, 0);
        let len = convert_utf8_to_utf16(src.as_bytes(), &mut dst[..]);
        dst.truncate(len);
        let reference: Vec<u16> = src.encode_utf16().collect();
        assert_eq!(dst, reference);
    }

    #[test]
    fn test_convert_str_to_utf16() {
        let src = "abcdefghijklmnopqrstu\u{1F4A9}v\u{2603}w\u{00B6}xyzz";
        let mut dst: Vec<u16> = Vec::with_capacity(src.len());
        dst.resize(src.len(), 0);
        let len = convert_str_to_utf16(src, &mut dst[..]);
        dst.truncate(len);
        let reference: Vec<u16> = src.encode_utf16().collect();
        assert_eq!(dst, reference);
    }

    #[test]
    fn test_convert_utf16_to_utf8() {
        let reference = "abcdefghijklmnopqrstu\u{1F4A9}v\u{2603}w\u{00B6}xyzz";
        let src: Vec<u16> = reference.encode_utf16().collect();
        let mut dst: Vec<u8> = Vec::with_capacity(src.len() * 3 + 1);
        dst.resize(src.len() * 3 + 1, 0);
        let len = convert_utf16_to_utf8(&src[..], &mut dst[..]);
        dst.truncate(len);
        assert_eq!(dst, reference.as_bytes());
    }

    #[test]
    fn test_convert_latin1_to_utf16() {
        let mut src: Vec<u8> = Vec::with_capacity(256);
        src.resize(256, 0);
        let mut reference: Vec<u16> = Vec::with_capacity(256);
        reference.resize(256, 0);
        for i in 0..256 {
            src[i] = i as u8;
            reference[i] = i as u16;
        }
        let mut dst: Vec<u16> = Vec::with_capacity(src.len());
        dst.resize(src.len(), 0);
        convert_latin1_to_utf16(&src[..], &mut dst[..]);
        assert_eq!(dst, reference);
    }

    #[test]
    fn test_convert_latin1_to_utf8() {
        let mut src: Vec<u8> = Vec::with_capacity(256);
        src.resize(256, 0);
        let mut reference: Vec<u16> = Vec::with_capacity(256);
        reference.resize(256, 0);
        for i in 0..256 {
            src[i] = i as u8;
            reference[i] = i as u16;
        }
        let s = String::from_utf16(&reference[..]).unwrap();
        let mut dst: Vec<u8> = Vec::with_capacity(src.len() * 2);
        dst.resize(src.len() * 2, 0);
        let len = convert_latin1_to_utf8(&src[..], &mut dst[..]);
        dst.truncate(len);
        assert_eq!(&dst[..], s.as_bytes());
    }

    #[test]
    fn test_convert_utf8_to_latin1_lossy() {
        let mut reference: Vec<u8> = Vec::with_capacity(256);
        reference.resize(256, 0);
        let mut src16: Vec<u16> = Vec::with_capacity(256);
        src16.resize(256, 0);
        for i in 0..256 {
            src16[i] = i as u16;
            reference[i] = i as u8;
        }
        let src = String::from_utf16(&src16[..]).unwrap();
        let mut dst: Vec<u8> = Vec::with_capacity(src.len());
        dst.resize(src.len(), 0);
        let len = convert_utf8_to_latin1_lossy(src.as_bytes(), &mut dst[..]);
        dst.truncate(len);
        assert_eq!(dst, reference);
    }

    #[test]
    fn test_convert_utf16_to_latin1_lossy() {
        let mut src: Vec<u16> = Vec::with_capacity(256);
        src.resize(256, 0);
        let mut reference: Vec<u8> = Vec::with_capacity(256);
        reference.resize(256, 0);
        for i in 0..256 {
            src[i] = i as u16;
            reference[i] = i as u8;
        }
        let mut dst: Vec<u8> = Vec::with_capacity(src.len());
        dst.resize(src.len(), 0);
        convert_utf16_to_latin1_lossy(&src[..], &mut dst[..]);
        assert_eq!(dst, reference);
    }

    #[test]
    fn test_utf16_valid_up_to() {
        let valid = vec![0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16,
                         0x2603u16, 0xD83Du16, 0xDCA9u16, 0x00B6u16];
        assert_eq!(utf16_valid_up_to(&valid[..]), 16);;
        let lone_high = vec![0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16,
                             0u16, 0u16, 0x2603u16, 0xD83Du16, 0x00B6u16];
        assert_eq!(utf16_valid_up_to(&lone_high[..]), 14);;
        let lone_low = vec![0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16,
                            0u16, 0u16, 0x2603u16, 0xDCA9u16, 0x00B6u16];
        assert_eq!(utf16_valid_up_to(&lone_low[..]), 14);;
        let lone_high_at_end = vec![0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16,
                                    0u16, 0u16, 0u16, 0x2603u16, 0x00B6u16, 0xD83Du16];
        assert_eq!(utf16_valid_up_to(&lone_high_at_end[..]), 15);;
    }

    #[test]
    fn test_ensure_utf16_validity() {
        let mut src = vec![0u16, 0xD83Du16, 0u16, 0u16, 0u16, 0xD83Du16, 0xDCA9u16, 0u16, 0u16,
                           0u16, 0u16, 0u16, 0u16, 0xDCA9u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16,
                           0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16];
        let reference = vec![0u16, 0xFFFDu16, 0u16, 0u16, 0u16, 0xD83Du16, 0xDCA9u16, 0u16, 0u16,
                             0u16, 0u16, 0u16, 0u16, 0xFFFDu16, 0u16, 0u16, 0u16, 0u16, 0u16,
                             0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16,
                             0u16];
        ensure_utf16_validity(&mut src[..]);
        assert_eq!(src, reference);
    }

}
