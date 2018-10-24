// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "parallel-utf8")]
extern crate rayon;

use super::*;
use ascii::ascii_to_basic_latin;
use ascii::basic_latin_to_ascii;
use handles::*;
use variant::*;

cfg_if!{
    if #[cfg(feature = "simd-accel")] {
        use ::std::intrinsics::unlikely;
        use ::std::intrinsics::likely;
    } else {
        #[inline(always)]
        // Unsafe to match the intrinsic, which is needlessly unsafe.
        unsafe fn unlikely(b: bool) -> bool {
            b
        }
        #[inline(always)]
        // Unsafe to match the intrinsic, which is needlessly unsafe.
        unsafe fn likely(b: bool) -> bool {
            b
        }
    }
}

// Keep this cfg_if in sync with whether the utf_8_core module is defined in lib.rs.
cfg_if! {
    // When running 32-bit ARM code on Raspberry Pi 3, which has a 64-bit CPU,
    // this is a pessimization for non-Latin, non-CJK scripts. However, this
    // optimization seems to work across scripts when running 32-bit ARM code
    // on a 32-bit ARM CPU (particularly good on Exynos 5) and when running
    // 64-bit ARM code on a 64-bit ARM CPU.
    if #[cfg(any(all(feature = "simd-accel", target_feature = "sse2"), all(target_endian = "little", target_arch = "aarch64"), all(target_endian = "little", target_arch = "arm")))] {
        use utf_8_core::run_utf8_validation;
    } else {
        use ::std::str::Utf8Error;
        #[inline(always)]
        fn run_utf8_validation(v: &[u8]) -> Result<&str, Utf8Error> {
            ::std::str::from_utf8(v)
        }
    }
}

/// Format for the value in the first table
///
/// Bit 0: 1 iff lead is C2..DF (two-byte sequence)
/// Bit 1: 1 iff lead is E0 (three-byte with special lower bound for second)
/// Bit 2: 1 iff lead is E1..EC (first normal three-byte range)
/// Bit 3: 1 iff lead is ED (three-byte with special upper bound for second)
/// Bit 4: 1 iff lead is EE..EF (second normal three-byte range)
/// Bit 5: 1 iff lead is F0 (four-byte with special lower bound for second)
/// Bit 6: 1 iff lead is F1..F3 (normal four-byte sequence)
/// Bit 7: 1 iff lead is F4 (four-byte with special upper bound for second)
///
/// Bit 8: 1 iff none of the above is 1
///
/// Bit 9: 1 iff two-byte or four-byte
/// Bit 10: 1 iff three-byte or four-byte
///
/// Shifting right by 9, ANDing by 3 and adding 1 gives the advance.
///
/// Bit 11: Always 1 if valid lead
/// Bit 12: Always 1 if valid lead
/// Bit 13: Always 1 if valid lead
/// Bit 14: 1 iff three-byte or two-byte
/// Bit 15: 1 iff two-byte
///
/// Shifting right by 11 gives the mask for extracting the payload
/// bits from the lead byte.
///
/// Format for the other tables:
/// Bit is 1 iff the given the lead corresponding to the bit position
/// this trail byte is invalid.
#[repr(align(64))] // Align to cache lines
pub struct Utf8Data {
    first: [u16; 128],
    second: [u8; 256],
    third: [u8; 256],
    fourth: [u8; 256],
}

pub static UTF_8_DATA: Utf8Data = Utf8Data {
    // BEGIN GENERATED CODE. PLEASE DO NOT EDIT.
    // Instead, please regenerate using generate-encoding-data.py
    first: [
        0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800,
        0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800,
        0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800,
        0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800,
        0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800,
        0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800,
        0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01,
        0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01,
        0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0xFA01, 0x7C02, 0x7C04, 0x7C04,
        0x7C04, 0x7C04, 0x7C04, 0x7C04, 0x7C04, 0x7C04, 0x7C04, 0x7C04, 0x7C04, 0x7C04, 0x7C08,
        0x7C10, 0x7C10, 0x3E20, 0x3E40, 0x3E40, 0x3E40, 0x3E40, 0x3800, 0x3800, 0x3800, 0x3800,
        0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800, 0x3800,
    ],
    second: [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
        0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x82, 0x82, 0x82, 0x82, 0x82, 0x82,
        0x82, 0x82, 0x82, 0x82, 0x82, 0x82, 0x82, 0x82, 0x82, 0x82, 0x88, 0x88, 0x88, 0x88, 0x88,
        0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88,
        0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF,
    ],
    third: [
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0xFE, 0xFE,
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
        0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
        0xFE,
    ],
    fourth: [
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0,
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0,
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0,
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0,
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0,
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0,
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0,
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0,
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0xE0, 0xE0,
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0,
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0,
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0,
        0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0, 0xE0,
        0xE0,
    ],
    // END GENERATED CODE
};

pub const UTF8_NORMAL_TRAIL: u8 = 1 << 3;

pub const UTF8_THREE_BYTE_SPECIAL_LOWER_BOUND_TRAIL: u8 = 1 << 4;

pub const UTF8_THREE_BYTE_SPECIAL_UPPER_BOUND_TRAIL: u8 = 1 << 5;

pub const UTF8_FOUR_BYTE_SPECIAL_LOWER_BOUND_TRAIL: u8 = 1 << 6;

pub const UTF8_FOUR_BYTE_SPECIAL_UPPER_BOUND_TRAIL: u8 = 1 << 7;

