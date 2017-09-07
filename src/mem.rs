// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ascii::*;
use super::in_inclusive_range8;
use super::DecoderResult;
use super::EncoderResult;
use utf_8::Utf8Decoder;
use utf_8::Utf8Encoder;

const SIMD_ALIGNMENT: usize = 16;

const SIMD_ALIGNMENT_MASK: usize = 15;

const LINE_ALIGNMENT: usize = 64;

const LINE_ALIGNMENT_MASK: usize = 63;

pub fn is_ascii(buffer: &[u8]) -> bool {
    let src = buffer.as_ptr();
    let len = buffer.len();
    if len == 0 {
        return true;
    }
    let mut offset = 0usize;
    let mut until_simd_alignment = (SIMD_ALIGNMENT - ((src as usize) & SIMD_ALIGNMENT_MASK)) & SIMD_ALIGNMENT_MASK;
    let mut alu_accu = 0usize;
    if until_simd_alignment + SIMD_ALIGNMENT <= len {
        while until_simd_alignment != 0 {
            alu_accu |= buffer[offset] as usize;
            offset += 1;
            until_simd_alignment -= 1;
        }
        if alu_accu >= 0x80 {
            return false;
        }
        let mut simd_accu = 0;


    }
    while offset < len {
        alu_accu |= buffer[offset] as usize;
    }
    alu_accu < 0x80
}

pub fn is_basic_latin(buffer: &[u16]) -> bool {
    true
}

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

pub fn is_utf16_latin1(buffer: &[u16]) -> bool {
    true
}

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

pub fn convert_str_to_utf16(src: &str, dst: &mut [u16]) -> usize {
    assert!(dst.len() >= src.len(), "Destination must not be shorter than the source.");
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

pub fn convert_utf16_to_utf8(src: &[u16], dst: &mut [u8]) -> usize {
    assert!(dst.len() >= src.len() * 3 + 1);
    let mut encoder = Utf8Encoder;
    let (result, _, written) = encoder.encode_from_utf16_raw(src, dst, true);
    debug_assert!(result == EncoderResult::InputEmpty);
    written
}

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

pub fn convert_latin1_to_utf16(src: &[u8], dst: &mut [u16]) {
    assert!(dst.len() >= src.len(), "Destination must not be shorter than the source.");
    unsafe {
        unpack_latin1(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
}

pub fn convert_latin1_to_utf8(src: &[u8], dst: &mut [u8]) -> usize {
    assert!(dst.len() >= src.len() * 2, "Destination must not be shorter than the source times two.");
    let dst_len = dst.len();
    let src_ptr = src.as_ptr();
    let dst_ptr = dst.as_mut_ptr();
    let mut total_read = 0usize;
    let mut total_written = 0usize;
    loop {
        // src can't advance more than dst
        let dst_left = dst_len - total_read;
        if let Some((non_ascii, consumed)) =
            unsafe {
                ascii_to_ascii(
                    src_ptr.offset(total_read as isize),
                    dst_ptr.offset(total_written as isize),
                    dst_left,
                )
            } {
            total_read += consumed + 1;
            total_written += consumed;

            let code_point = non_ascii as u32;
            dst[total_written] = ((code_point >> 6) | 0xC0u32) as u8;
            total_written += 1;
            if total_written == dst_len {
                return total_written;
            }
            dst[total_written] = ((code_point as u32 & 0x3Fu32) | 0x80u32) as u8;
            total_written += 1;
            continue;
        }
        return total_written + dst_left;
    }
}

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

pub fn convert_utf8_to_latin1_lossy(src: &[u8], dst: &mut [u8]) -> usize {
    assert!(dst.len() >= src.len(), "Destination must not be shorter than the source.");
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

            dst[total_written] = (((non_ascii as u32 & 0x1Fu32) << 6) | (trail as u32 & 0x3Fu32)) as u8;
            total_written += 1;
            continue;
        }
        return total_written + src_left;
    }
}

pub fn convert_utf16_to_latin1_lossy(src: &[u16], dst: &mut [u8]) {
    assert!(dst.len() >= src.len(), "Destination must not be shorter than the source.");
    unsafe {
        pack_latin1(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
}

fn utf16_valid_up_to_alu(buffer: &[u16]) -> usize {
    0
}

pub fn utf16_valid_up_to(buffer: &[u16]) -> usize {
    0
}

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

pub fn copy_ascii_to_ascii(src: &[u8], dst: &mut [u8]) -> usize {
    assert!(dst.len() >= src.len(), "Destination must not be shorter than the source.");
    if let Some((_, consumed)) =
            unsafe {
                ascii_to_ascii(
                    src.as_ptr(),
                    dst.as_mut_ptr(),
                    src.len(),
                )
            } {
        consumed
    } else {
        src.len()
    }
}

pub fn copy_ascii_to_basic_latin(src: &[u8], dst: &mut [u16]) -> usize {
    assert!(dst.len() >= src.len(), "Destination must not be shorter than the source.");
    if let Some((_, consumed)) =
            unsafe {
                ascii_to_basic_latin(
                    src.as_ptr(),
                    dst.as_mut_ptr(),
                    src.len(),
                )
            } {
        consumed
    } else {
        src.len()
    }
}

pub fn copy_basic_latin_to_ascii(src: &[u16], dst: &mut [u8]) -> usize {
    assert!(dst.len() >= src.len(), "Destination must not be shorter than the source.");
    if let Some((_, consumed)) =
            unsafe {
                basic_latin_to_ascii(
                    src.as_ptr(),
                    dst.as_mut_ptr(),
                    src.len(),
                )
            } {
        consumed
    } else {
        src.len()
    }
}
