// Copyright Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// It's assumed that in due course Rust will have explicit SIMD but will not
// be good at run-time selection of SIMD vs. no-SIMD. In such a future,
// x86_64 will always use SSE2 and 32-bit x86 will use SSE2 when compiled with
// a Mozilla-shipped rustc. SIMD support and especially detection on ARM is a
// mess. Under the circumstances, it seems to make sense to optimize the ALU
// case for ARMv7 rather than x86. Annoyingly, I was unable to get useful
// numbers of the actual ARMv7 CPU I have access to, because (thermal?)
// throttling kept interfering. Since Raspberry Pi 3 (ARMv8 core but running
// ARMv7 code) produced reproducible performance numbers, that's the ARM
// computer that this code ended up being optimized for in the ALU case.
// Less popular CPU architectures simply get the approach that was chosen based
// on Raspberry Pi 3 measurements. The UTF-16 and UTF-8 ALU cases take
// different approaches based on benchmarking on Raspberry Pi 3.

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