/// Bit is 1 if the trail is invalid.
pub static UTF8_TRAIL_INVALID: [u8; 256] = [
    248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248,
    248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248,
    248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248,
    248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248,
    248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248,
    248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248,
    248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 80, 80, 80, 80, 80, 80,
    80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 144, 144, 144, 144, 144, 144, 144, 144, 144, 144, 144,
    144, 144, 144, 144, 144, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 160, 248,
    248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248,
    248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248,
    248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248, 248,
    248, 248, 248, 248, 248, 248,
];

#[cfg(feature = "parallel-utf8")]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::never_loop))]
pub fn utf8_valid_up_to(bytes: &[u8]) -> usize {
    let mut len = bytes.len();
    // The purpose of the outer loop is to avoid recursion when the attempt
    // to find the split point discovers and over-long sequence.
    'outer: loop {
        // This magic number has been determined on i7-4770 with SSE2 enabled.
        // It's very likely that the number should be different when different
        // ISA is used for ASCII acceleration. The number has been chosen
        // to optimize the all-ASCII case. With mostly non-ASCII, the number
        // should be much smaller, but that would pessimize the all-ASCII case,
        // which we are trying to optimize here.
        if len < 290000 {
            return match run_utf8_validation(&bytes[..len]) {
                Ok(_) => bytes.len(),
                Err(e) => e.valid_up_to(),
            };
        }
        let mid = len >> 1;
        let mut adjusted = mid;
        let mut i = 0;
        'inner: loop {
            // No need to check for `adjusted` reaching `len` because we
            // already know that `len` is way larger than `(len / 2) + 4`.
            if i == 3 {
                // `mid` landed inside an overlong sequence.
                len = mid;
                continue 'outer;
            }
            if (bytes[adjusted] & 0xC0) != 0x80 {
                break 'inner;
            }
            adjusted += 1;
            i += 1;
        }
        let (head, tail) = bytes[..len].split_at(adjusted);
        let (head_valid_up_to, tail_valid_up_to) =
            rayon::join(|| utf8_valid_up_to(head), || utf8_valid_up_to(tail));
        if head_valid_up_to == adjusted {
            return adjusted + tail_valid_up_to;
        }
        return head_valid_up_to;
    }
}

#[cfg(not(feature = "parallel-utf8"))]
pub fn utf8_valid_up_to(bytes: &[u8]) -> usize {
    match run_utf8_validation(bytes) {
        Ok(_) => bytes.len(),
        Err(e) => e.valid_up_to(),
    }
}

