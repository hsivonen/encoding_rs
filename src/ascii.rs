// Copyright Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(all(
    feature = "simd-accel",
    any(
        target_feature = "sse2",
        all(target_endian = "little", target_arch = "aarch64"),
        all(target_endian = "little", target_feature = "neon")
    )
))]
use crate::simd_funcs::*;

pub(crate) const MAX_STRIDE_SIZE: usize = 16;

const STRIDE: usize = 16;

macro_rules! ascii_copy {
    ($name:ident, $stride:ident, $src_unit:ty, $dst_unit:ty) => {
        pub fn $name(src: &[$src_unit], dst: &mut [$dst_unit]) -> Option<($src_unit, usize)> {
            let mut consumed = 0usize;
            let (src_strides, _) = src.as_chunks::<STRIDE>();
            let (dst_strides, _) = dst.as_chunks_mut::<STRIDE>();
            for (src_stride, dst_stride) in src_strides.iter().zip(dst_strides.iter_mut()) {
                if let Some((non_ascii, pos)) = $stride(src_stride, dst_stride) {
                    return Some((non_ascii, consumed + pos));
                }
                consumed += STRIDE;
            }
            let src_tail = &src[consumed..];
            let dst_tail = &mut dst[consumed..];
            for (src_slot, dst_slot) in src_tail.iter().zip(dst_tail.iter_mut()) {
                let c = *src_slot;
                if c >= 0x80 {
                    return Some((c, consumed));
                }
                *dst_slot = c as $dst_unit;
                consumed += 1;
            }
            None
        }
    };
}

ascii_copy!(ascii_to_ascii, ascii_to_ascii_stride, u8, u8);
ascii_copy!(ascii_to_basic_latin, ascii_to_basic_latin_stride, u8, u16);
ascii_copy!(basic_latin_to_ascii, basic_latin_to_ascii_stride, u16, u8);

fn ascii_to_ascii_stride(
    src_stride: &[u8; STRIDE],
    dst_stride: &mut [u8; STRIDE],
) -> Option<(u8, usize)> {
    copy_stride(src_stride, dst_stride);
    validate_ascii_stride(src_stride)
}

fn ascii_to_basic_latin_stride(
    src_stride: &[u8; STRIDE],
    dst_stride: &mut [u16; STRIDE],
) -> Option<(u8, usize)> {
    unpack_stride(src_stride, dst_stride);
    validate_ascii_stride(src_stride)
}

fn basic_latin_to_ascii_stride(
    src_stride: &[u16; STRIDE],
    dst_stride: &mut [u8; STRIDE],
) -> Option<(u16, usize)> {
    pack_stride(src_stride, dst_stride);
    validate_basic_latin_stride(src_stride)
}

macro_rules! ascii_validate_stride {
    ($name:ident, $src_unit:ty) => {
        #[inline(always)]
        fn $name(src_stride: &[$src_unit; STRIDE]) -> Option<($src_unit, usize)> {
            if (src_stride[0] < 0x80)
                && (src_stride[1] < 0x80)
                && (src_stride[2] < 0x80)
                && (src_stride[3] < 0x80)
                && (src_stride[4] < 0x80)
                && (src_stride[5] < 0x80)
                && (src_stride[6] < 0x80)
                && (src_stride[7] < 0x80)
                && (src_stride[8] < 0x80)
                && (src_stride[9] < 0x80)
                && (src_stride[10] < 0x80)
                && (src_stride[11] < 0x80)
                && (src_stride[12] < 0x80)
                && (src_stride[13] < 0x80)
                && (src_stride[14] < 0x80)
                && (src_stride[15] < 0x80)
            {
                return None;
            }
            for i in 0..STRIDE {
                let c = src_stride[i];
                if c >= 0x80 {
                    return Some((c, i));
                }
            }
            debug_assert!(false);
            None
        }
    };
}

ascii_validate_stride!(validate_ascii_stride, u8);
ascii_validate_stride!(validate_basic_latin_stride, u16);

fn copy_stride(src_stride: &[u8; STRIDE], dst_stride: &mut [u8; STRIDE]) {
    *dst_stride = *src_stride;
}

fn unpack_stride(src_stride: &[u8; STRIDE], dst_stride: &mut [u16; STRIDE]) {
    dst_stride[0] = src_stride[0] as u16;
    dst_stride[1] = src_stride[1] as u16;
    dst_stride[2] = src_stride[2] as u16;
    dst_stride[3] = src_stride[3] as u16;
    dst_stride[4] = src_stride[4] as u16;
    dst_stride[5] = src_stride[5] as u16;
    dst_stride[6] = src_stride[6] as u16;
    dst_stride[7] = src_stride[7] as u16;
    dst_stride[8] = src_stride[8] as u16;
    dst_stride[9] = src_stride[9] as u16;
    dst_stride[10] = src_stride[10] as u16;
    dst_stride[11] = src_stride[11] as u16;
    dst_stride[12] = src_stride[12] as u16;
    dst_stride[13] = src_stride[13] as u16;
    dst_stride[14] = src_stride[14] as u16;
    dst_stride[15] = src_stride[15] as u16;
}

