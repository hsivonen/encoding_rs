// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ascii::*;
use ascii;
use super::in_inclusive_range8;
use super::DecoderResult;
use super::EncoderResult;
use utf_8::Utf8Decoder;
use utf_8::Utf8Encoder;

const SIMD_ALIGNMENT: usize = 16;

const SIMD_ALIGNMENT_MASK: usize = 15;

const ALU_ALIGNMENT: usize = 8;

const ALU_ALIGNMENT_MASK: usize = 7;

const ALU_STRIDE_SIZE: usize = 8;

// `as` truncates, so works on 32-bit, too.
const LATIN1_MASK: usize = 0xFF00FF00_FF00FF00u64 as usize;

#[inline(always)]
fn is_ascii_alu(buffer: &[u8]) -> bool {
    let src = buffer.as_ptr();
    let len = buffer.len();
    let mut offset = 0usize;
    let mut until_alignment = (ALU_ALIGNMENT - ((src as usize) & ALU_ALIGNMENT_MASK)) & ALU_ALIGNMENT_MASK;
    let mut accu = 0usize;
    if until_alignment + ALU_STRIDE_SIZE <= len {
        while until_alignment != 0 {
            accu |= buffer[offset] as usize;
            offset += 1;
            until_alignment -= 1;
        }
        let len_minus_stride = len - ALU_STRIDE_SIZE;
        loop {
            accu |= unsafe { *(src.offset(offset as isize) as *const usize) };
            offset += ALU_STRIDE_SIZE;
            if offset > len_minus_stride {
                break;
            }
        }
    }
    while offset < len {
        accu |= buffer[offset] as usize;
        offset += 1;
    }
    accu & ascii::ASCII_MASK == 0
}

#[inline(always)]
fn is_basic_latin_alu(buffer: &[u16]) -> bool {
    let src = buffer.as_ptr();
    let len = buffer.len();
    let mut offset = 0usize;
    let mut until_alignment = ((ALU_ALIGNMENT - ((src as usize) & ALU_ALIGNMENT_MASK)) & ALU_ALIGNMENT_MASK) / 2;
    let mut accu = 0usize;
    if until_alignment + ALU_STRIDE_SIZE / 2 <= len {
        while until_alignment != 0 {
            accu |= buffer[offset] as usize;
            offset += 1;
            until_alignment -= 1;
        }
        let len_minus_stride = len - ALU_STRIDE_SIZE / 2;
        loop {
            accu |= unsafe { *(src.offset(offset as isize) as *const usize) };
            offset += ALU_STRIDE_SIZE / 2;
            if offset > len_minus_stride {
                break;
            }
        }
    }
    while offset < len {
        accu |= buffer[offset] as usize;
        offset += 1;
    }
    accu & ascii::BASIC_LATIN_MASK == 0
}

#[inline(always)]
fn is_utf16_latin1_alu(buffer: &[u16]) -> bool {
    let src = buffer.as_ptr();
    let len = buffer.len();
    let mut offset = 0usize;
    let mut until_alignment = ((ALU_ALIGNMENT - ((src as usize) & ALU_ALIGNMENT_MASK)) & ALU_ALIGNMENT_MASK) / 2;
    let mut accu = 0usize;
    if until_alignment + ALU_STRIDE_SIZE / 2 <= len {
        while until_alignment != 0 {
            accu |= buffer[offset] as usize;
            offset += 1;
            until_alignment -= 1;
        }
        let len_minus_stride = len - ALU_STRIDE_SIZE / 2;
        loop {
            accu |= unsafe { *(src.offset(offset as isize) as *const usize) };
            offset += ALU_STRIDE_SIZE / 2;
            if offset > len_minus_stride {
                break;
            }
        }
    }
    while offset < len {
        accu |= buffer[offset] as usize;
        offset += 1;
    }
    accu & LATIN1_MASK == 0
}

#[inline(always)]
fn utf16_valid_up_to_alu(buffer: &[u16]) -> usize {
    let len = buffer.len();
    let mut offset = 0usize;
    while offset < len {
        let unit = buffer[offset];
        let next = offset + 1;
        let unit_minus_surrogate_start = unit.wrapping_sub(0xD800);
        if unit_minus_surrogate_start > (0xDFFF - 0xD800) {
            // Not a surrogate
            offset = next;
            continue;
        }
        if unit_minus_surrogate_start <= (0xDFFF - 0xDBFF) {
            // high surrogate
            if next < len {
                let second = buffer[next];
                let second_minus_low_surrogate_start = second.wrapping_sub(0xDC00);
                if second_minus_low_surrogate_start <= (0xDFFF - 0xDC00) {
                    // The next code unit is a low surrogate. Advance position.
                    offset = next + 1;
                    continue;
                }
                // The next code unit is not a low surrogate. Don't advance
                // position and treat the high surrogate as unpaired.
                // fall through
            }
            // Unpaired surrogate
            return offset;
        }
    }
    len
}

pub fn is_ascii(buffer: &[u8]) -> bool {
    is_ascii_alu(buffer)
}

pub fn is_basic_latin(buffer: &[u16]) -> bool {
    is_basic_latin_alu(buffer)
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
    is_utf16_latin1_alu(buffer)
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

pub fn utf16_valid_up_to(buffer: &[u16]) -> usize {
    utf16_valid_up_to_alu(buffer)
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
}