#[cfg_attr(
    feature = "cargo-clippy",
    allow(clippy::never_loop, clippy::cyclomatic_complexity)
)]
pub fn convert_utf8_to_utf16_up_to_invalid(src: &[u8], dst: &mut [u16]) -> (usize, usize) {
    // This algorithm differs from the UTF-8 validation algorithm, but making
    // this one consistent with that one makes this slower for reasons I don't
    // understand.
    let mut read = 0;
    let mut written = 0;
    'outer: loop {
        {
            let src_remaining = &src[read..];
            let dst_remaining = &mut dst[written..];
            let length = ::std::cmp::min(src_remaining.len(), dst_remaining.len());
            match unsafe {
                ascii_to_basic_latin(src_remaining.as_ptr(), dst_remaining.as_mut_ptr(), length)
            } {
                None => {
                    read += length;
                    written += length;
                    break 'outer;
                }
                Some((_, consumed)) => {
                    read += consumed;
                    written += consumed;
                }
            }
        }
        // Longest sequence times two
        if unsafe { likely(read + 8 <= src.len()) } {
            // These fuse into a single unaligned 32-bit read
            let mut first = unsafe { *(src.get_unchecked(read)) };
            let mut second = unsafe { *(src.get_unchecked(read + 1)) };
            let mut third = unsafe { *(src.get_unchecked(read + 2)) };
            let mut fourth = unsafe { *(src.get_unchecked(read + 3)) };
            'mid: loop {
                debug_assert!(first >= 0x80);
                let first_info =
                    unsafe { *(UTF_8_DATA.first.get_unchecked(first as usize - 0x80)) };
                let second_info = UTF_8_DATA.second[second as usize];

                // TODO: Compute the following two without table lookups
                // let third_info = UTF_8_DATA.third[third as usize];
                // let fourth_info = UTF_8_DATA.fourth[fourth as usize];

                // These compile to cmovl instructions that are slower than
                // memory accesses.
                let third_info = if in_inclusive_range8(third, 0x80, 0xBF) {
                    0u8
                } else {
                    0xFE
                };
                let fourth_info = if in_inclusive_range8(fourth, 0x80, 0xBF) {
                    0u8
                } else {
                    0xE0
                };

                let trail_status = 0x100u16
                    | u16::from(second_info)
                    | u16::from(third_info)
                    | u16::from(fourth_info);
                let validity = trail_status & first_info;

                // Move the validity check to later to make it obvious how long
                // this basic block is.

                let advance_minus_one = (first_info >> 9) & 3;
                let advance = usize::from(advance_minus_one) + 1;
                let lead_mask = first_info >> 11;

                let mut code_point = u32::from(first) & u32::from(lead_mask);
                code_point <<= 6;
                code_point |= u32::from(second & 0x3F);

                // Branchlessly compute a mask that has enough bits set to cover 6
                // or 0x3F if three-byte or four-byte and bits zeroed if two-byte
                // to make the handling of the third byte a branchless no-op in the
                // two-byte case. Bit 15 is 1 iff two-byte, so sign-extend it across
                // 16 bits and negate.
                let mask = !((first_info as i16) >> 15) as u8;
                code_point <<= 6 & mask;
                code_point |= u32::from(third & 0x3F & mask);

                if unsafe { unlikely(validity != 0) } {
                    break 'outer;
                }
                if unsafe { likely(advance != 4) } {
                    unsafe { *(dst.get_unchecked_mut(written)) = code_point as u16 };
                    written += 1;
                } else if unsafe { likely(written + 1 <= dst.len()) } {
                    code_point <<= 6;
                    code_point |= u32::from(fourth & 0x3F);

                    unsafe {
                        *(dst.get_unchecked_mut(written)) = (0xD7C0 + (code_point >> 10)) as u16
                    };
                    unsafe {
                        *(dst.get_unchecked_mut(written + 1)) =
                            (0xDC00 + (code_point & 0x3FF)) as u16
                    };
                    written += 2;
                } else {
                    break 'outer;
                }
                read += advance;

                if unsafe { unlikely(read + 8 > src.len()) } {
                    break 'mid;
                }
                // >= optimizes better than ==
                if unsafe { unlikely(written >= dst.len()) } {
                    debug_assert_eq!(written, dst.len());
                    break 'outer;
                }

                // These fuse into a single unaligned 32-bit read
                first = unsafe { *(src.get_unchecked(read)) };
                second = unsafe { *(src.get_unchecked(read + 1)) };
                third = unsafe { *(src.get_unchecked(read + 2)) };
                fourth = unsafe { *(src.get_unchecked(read + 3)) };
                if unsafe { likely(first >= 0x80) } {
                    continue 'mid;
                }
                unsafe { *(dst.get_unchecked_mut(written)) = u16::from(first) };
                read += 1;
                written += 1;
                continue 'outer;
            }
        }
        // tail
        'tail: loop {
            if read >= src.len() {
                debug_assert_eq!(read, src.len());
                break 'outer;
            }
            let first = src[read];
            if first < 0x80 {
                if written >= dst.len() {
                    debug_assert_eq!(written, dst.len());
                    break 'outer;
                }
                dst[written] = u16::from(first);
                read += 1;
                written += 1;
                continue 'tail;
            }

            if read + 1 >= src.len() {
                debug_assert_eq!(read + 1, src.len());
                break 'outer;
            }
            if written >= dst.len() {
                debug_assert_eq!(written, dst.len());
                break 'outer;
            }

            let second = src[read + 1];

            debug_assert!(first >= 0x80);
            let first_info = unsafe { *(UTF_8_DATA.first.get_unchecked(first as usize - 0x80)) };
            let second_info = UTF_8_DATA.second[second as usize];

            let advance_minus_one = (first_info >> 9) & 3;
            let advance = usize::from(advance_minus_one) + 1;
            let lead_mask = first_info >> 11;

            let mut code_point = u32::from(first) & u32::from(lead_mask);
            code_point <<= 6;
            code_point |= u32::from(second & 0x3F);

            let mut trail_status = 0x100u16 | u16::from(second_info);
            if advance == 2 {
                let validity = trail_status & first_info;
                if validity != 0 {
                    break 'outer;
                }
                dst[written] = code_point as u16;
                written += 1;
                read += advance;
                continue 'tail;
            }
            if read + 2 >= src.len() {
                debug_assert_eq!(read + 2, src.len());
                break 'outer;
            }
            let third = src[read + 2];
            let third_info = UTF_8_DATA.third[third as usize];
            trail_status |= u16::from(third_info);
            code_point <<= 6;
            code_point |= u32::from(third & 0x3F);
            if advance == 3 {
                let validity = trail_status & first_info;
                if validity != 0 {
                    break 'outer;
                }
                dst[written] = code_point as u16;
                written += 1;
                read += advance;
                continue 'tail;
            }
            debug_assert_eq!(advance, 4);
            if read + 3 >= src.len() {
                debug_assert_eq!(read + 3, src.len());
                break 'outer;
            }
            if written + 1 >= dst.len() {
                debug_assert_eq!(written + 1, dst.len());
                break 'outer;
            }
            let fourth = src[read + 3];
            let fourth_info = UTF_8_DATA.fourth[fourth as usize];
            trail_status |= u16::from(fourth_info);
            code_point <<= 6;
            code_point |= u32::from(fourth & 0x3F);
            let validity = trail_status & first_info;
            if validity != 0 {
                break 'outer;
            }
            dst[written] = (0xD7C0 + (code_point >> 10)) as u16;
            dst[written + 1] = (0xDC00 + (code_point & 0x3FF)) as u16;
            written += 2;
            read += advance;
            continue 'tail;
        }
        // Unreachable
    }
    (read, written)
}

pub struct Utf8Decoder {
    code_point: u32,
    bytes_seen: usize,   // 1, 2 or 3: counts continuations only
    bytes_needed: usize, // 1, 2 or 3: counts continuations only
    lower_boundary: u8,
    upper_boundary: u8,
}

impl Utf8Decoder {
    pub fn new_inner() -> Utf8Decoder {
        Utf8Decoder {
            code_point: 0,
            bytes_seen: 0,
            bytes_needed: 0,
            lower_boundary: 0x80u8,
            upper_boundary: 0xBFu8,
        }
    }