fn pack_stride(src_stride: &[u16; STRIDE], dst_stride: &mut [u8; STRIDE]) {
    dst_stride[0] = src_stride[0] as u8;
    dst_stride[1] = src_stride[1] as u8;
    dst_stride[2] = src_stride[2] as u8;
    dst_stride[3] = src_stride[3] as u8;
    dst_stride[4] = src_stride[4] as u8;
    dst_stride[5] = src_stride[5] as u8;
    dst_stride[6] = src_stride[6] as u8;
    dst_stride[7] = src_stride[7] as u8;
    dst_stride[8] = src_stride[8] as u8;
    dst_stride[9] = src_stride[9] as u8;
    dst_stride[10] = src_stride[10] as u8;
    dst_stride[11] = src_stride[11] as u8;
    dst_stride[12] = src_stride[12] as u8;
    dst_stride[13] = src_stride[13] as u8;
    dst_stride[14] = src_stride[14] as u8;
    dst_stride[15] = src_stride[15] as u8;
}

/*
pub fn ascii_to_ascii(src: &[u8], dst: &mut [u8]) -> Option<(u8, usize)> {
    let mut copied = 0;
    for (src_slot, dst_slot) in src.iter().zip(dst.iter_mut()) {
        let c = *src_slot;
        if c >= 0x80 {
            return Some((c, copied));
        }
        *dst_slot = c;
        copied += 1;
    }
    None
}

pub fn ascii_to_basic_latin(src: &[u8], dst: &mut [u16]) -> Option<(u8, usize)> {
    let mut copied = 0;
    for (src_slot, dst_slot) in src.iter().zip(dst.iter_mut()) {
        let c = *src_slot;
        if c >= 0x80 {
            return Some((c, copied));
        }
        *dst_slot = u16::from(c);
        copied += 1;
    }
    None
}

pub fn basic_latin_to_ascii(src: &[u16], dst: &mut [u8]) -> Option<(u16, usize)> {
    let mut copied = 0;
    for (src_slot, dst_slot) in src.iter().zip(dst.iter_mut()) {
        let c = *src_slot;
        if c >= 0x80 {
            return Some((c, copied));
        }
        *dst_slot = c as u8;
        copied += 1;
    }
    None
}

pub fn validate_ascii(src: &[u8]) -> Option<(u8, usize)> {
    let mut checked = 0;
    for src_slot in src.iter() {
        let c = *src_slot;
        if c >= 0x80 {
            return Some((c, checked));
        }
        checked += 1;
    }
    None
}

*/

pub fn validate_ascii(bytes: &[u8]) -> Option<(u8, usize)> {
    let mut consumed = 0usize;
    let (strides, _) = bytes.as_chunks::<STRIDE>();
    for stride in strides.iter() {
        if let Some((non_ascii, pos)) = validate_ascii_stride(stride) {
            return Some((non_ascii, consumed + pos));
        }
        consumed += STRIDE;
    }
    let tail = &bytes[consumed..];
    for slot in tail.iter() {
        let c = *slot;
        if c >= 0x80 {
            return Some((c, consumed));
        }
        consumed += 1;
    }
    None
}

pub fn ascii_valid_up_to(bytes: &[u8]) -> usize {
    match validate_ascii(bytes) {
        None => bytes.len(),
        Some((_, num_valid)) => num_valid,
    }
}

pub fn iso_2022_jp_ascii_valid_up_to(bytes: &[u8]) -> usize {
    for (i, b_ref) in bytes.iter().enumerate() {
        let b = *b_ref;
        if b >= 0x80 || b == 0x1B || b == 0x0E || b == 0x0F {
            return i;
        }
    }
    bytes.len()
}

// Any copyright to the test code below this comment is dedicated to the
// Public Domain. http://creativecommons.org/publicdomain/zero/1.0/

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    macro_rules! test_ascii {
        ($test_name:ident, $fn_tested:ident, $src_unit:ty, $dst_unit:ty) => {
            #[test]
            fn $test_name() {
                let mut src: Vec<$src_unit> = Vec::with_capacity(32);
                let mut dst: Vec<$dst_unit> = Vec::with_capacity(32);
                for i in 0..32 {
                    src.clear();
                    dst.clear();
                    dst.resize(32, 0);
                    for j in 0..32 {
                        let c = if i == j { 0xAA } else { j + 0x40 };
                        src.push(c as $src_unit);
                    }
                    match { $fn_tested(&src, &mut dst) } {
                        None => unreachable!("Should always find non-ASCII"),
                        Some((non_ascii, num_ascii)) => {
                            assert_eq!(non_ascii, 0xAA);
                            assert_eq!(num_ascii, i);
                            for j in 0..i {
                                assert_eq!(dst[j], (j + 0x40) as $dst_unit);
                            }
                        }
                    }
                }
            }
        };
    }

    test_ascii!(test_ascii_to_ascii, ascii_to_ascii, u8, u8);
    test_ascii!(test_ascii_to_basic_latin, ascii_to_basic_latin, u8, u16);
    test_ascii!(test_basic_latin_to_ascii, basic_latin_to_ascii, u16, u8);
}