    pub fn new() -> VariantDecoder {
        VariantDecoder::Utf8(Utf8Decoder::new_inner())
    }

    fn extra_from_state(&self) -> usize {
        if self.bytes_needed == 0 {
            0
        } else {
            self.bytes_seen + 1
        }
    }

    pub fn max_utf16_buffer_length(&self, byte_length: usize) -> Option<usize> {
        byte_length.checked_add(1 + self.extra_from_state())
    }

    pub fn max_utf8_buffer_length_without_replacement(&self, byte_length: usize) -> Option<usize> {
        byte_length.checked_add(3 + self.extra_from_state())
    }

    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> Option<usize> {
        checked_add(
            3,
            checked_mul(3, byte_length.checked_add(self.extra_from_state())),
        )
    }

    decoder_functions!(
        {},
        {
            // This is the fast path. The rest runs only at the
            // start and end for partial sequences.
            if self.bytes_needed == 0 {
                dest.copy_utf8_up_to_invalid_from(&mut source);
            }
        },
        {
            if self.bytes_needed != 0 {
                let bad_bytes = (self.bytes_seen + 1) as u8;
                self.code_point = 0;
                self.bytes_needed = 0;
                self.bytes_seen = 0;
                return (
                    DecoderResult::Malformed(bad_bytes, 0),
                    src_consumed,
                    dest.written(),
                );
            }
        },
        {
            if self.bytes_needed == 0 {
                if b < 0x80u8 {
                    destination_handle.write_ascii(b);
                    continue;
                }
                if b < 0xC2u8 {
                    return (
                        DecoderResult::Malformed(1, 0),
                        unread_handle.consumed(),
                        destination_handle.written(),
                    );
                }
                if b < 0xE0u8 {
                    self.bytes_needed = 1;
                    self.code_point = u32::from(b) & 0x1F;
                    continue;
                }
                if b < 0xF0u8 {
                    if b == 0xE0u8 {
                        self.lower_boundary = 0xA0u8;
                    } else if b == 0xEDu8 {
                        self.upper_boundary = 0x9Fu8;
                    }
                    self.bytes_needed = 2;
                    self.code_point = u32::from(b) & 0xF;
                    continue;
                }
                if b < 0xF5u8 {
                    if b == 0xF0u8 {
                        self.lower_boundary = 0x90u8;
                    } else if b == 0xF4u8 {
                        self.upper_boundary = 0x8Fu8;
                    }
                    self.bytes_needed = 3;
                    self.code_point = u32::from(b) & 0x7;
                    continue;
                }
                return (
                    DecoderResult::Malformed(1, 0),
                    unread_handle.consumed(),
                    destination_handle.written(),
                );
            }
            // self.bytes_needed != 0
            if !(b >= self.lower_boundary && b <= self.upper_boundary) {
                let bad_bytes = (self.bytes_seen + 1) as u8;
                self.code_point = 0;
                self.bytes_needed = 0;
                self.bytes_seen = 0;
                self.lower_boundary = 0x80u8;
                self.upper_boundary = 0xBFu8;
                return (
                    DecoderResult::Malformed(bad_bytes, 0),
                    unread_handle.unread(),
                    destination_handle.written(),
                );
            }
            self.lower_boundary = 0x80u8;
            self.upper_boundary = 0xBFu8;
            self.code_point = (self.code_point << 6) | (u32::from(b) & 0x3F);
            self.bytes_seen += 1;
            if self.bytes_seen != self.bytes_needed {
                continue;
            }
            if self.bytes_needed == 3 {
                destination_handle.write_astral(self.code_point);
            } else {
                destination_handle.write_bmp_excl_ascii(self.code_point as u16);
            }
            self.code_point = 0;
            self.bytes_needed = 0;
            self.bytes_seen = 0;
            continue;
        },
        self,
        src_consumed,
        dest,
        source,
        b,
        destination_handle,
        unread_handle,
        check_space_astral
    );
}

pub struct Utf8Encoder;

impl Utf8Encoder {
    pub fn new(encoding: &'static Encoding) -> Encoder {
        Encoder::new(encoding, VariantEncoder::Utf8(Utf8Encoder))
    }

    pub fn max_buffer_length_from_utf16_without_replacement(
        &self,
        u16_length: usize,
    ) -> Option<usize> {
        checked_add(1, u16_length.checked_mul(3))
    }

    pub fn max_buffer_length_from_utf8_without_replacement(
        &self,
        byte_length: usize,
    ) -> Option<usize> {
        Some(byte_length)
    }

    #[cfg_attr(feature = "cargo-clippy", allow(clippy::never_loop))]
    pub fn encode_from_utf16_raw(
        &mut self,
        src: &[u16],
        dst: &mut [u8],
        _last: bool,
    ) -> (EncoderResult, usize, usize) {
        let mut read = 0;
        let mut written = 0;
        'outer: loop {
            let mut unit = {
                let src_remaining = &src[read..];
                let dst_remaining = &mut dst[written..];
                let (pending, length) = if dst_remaining.len() < src_remaining.len() {
                    (EncoderResult::OutputFull, dst_remaining.len())
                } else {
                    (EncoderResult::InputEmpty, src_remaining.len())
                };
                match unsafe {
                    basic_latin_to_ascii(src_remaining.as_ptr(), dst_remaining.as_mut_ptr(), length)
                } {
                    None => {
                        read += length;
                        written += length;
                        return (pending, read, written);
                    }
                    Some((non_ascii, consumed)) => {
                        read += consumed;
                        written += consumed;
                        non_ascii
                    }
                }
            };
            'inner: loop {
                // The following loop is only broken out of as a goto forward.
                loop {
                    // Unfortunately, this check isn't enough for the compiler to elide
                    // the bound checks on writes to dst, which is why they are manually
                    // elided, which makes a measurable difference.
                    if written.checked_add(4).unwrap() > dst.len() {
                        return (EncoderResult::OutputFull, read, written);
                    }
                    read += 1;
                    if unit < 0x800 {
                        unsafe {
                            *(dst.get_unchecked_mut(written)) = (unit >> 6) as u8 | 0xC0u8;
                            written += 1;
                            *(dst.get_unchecked_mut(written)) = (unit & 0x3F) as u8 | 0x80u8;
                            written += 1;
                        }
                        break;
                    }
                    let unit_minus_surrogate_start = unit.wrapping_sub(0xD800);
                    if unsafe { likely(unit_minus_surrogate_start > (0xDFFF - 0xD800)) } {
                        unsafe {
                            *(dst.get_unchecked_mut(written)) = (unit >> 12) as u8 | 0xE0u8;
                            written += 1;
                            *(dst.get_unchecked_mut(written)) =
                                ((unit & 0xFC0) >> 6) as u8 | 0x80u8;
                            written += 1;
                            *(dst.get_unchecked_mut(written)) = (unit & 0x3F) as u8 | 0x80u8;
                            written += 1;
                        }
                        break;
                    }
                    if unsafe { likely(unit_minus_surrogate_start <= (0xDBFF - 0xD800)) } {
                        // high surrogate
                        // read > src.len() is impossible, but using
                        // >= instead of == allows the compiler to elide a bound check.
                        if read >= src.len() {
                            debug_assert_eq!(read, src.len());
                            // Unpaired surrogate at the end of the buffer.
                            unsafe {
                                *(dst.get_unchecked_mut(written)) = 0xEFu8;
                                written += 1;
                                *(dst.get_unchecked_mut(written)) = 0xBFu8;
                                written += 1;
                                *(dst.get_unchecked_mut(written)) = 0xBDu8;
                                written += 1;
                            }
                            return (EncoderResult::InputEmpty, read, written);
                        }
                        let second = src[read];
                        let second_minus_low_surrogate_start = second.wrapping_sub(0xDC00);
                        if unsafe { likely(second_minus_low_surrogate_start <= (0xDFFF - 0xDC00)) }
                        {
                            // The next code unit is a low surrogate. Advance position.
                            read += 1;
                            let astral = (u32::from(unit) << 10) + u32::from(second)
                                - (((0xD800u32 << 10) - 0x10000u32) + 0xDC00u32);
                            unsafe {
                                *(dst.get_unchecked_mut(written)) = (astral >> 18) as u8 | 0xF0u8;
                                written += 1;
                                *(dst.get_unchecked_mut(written)) =
                                    ((astral & 0x3F000u32) >> 12) as u8 | 0x80u8;
                                written += 1;
                                *(dst.get_unchecked_mut(written)) =
                                    ((astral & 0xFC0u32) >> 6) as u8 | 0x80u8;
                                written += 1;
                                *(dst.get_unchecked_mut(written)) = (astral & 0x3F) as u8 | 0x80u8;
                                written += 1;
                            }
                            break;
                        }
                        // The next code unit is not a low surrogate. Don't advance
                        // position and treat the high surrogate as unpaired.
                        // Fall through
                    }
                    // Unpaired low surrogate
                    unsafe {
                        *(dst.get_unchecked_mut(written)) = 0xEFu8;
                        written += 1;
                        *(dst.get_unchecked_mut(written)) = 0xBFu8;
                        written += 1;
                        *(dst.get_unchecked_mut(written)) = 0xBDu8;
                        written += 1;
                    }
                    break;
                }
                // Now see if the next unit is Basic Latin
                // read > src.len() is impossible, but using
                // >= instead of == allows the compiler to elide a bound check.
                if read >= src.len() {
                    debug_assert_eq!(read, src.len());
                    return (EncoderResult::InputEmpty, read, written);
                }
                unit = src[read];
                if unsafe { unlikely(unit < 0x80) } {
                    // written > dst.len() is impossible, but using
                    // >= instead of == allows the compiler to elide a bound check.
                    if written >= dst.len() {
                        debug_assert_eq!(written, dst.len());
                        return (EncoderResult::OutputFull, read, written);
                    }
                    dst[written] = unit as u8;
                    read += 1;
                    written += 1;
                    // Mysteriously, adding a punctuation check here makes
                    // the expected benificiary cases *slower*!
                    continue 'outer;
                }
                continue 'inner;
            }
        }
    }

    pub fn encode_from_utf8_raw(
        &mut self,
        src: &str,
        dst: &mut [u8],
        _last: bool,
    ) -> (EncoderResult, usize, usize) {
        let bytes = src.as_bytes();
        let mut to_write = bytes.len();
        if to_write <= dst.len() {
            (&mut dst[..to_write]).copy_from_slice(bytes);
            return (EncoderResult::InputEmpty, to_write, to_write);
        }
        to_write = dst.len();
        // Move back until we find a UTF-8 sequence boundary.
        while (bytes[to_write] & 0xC0) == 0x80 {
            to_write -= 1;
        }
        (&mut dst[..to_write]).copy_from_slice(&bytes[..to_write]);
        (EncoderResult::OutputFull, to_write, to_write)
    }
}

// Any copyright to the test code below this comment is dedicated to the
// Public Domain. http://creativecommons.org/publicdomain/zero/1.0/

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::*;

    //    fn decode_utf8_to_utf16(bytes: &[u8], expect: &[u16]) {
    //        decode_to_utf16_without_replacement(UTF_8, bytes, expect);
    //    }

    fn decode_utf8_to_utf8(bytes: &[u8], expect: &str) {
        decode_to_utf8(UTF_8, bytes, expect);
    }

    fn decode_valid_utf8(string: &str) {
        decode_utf8_to_utf8(string.as_bytes(), string);
    }

    fn encode_utf8_from_utf16(string: &[u16], expect: &[u8]) {
        encode_from_utf16(UTF_8, string, expect);
    }

    fn encode_utf8_from_utf8(string: &str, expect: &[u8]) {
        encode_from_utf8(UTF_8, string, expect);
    }

    #[test]
    fn test_utf8_decode() {
        // Empty
        decode_valid_utf8("");
        // ASCII
        decode_valid_utf8("ab");
        // Low BMP
        decode_valid_utf8("a\u{E4}Z");
        // High BMP
        decode_valid_utf8("a\u{2603}Z");
        // Astral
        decode_valid_utf8("a\u{1F4A9}Z");
        // Low BMP with last byte missing
        decode_utf8_to_utf8(b"a\xC3Z", "a\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\xC3", "a\u{FFFD}");
        // High BMP with last byte missing
        decode_utf8_to_utf8(b"a\xE2\x98Z", "a\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\xE2\x98", "a\u{FFFD}");
        // Astral with last byte missing
        decode_utf8_to_utf8(b"a\xF0\x9F\x92Z", "a\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\xF0\x9F\x92", "a\u{FFFD}");
        // Lone highest continuation
        decode_utf8_to_utf8(b"a\xBFZ", "a\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\xBF", "a\u{FFFD}");
        // Two lone highest continuations
        decode_utf8_to_utf8(b"a\xBF\xBFZ", "a\u{FFFD}\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\xBF\xBF", "a\u{FFFD}\u{FFFD}");
        // Low BMP followed by lowest lone continuation
        decode_utf8_to_utf8(b"a\xC3\xA4\x80Z", "a\u{E4}\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\xC3\xA4\x80", "a\u{E4}\u{FFFD}");
        // Low BMP followed by highest lone continuation
        decode_utf8_to_utf8(b"a\xC3\xA4\xBFZ", "a\u{E4}\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\xC3\xA4\xBF", "a\u{E4}\u{FFFD}");
        // High BMP followed by lowest lone continuation
        decode_utf8_to_utf8(b"a\xE2\x98\x83\x80Z", "a\u{2603}\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\xE2\x98\x83\x80", "a\u{2603}\u{FFFD}");
        // High BMP followed by highest lone continuation
        decode_utf8_to_utf8(b"a\xE2\x98\x83\xBFZ", "a\u{2603}\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\xE2\x98\x83\xBF", "a\u{2603}\u{FFFD}");
        // Astral followed by lowest lone continuation
        decode_utf8_to_utf8(b"a\xF0\x9F\x92\xA9\x80Z", "a\u{1F4A9}\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\xF0\x9F\x92\xA9\x80", "a\u{1F4A9}\u{FFFD}");
        // Astral followed by highest lone continuation
        decode_utf8_to_utf8(b"a\xF0\x9F\x92\xA9\xBFZ", "a\u{1F4A9}\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\xF0\x9F\x92\xA9\xBF", "a\u{1F4A9}\u{FFFD}");

        // Boundary conditions
        // Lowest single-byte
        decode_valid_utf8("Z\x00");
        decode_valid_utf8("Z\x00Z");
        // Lowest single-byte as two-byte overlong sequence
        decode_utf8_to_utf8(b"a\xC0\x80", "a\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xC0\x80Z", "a\u{FFFD}\u{FFFD}Z");
        // Lowest single-byte as three-byte overlong sequence
        decode_utf8_to_utf8(b"a\xE0\x80\x80", "a\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xE0\x80\x80Z", "a\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // Lowest single-byte as four-byte overlong sequence
        decode_utf8_to_utf8(b"a\xF0\x80\x80\x80", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x80\x80\x80Z", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // One below lowest single-byte
        decode_utf8_to_utf8(b"a\xFF", "a\u{FFFD}");
        decode_utf8_to_utf8(b"a\xFFZ", "a\u{FFFD}Z");
        // Highest single-byte
        decode_valid_utf8("a\x7F");
        decode_valid_utf8("a\x7FZ");
        // Highest single-byte as two-byte overlong sequence
        decode_utf8_to_utf8(b"a\xC1\xBF", "a\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xC1\xBFZ", "a\u{FFFD}\u{FFFD}Z");
        // Highest single-byte as three-byte overlong sequence
        decode_utf8_to_utf8(b"a\xE0\x81\xBF", "a\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xE0\x81\xBFZ", "a\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // Highest single-byte as four-byte overlong sequence
        decode_utf8_to_utf8(b"a\xF0\x80\x81\xBF", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x80\x81\xBFZ", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // One past highest single byte (also lone continuation)
        decode_utf8_to_utf8(b"a\x80Z", "a\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\x80", "a\u{FFFD}");
        // Two lone continuations
        decode_utf8_to_utf8(b"a\x80\x80Z", "a\u{FFFD}\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\x80\x80", "a\u{FFFD}\u{FFFD}");
        // Three lone continuations
        decode_utf8_to_utf8(b"a\x80\x80\x80Z", "a\u{FFFD}\u{FFFD}\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\x80\x80\x80", "a\u{FFFD}\u{FFFD}\u{FFFD}");
        // Four lone continuations
        decode_utf8_to_utf8(b"a\x80\x80\x80\x80Z", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}Z");
        decode_utf8_to_utf8(b"a\x80\x80\x80\x80", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
        // Lowest two-byte
        decode_utf8_to_utf8(b"a\xC2\x80", "a\u{0080}");
        decode_utf8_to_utf8(b"a\xC2\x80Z", "a\u{0080}Z");
        // Lowest two-byte as three-byte overlong sequence
        decode_utf8_to_utf8(b"a\xE0\x82\x80", "a\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xE0\x82\x80Z", "a\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // Lowest two-byte as four-byte overlong sequence
        decode_utf8_to_utf8(b"a\xF0\x80\x82\x80", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x80\x82\x80Z", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // Lead one below lowest two-byte
        decode_utf8_to_utf8(b"a\xC1\x80", "a\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xC1\x80Z", "a\u{FFFD}\u{FFFD}Z");
        // Trail one below lowest two-byte
        decode_utf8_to_utf8(b"a\xC2\x7F", "a\u{FFFD}\u{007F}");
        decode_utf8_to_utf8(b"a\xC2\x7FZ", "a\u{FFFD}\u{007F}Z");
        // Highest two-byte
        decode_utf8_to_utf8(b"a\xDF\xBF", "a\u{07FF}");
        decode_utf8_to_utf8(b"a\xDF\xBFZ", "a\u{07FF}Z");
        // Highest two-byte as three-byte overlong sequence
        decode_utf8_to_utf8(b"a\xE0\x9F\xBF", "a\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xE0\x9F\xBFZ", "a\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // Highest two-byte as four-byte overlong sequence
        decode_utf8_to_utf8(b"a\xF0\x80\x9F\xBF", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x80\x9F\xBFZ", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // Lowest three-byte
        decode_utf8_to_utf8(b"a\xE0\xA0\x80", "a\u{0800}");
        decode_utf8_to_utf8(b"a\xE0\xA0\x80Z", "a\u{0800}Z");
        // Lowest three-byte as four-byte overlong sequence
        decode_utf8_to_utf8(b"a\xF0\x80\xA0\x80", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x80\xA0\x80Z", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // Highest below surrogates
        decode_utf8_to_utf8(b"a\xED\x9F\xBF", "a\u{D7FF}");
        decode_utf8_to_utf8(b"a\xED\x9F\xBFZ", "a\u{D7FF}Z");
        // Highest below surrogates as four-byte overlong sequence
        decode_utf8_to_utf8(b"a\xF0\x8D\x9F\xBF", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x8D\x9F\xBFZ", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // First surrogate
        decode_utf8_to_utf8(b"a\xED\xA0\x80", "a\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xED\xA0\x80Z", "a\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // First surrogate as four-byte overlong sequence
        decode_utf8_to_utf8(b"a\xF0\x8D\xA0\x80", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x8D\xA0\x80Z", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // Last surrogate
        decode_utf8_to_utf8(b"a\xED\xBF\xBF", "a\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xED\xBF\xBFZ", "a\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // Last surrogate as four-byte overlong sequence
        decode_utf8_to_utf8(b"a\xF0\x8D\xBF\xBF", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x8D\xBF\xBFZ", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // Lowest above surrogates
        decode_utf8_to_utf8(b"a\xEE\x80\x80", "a\u{E000}");
        decode_utf8_to_utf8(b"a\xEE\x80\x80Z", "a\u{E000}Z");
        // Lowest above surrogates as four-byte overlong sequence
        decode_utf8_to_utf8(b"a\xF0\x8E\x80\x80", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x8E\x80\x80Z", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // Highest three-byte
        decode_utf8_to_utf8(b"a\xEF\xBF\xBF", "a\u{FFFF}");
        decode_utf8_to_utf8(b"a\xEF\xBF\xBFZ", "a\u{FFFF}Z");
        // Highest three-byte as four-byte overlong sequence
        decode_utf8_to_utf8(b"a\xF0\x8F\xBF\xBF", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF0\x8F\xBF\xBFZ", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}Z");
        // Lowest four-byte
        decode_utf8_to_utf8(b"a\xF0\x90\x80\x80", "a\u{10000}");
        decode_utf8_to_utf8(b"a\xF0\x90\x80\x80Z", "a\u{10000}Z");
        // Highest four-byte
        decode_utf8_to_utf8(b"a\xF4\x8F\xBF\xBF", "a\u{10FFFF}");
        decode_utf8_to_utf8(b"a\xF4\x8F\xBF\xBFZ", "a\u{10FFFF}Z");
        // One past highest four-byte
        decode_utf8_to_utf8(b"a\xF4\x90\x80\x80", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF4\x90\x80\x80Z", "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}Z");

        // Highest four-byte with last byte replaced with 0xFF
        decode_utf8_to_utf8(b"a\xF4\x8F\xBF\xFF", "a\u{FFFD}\u{FFFD}");
        decode_utf8_to_utf8(b"a\xF4\x8F\xBF\xFFZ", "a\u{FFFD}\u{FFFD}Z");
    }

    #[test]
    fn test_utf8_encode() {
        // Empty
        encode_utf8_from_utf16(&[], b"");
        encode_utf8_from_utf8("", b"");

        encode_utf8_from_utf16(&[0x0000], "\u{0000}".as_bytes());
        encode_utf8_from_utf16(&[0x007F], "\u{007F}".as_bytes());
        encode_utf8_from_utf16(&[0x0080], "\u{0080}".as_bytes());
        encode_utf8_from_utf16(&[0x07FF], "\u{07FF}".as_bytes());
        encode_utf8_from_utf16(&[0x0800], "\u{0800}".as_bytes());
        encode_utf8_from_utf16(&[0xD7FF], "\u{D7FF}".as_bytes());
        encode_utf8_from_utf16(&[0xD800], "\u{FFFD}".as_bytes());
        encode_utf8_from_utf16(&[0xD800, 0x0062], "\u{FFFD}\u{0062}".as_bytes());
        encode_utf8_from_utf16(&[0xDFFF], "\u{FFFD}".as_bytes());
        encode_utf8_from_utf16(&[0xDFFF, 0x0062], "\u{FFFD}\u{0062}".as_bytes());
        encode_utf8_from_utf16(&[0xE000], "\u{E000}".as_bytes());
        encode_utf8_from_utf16(&[0xFFFF], "\u{FFFF}".as_bytes());
        encode_utf8_from_utf16(&[0xD800, 0xDC00], "\u{10000}".as_bytes());
        encode_utf8_from_utf16(&[0xDBFF, 0xDFFF], "\u{10FFFF}".as_bytes());
        encode_utf8_from_utf16(&[0xDC00, 0xDEDE], "\u{FFFD}\u{FFFD}".as_bytes());
    }

    #[test]
    fn test_utf8_max_length_from_utf16() {
        let mut encoder = UTF_8.new_encoder();
        let mut output = [0u8; 13];
        let input = &[0x2C9Fu16, 0x2CA9u16, 0x2CA3u16, 0x2C9Fu16];
        let needed = encoder
            .max_buffer_length_from_utf16_without_replacement(input.len())
            .unwrap();
        let (result, _, _) =
            encoder.encode_from_utf16_without_replacement(input, &mut output[..needed], true);
        assert_eq!(result, EncoderResult::InputEmpty);
    }

    #[test]
    fn test_decode_bom_prefixed_split_byte_triple() {
        let mut output = [0u16; 20];
        let mut decoder = UTF_8.new_decoder();
        {
            let needed = decoder.max_utf16_buffer_length(1).unwrap();
            let (result, read, written, had_errors) =
                decoder.decode_to_utf16(b"\xEF", &mut output[..needed], false);
            assert_eq!(result, CoderResult::InputEmpty);
            assert_eq!(read, 1);
            assert_eq!(written, 0);
            assert!(!had_errors);
        }
        {
            let needed = decoder.max_utf16_buffer_length(1).unwrap();
            let (result, read, written, had_errors) =
                decoder.decode_to_utf16(b"\xBF", &mut output[..needed], false);
            assert_eq!(result, CoderResult::InputEmpty);
            assert_eq!(read, 1);
            assert_eq!(written, 0);
            assert!(!had_errors);
        }
        {
            let needed = decoder.max_utf16_buffer_length(1).unwrap();
            let (result, read, written, had_errors) =
                decoder.decode_to_utf16(b"\xBE", &mut output[..needed], true);
            assert_eq!(result, CoderResult::InputEmpty);
            assert_eq!(read, 1);
            assert_eq!(written, 1);
            assert!(!had_errors);
            assert_eq!(output[0], 0xFFFE);
        }
    }

    #[test]
    fn test_decode_bom_prefixed_split_byte_pair() {
        let mut output = [0u16; 20];
        let mut decoder = UTF_8.new_decoder();
        {
            let needed = decoder.max_utf16_buffer_length(1).unwrap();
            let (result, read, written, had_errors) =
                decoder.decode_to_utf16(b"\xEF", &mut output[..needed], false);
            assert_eq!(result, CoderResult::InputEmpty);
            assert_eq!(read, 1);
            assert_eq!(written, 0);
            assert!(!had_errors);
        }
        {
            let needed = decoder.max_utf16_buffer_length(1).unwrap();
            let (result, read, written, had_errors) =
                decoder.decode_to_utf16(b"\xBC", &mut output[..needed], true);
            assert_eq!(result, CoderResult::InputEmpty);
            assert_eq!(read, 1);
            assert_eq!(written, 1);
            assert!(had_errors);
            assert_eq!(output[0], 0xFFFD);
        }
    }

    #[test]
    fn test_decode_bom_prefix() {
        let mut output = [0u16; 20];
        let mut decoder = UTF_8.new_decoder();
        {
            let needed = decoder.max_utf16_buffer_length(1).unwrap();
            let (result, read, written, had_errors) =
                decoder.decode_to_utf16(b"\xEF", &mut output[..needed], true);
            assert_eq!(result, CoderResult::InputEmpty);
            assert_eq!(read, 1);
            assert_eq!(written, 1);
            assert!(had_errors);
            assert_eq!(output[0], 0xFFFD);
        }
    }

}
