// Copyright Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use any_all_workaround::all_mask8x16;
use any_all_workaround::all_mask16x8;
use any_all_workaround::any_mask8x16;
use any_all_workaround::any_mask16x8;
use core::simd::ToBytes;
use core::simd::cmp::SimdPartialEq;
use core::simd::cmp::SimdPartialOrd;
use core::simd::mask8x16;
use core::simd::mask16x8;
use core::simd::simd_swizzle;
use core::simd::u8x16;
use core::simd::u16x8;

// TODO: Migrate unaligned access to stdlib code if/when the RFC
// https://github.com/rust-lang/rfcs/pull/1725 is implemented.

/// Safety invariant: ptr must be valid for an unaligned read of 16 bytes
#[inline(always)]
pub unsafe fn load16_unaligned(ptr: *const u8) -> u8x16 {
    let mut simd = ::core::mem::MaybeUninit::<u8x16>::uninit();
    unsafe {
        ::core::ptr::copy_nonoverlapping(ptr, simd.as_mut_ptr() as *mut u8, 16);
        // Safety: copied 16 bytes of initialized memory into this, it is now initialized
        simd.assume_init()
    }
}

/// Safety invariant: ptr must be valid for an unaligned store of 16 bytes
#[inline(always)]
pub unsafe fn store16_unaligned(ptr: *mut u8, s: u8x16) {
    unsafe {
        ::core::ptr::copy_nonoverlapping(&s as *const u8x16 as *const u8, ptr, 16);
    }
}

/// Safety invariant: ptr must be valid for an unaligned read of 16 bytes
#[inline(always)]
pub unsafe fn load8_unaligned(ptr: *const u16) -> u16x8 {
    let mut simd = ::core::mem::MaybeUninit::<u16x8>::uninit();
    unsafe {
        ::core::ptr::copy_nonoverlapping(ptr as *const u8, simd.as_mut_ptr() as *mut u8, 16);
        // Safety: copied 16 bytes of initialized memory into this, it is now initialized
        simd.assume_init()
    }
}

/// Safety invariant: ptr must be valid for an unaligned store of 16 bytes
#[inline(always)]
pub unsafe fn store8_unaligned(ptr: *mut u16, s: u16x8) {
    unsafe {
        ::core::ptr::copy_nonoverlapping(&s as *const u16x8 as *const u8, ptr as *mut u8, 16);
    }
}

cfg_if! {
    if #[cfg(all(target_feature = "sse2", target_arch = "x86_64"))] {
        use core::arch::x86_64::__m128i;
        use core::arch::x86_64::_mm_movemask_epi8;
        use core::arch::x86_64::_mm_packus_epi16;
    } else if #[cfg(all(target_feature = "sse2", target_arch = "x86"))] {
        use core::arch::x86::__m128i;
        use core::arch::x86::_mm_movemask_epi8;
        use core::arch::x86::_mm_packus_epi16;
    } else if #[cfg(target_arch = "aarch64")]{
        use core::arch::aarch64::vmaxvq_u8;
        use core::arch::aarch64::vmaxvq_u16;
    } else {

    }
}

// #[inline(always)]
// fn simd_byte_swap_u8(s: u8x16) -> u8x16 {
//     unsafe {
//         shuffle!(s, s, [1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14])
//     }
// }

// #[inline(always)]
// pub fn simd_byte_swap(s: u16x8) -> u16x8 {
//     to_u16_lanes(simd_byte_swap_u8(to_u8_lanes(s)))
// }

#[inline(always)]
pub fn simd_byte_swap(s: u16x8) -> u16x8 {
    let left = s << 8;
    let right = s >> 8;
    left | right
}

#[inline(always)]
pub fn to_u16_lanes(s: u8x16) -> u16x8 {
    u16x8::from_ne_bytes(s)
}

cfg_if! {
    if #[cfg(target_feature = "sse2")] {

        // Expose low-level mask instead of higher-level conclusion,
        // because the non-ASCII case would perform less well otherwise.
        // Safety-usable invariant: This returned value is whether each high bit is set
        #[inline(always)]
        pub fn mask_ascii(s: u8x16) -> i32 {
            unsafe {
                _mm_movemask_epi8(s.into())
            }
        }

    } else {

    }
}

cfg_if! {
    if #[cfg(target_feature = "sse2")] {
        #[inline(always)]
        pub fn simd_is_ascii(s: u8x16) -> bool {
            unsafe {
                // Safety: We have cfg()d the correct platform
                _mm_movemask_epi8(s.into()) == 0
            }
        }
    } else if #[cfg(target_arch = "aarch64")]{
        #[inline(always)]
        pub fn simd_is_ascii(s: u8x16) -> bool {
            unsafe {
                // Safety: We have cfg()d the correct platform
                vmaxvq_u8(s.into()) < 0x80
            }
        }
    } else {
        #[inline(always)]
        pub fn simd_is_ascii(s: u8x16) -> bool {
            // This optimizes better on ARM than
            // the lt formulation.
            let highest_ascii = u8x16::splat(0x7F);
            !any_mask8x16(s.simd_gt(highest_ascii))
        }
    }
}

cfg_if! {
    if #[cfg(target_feature = "sse2")] {
        #[inline(always)]
        pub fn simd_is_str_latin1(s: u8x16) -> bool {
            if simd_is_ascii(s) {
                return true;
            }
            let above_str_latin1 = u8x16::splat(0xC4);
            s.simd_lt(above_str_latin1).all()
        }
    } else if #[cfg(target_arch = "aarch64")]{
        #[inline(always)]
        pub fn simd_is_str_latin1(s: u8x16) -> bool {
            unsafe {
                // Safety: We have cfg()d the correct platform
                vmaxvq_u8(s.into()) < 0xC4
            }
        }
    } else {
        #[inline(always)]
        pub fn simd_is_str_latin1(s: u8x16) -> bool {
            let above_str_latin1 = u8x16::splat(0xC4);
            all_mask8x16(s.simd_lt(above_str_latin1))
        }
    }
}

cfg_if! {
    if #[cfg(target_arch = "aarch64")]{
        #[inline(always)]
        pub fn simd_is_basic_latin(s: u16x8) -> bool {
            unsafe {
                // Safety: We have cfg()d the correct platform
                vmaxvq_u16(s.into()) < 0x80
            }
        }

        #[inline(always)]
        pub fn simd_is_latin1(s: u16x8) -> bool {
            unsafe {
                // Safety: We have cfg()d the correct platform
                vmaxvq_u16(s.into()) < 0x100
            }
        }
    } else {
        #[inline(always)]
        pub fn simd_is_basic_latin(s: u16x8) -> bool {
            let above_ascii = u16x8::splat(0x80);
            all_mask16x8(s.simd_lt(above_ascii))
        }

        #[inline(always)]
        pub fn simd_is_latin1(s: u16x8) -> bool {
            // For some reason, on SSE2 this formulation
            // seems faster in this case while the above
            // function is better the other way round...
            let highest_latin1 = u16x8::splat(0xFF);
            !any_mask16x8(s.simd_gt(highest_latin1))
        }
    }
}

#[inline(always)]
pub fn contains_surrogates(s: u16x8) -> bool {
    let mask = u16x8::splat(0xF800);
    let surrogate_bits = u16x8::splat(0xD800);
    any_mask16x8((s & mask).simd_eq(surrogate_bits))
}

cfg_if! {
    if #[cfg(target_arch = "aarch64")]{
        macro_rules! aarch64_return_false_if_below_hebrew {
            ($s:ident) => ({
                unsafe {
                    // Safety: We have cfg()d the correct platform
                    if vmaxvq_u16($s.into()) < 0x0590 {
                        return false;
                    }
                }
            })
        }

        macro_rules! non_aarch64_return_false_if_all {
            ($s:ident) => ()
        }
    } else {
        macro_rules! aarch64_return_false_if_below_hebrew {
            ($s:ident) => ()
        }

        macro_rules! non_aarch64_return_false_if_all {
            ($s:ident) => ({
                if all_mask16x8($s) {
                    return false;
                }
            })
        }
    }
}

macro_rules! in_range16x8 {
    ($s:ident, $start:expr, $end:expr) => {{
        // SIMD sub is wrapping
        ($s - u16x8::splat($start)).simd_lt(u16x8::splat($end - $start))
    }};
}

#[inline(always)]
pub(crate) fn is_u16x8_bidi(s: u16x8) -> bool {
    // We try to first quickly refute the RTLness of the vector. If that
    // fails, we do the real RTL check, so in that case we end up wasting
    // the work for the up-front quick checks. Even the quick-check is
    // two-fold in order to return `false` ASAP if everything is below
    // Hebrew.

    aarch64_return_false_if_below_hebrew!(s);

    let below_hebrew = s.simd_lt(u16x8::splat(0x0590));

    non_aarch64_return_false_if_all!(below_hebrew);

    if all_mask16x8(
        below_hebrew | in_range16x8!(s, 0x0900, 0x200F) | in_range16x8!(s, 0x2068, 0xD802),
    ) {
        return false;
    }

    // Quick refutation failed. Let's do the full check.

    any_mask16x8(
        (in_range16x8!(s, 0x0590, 0x0900)
            | in_range16x8!(s, 0xFB1D, 0xFE00)
            | in_range16x8!(s, 0xFE70, 0xFEFF)
            | in_range16x8!(s, 0xD802, 0xD804)
            | in_range16x8!(s, 0xD83A, 0xD83C)
            | s.simd_eq(u16x8::splat(0x200F))
            | s.simd_eq(u16x8::splat(0x202B))
            | s.simd_eq(u16x8::splat(0x202E))
            | s.simd_eq(u16x8::splat(0x2067))),
    )
}

// Code above is old code not re-vetted for Rust 1.88 and later `unsafe` removal.
// Code below has been checked as part of the Rust 1.88 and later `unsafe` removal.

use crate::ascii::STRIDE;

const HALF_STRIDE: usize = STRIDE / 2;

#[inline(always)]
pub fn simd_unpack(s: u8x16) -> (u16x8, u16x8) {
    let (first, second) = s.interleave(u8x16::splat(0));
    (u16x8::from_ne_bytes(first), u16x8::from_ne_bytes(second))
}

cfg_if! {
    if #[cfg(target_feature = "sse2")] {
        #[inline(always)]
        fn movemask(s: core::arch::x86_64::__m128i) -> u32 {
            // Safety: We have cfg()d the correct platform
            (unsafe { _mm_movemask_epi8(s) }) as u32
        }

        #[inline(always)]
        pub fn simd_pack(a: u16x8, b: u16x8) -> u8x16 {
            unsafe {
                // Safety: We have cfg()d the correct platform
                _mm_packus_epi16(a.into(), b.into()).into()
            }
        }

        #[inline(always)]
        fn validate_basic_latin_simd(first_simd: u16x8, second_simd: u16x8) -> Option<usize> {
            let bound = u16x8::splat(0x7F);
            let first_mask = movemask(first_simd.simd_gt(bound).to_simd().into());
            let second_mask = movemask(second_simd.simd_gt(bound).to_simd().into());
            let combined = (second_mask << 16) | first_mask;
            if combined == 0 {
                return None;
            }
            Some((combined.trailing_zeros() / 2) as usize)
        }

        #[inline(always)]
        fn validate_bmp_simd(first_simd: u16x8, second_simd: u16x8) -> Option<usize> {
            let surrogate_bits = u16x8::splat(0xD800);
            let mask = u16x8::splat(0xF800);
            let first_mask = movemask((first_simd & mask).simd_eq(surrogate_bits).to_simd().into());
            let second_mask = movemask((second_simd & mask).simd_eq(surrogate_bits).to_simd().into());
            let combined = (second_mask << 16) | first_mask;
            if combined == 0 {
                return None;
            }
            Some((combined.trailing_zeros() / 2) as usize)
        }
    } else {
        #[inline(always)]
        pub fn simd_pack(a: u16x8, b: u16x8) -> u8x16 {
            let first: u8x16 = a.to_ne_bytes();
            let second: u8x16 = b.to_ne_bytes();
            let (ret, _) = first.deinterleave(second);
            ret
        }

        #[inline(always)]
        fn validate_basic_latin_simd(first_simd: u16x8, second_simd: u16x8) -> Option<usize> {
            let first: u8x16 = first_simd.to_ne_bytes();
            let second: u8x16 = second_simd.to_ne_bytes();
            let (low, high) = first.deinterleave(second);
            (low.simd_gt(u8x16::splat(0x7F)) | high.simd_ne(u8x16::splat(0))).first_set()
        }

        #[inline(always)]
        fn validate_bmp_simd(first_simd: u16x8, second_simd: u16x8) -> Option<usize> {
            let first: u8x16 = first_simd.to_ne_bytes();
            let second: u8x16 = second_simd.to_ne_bytes();
            let (_, high) = first.deinterleave(second);
            (high & u8x16::splat(0xF8)).simd_eq(u8x16::splat(0xD8)).first_set()
        }
    }
}

cfg_if! {
    if #[cfg(target_feature = "sse2")] {
        #[inline(always)]
        pub fn validate_latin1_str_simd(s: u8x16) -> Option<usize> {
            if simd_is_ascii(s) {
                return None;
            }
            s.simd_gt(u8x16::splat(0xC3)).first_set()
        }
    } else if #[cfg(target_arch = "aarch64")]{
        #[inline(always)]
        pub fn validate_latin1_str_simd(s: u8x16) -> Option<usize> {
            if unsafe {
                // Safety: We have cfg()d the correct platform
                vmaxvq_u8(s.into()) < 0xC4
            } {
                return None;
            }
            s.simd_gt(u8x16::splat(0xC3)).first_set()
        }
    } else {
        #[inline(always)]
        pub fn validate_latin1_str_simd(s: u8x16) -> Option<usize> {
            s.simd_gt(u8x16::splat(0xC3)).first_set()
        }
    }
}

#[inline(always)]
fn split_u16_stride(stride: &[u16; STRIDE]) -> (&[u16; HALF_STRIDE], &[u16; HALF_STRIDE]) {
    let (chunks, _) = stride.as_chunks::<HALF_STRIDE>();
    (&chunks[0], &chunks[1])
}

#[inline(always)]
fn split_u16_stride_mut(
    stride: &mut [u16; STRIDE],
) -> (&mut [u16; HALF_STRIDE], &mut [u16; HALF_STRIDE]) {
    // Can't take two mutable references to output of `as_chunks_mut`.
    let (head, tail) = stride.split_at_mut(HALF_STRIDE);
    // `as_array` requires Rust 1.93.
    (
        &mut head.as_chunks_mut::<HALF_STRIDE>().0[0],
        &mut tail.as_chunks_mut::<HALF_STRIDE>().0[0],
    )
}

#[inline(always)]
fn unpack_simd_to(src_simd: u8x16, dst_stride: &mut [u16; STRIDE]) {
    let (first, second) = simd_unpack(src_simd);
    let (dst_first, dst_second) = split_u16_stride_mut(dst_stride);
    *dst_first = first.to_array();
    *dst_second = second.to_array();
}

#[inline(always)]
fn pack_simd_to(first_simd: u16x8, second_simd: u16x8, dst_stride: &mut [u8; STRIDE]) {
    let simd = simd_pack(first_simd, second_simd);
    *dst_stride = simd.to_array();
}

#[inline(always)]
fn validate_ascii_simd(simd: u8x16) -> Option<usize> {
    let mask = simd.simd_gt(u8x16::splat(0x7F));
    mask.first_set()
}

#[inline(always)]
pub(crate) fn ascii_to_ascii_stride(
    src_stride: &[u8; STRIDE],
    dst_stride: &mut [u8; STRIDE],
) -> Option<usize> {
    let src_simd: u8x16 = (*src_stride).into();
    *dst_stride = src_simd.to_array();
    validate_ascii_simd(src_simd)
}

#[inline(always)]
pub(crate) fn ascii_to_basic_latin_stride(
    src_stride: &[u8; STRIDE],
    dst_stride: &mut [u16; STRIDE],
) -> Option<usize> {
    let src_simd: u8x16 = (*src_stride).into();
    unpack_simd_to(src_simd, dst_stride);
    validate_ascii_simd(src_simd)
}

#[inline(always)]
pub(crate) fn basic_latin_to_ascii_stride(
    src_stride: &[u16; STRIDE],
    dst_stride: &mut [u8; STRIDE],
) -> Option<usize> {
    let (src_first, src_second) = split_u16_stride(src_stride);
    let first_simd: u16x8 = (*src_first).into();
    let second_simd: u16x8 = (*src_second).into();
    pack_simd_to(first_simd, second_simd, dst_stride);
    validate_basic_latin_simd(first_simd, second_simd)
}

#[inline(always)]
pub(crate) fn validate_ascii_stride(stride: &[u8; STRIDE]) -> Option<usize> {
    let simd: u8x16 = (*stride).into();
    validate_ascii_simd(simd)
}

#[inline(always)]
pub(crate) fn ascii_to_ascii_double_stride(
    src_double_stride: &[[u8; STRIDE]; 2],
    dst_double_stride: &mut [[u8; STRIDE]; 2],
) -> Option<usize> {
    let first_simd: u8x16 = src_double_stride[0].into();
    let second_simd: u8x16 = src_double_stride[1].into();
    dst_double_stride[0] = first_simd.to_array();
    if simd_is_ascii(first_simd | second_simd) {
        dst_double_stride[1] = second_simd.to_array();
        return None;
    }
    if let Some(pos) = validate_ascii_simd(first_simd) {
        return Some(pos);
    }
    dst_double_stride[1] = second_simd.to_array();
    if let Some(pos) = validate_ascii_simd(second_simd) {
        return Some(STRIDE + pos);
    }
    debug_assert!(false);
    None
}

#[inline(always)]
pub(crate) fn ascii_to_basic_latin_double_stride(
    src_double_stride: &[[u8; STRIDE]; 2],
    dst_double_stride: &mut [[u16; STRIDE]; 2],
) -> Option<usize> {
    let first_simd: u8x16 = src_double_stride[0].into();
    let second_simd: u8x16 = src_double_stride[1].into();
    unpack_simd_to(first_simd, &mut dst_double_stride[0]);
    if simd_is_ascii(first_simd | second_simd) {
        unpack_simd_to(second_simd, &mut dst_double_stride[1]);
        return None;
    }
    if let Some(pos) = validate_ascii_simd(first_simd) {
        return Some(pos);
    }
    unpack_simd_to(second_simd, &mut dst_double_stride[1]);
    if let Some(pos) = validate_ascii_simd(second_simd) {
        return Some(STRIDE + pos);
    }
    debug_assert!(false);
    None
}

#[inline(always)]
pub(crate) fn basic_latin_to_ascii_double_stride(
    src_double_stride: &[[u16; STRIDE]; 2],
    dst_double_stride: &mut [[u8; STRIDE]; 2],
) -> Option<usize> {
    let (src_first, src_second) = split_u16_stride(&src_double_stride[0]);
    let first_simd: u16x8 = (*src_first).into();
    let second_simd: u16x8 = (*src_second).into();
    let (src_third, src_fourth) = split_u16_stride(&src_double_stride[1]);
    let third_simd: u16x8 = (*src_third).into();
    let fourth_simd: u16x8 = (*src_fourth).into();
    pack_simd_to(first_simd, second_simd, &mut dst_double_stride[0]);
    if simd_is_basic_latin(first_simd | second_simd | third_simd | fourth_simd) {
        pack_simd_to(third_simd, fourth_simd, &mut dst_double_stride[1]);
        return None;
    }
    if let Some(pos) = validate_basic_latin_simd(first_simd, second_simd) {
        return Some(pos);
    }
    pack_simd_to(third_simd, fourth_simd, &mut dst_double_stride[1]);
    if let Some(pos) = validate_basic_latin_simd(third_simd, fourth_simd) {
        return Some(STRIDE + pos);
    }
    debug_assert!(false);
    None
}

#[inline(always)]
pub(crate) fn validate_ascii_double_stride(double_stride: &[[u8; STRIDE]; 2]) -> Option<usize> {
    let first_simd: u8x16 = double_stride[0].into();
    let second_simd: u8x16 = double_stride[1].into();
    if simd_is_ascii(first_simd | second_simd) {
        return None;
    }
    if let Some(pos) = validate_ascii_simd(first_simd) {
        return Some(pos);
    }
    if let Some(pos) = validate_ascii_simd(second_simd) {
        return Some(STRIDE + pos);
    }
    debug_assert!(false);
    None
}

#[inline(always)]
pub(crate) fn unpack_stride(src_stride: &[u8; STRIDE], dst_stride: &mut [u16; STRIDE]) {
    let src_simd: u8x16 = (*src_stride).into();
    unpack_simd_to(src_simd, dst_stride);
}

#[inline(always)]
pub(crate) fn pack_stride(src_stride: &[u16; STRIDE], dst_stride: &mut [u8; STRIDE]) {
    let (src_first, src_second) = split_u16_stride(src_stride);
    let first_simd: u16x8 = (*src_first).into();
    let second_simd: u16x8 = (*src_second).into();
    pack_simd_to(first_simd, second_simd, dst_stride);
}

#[inline(always)]
pub(crate) fn validate_bmp_stride(stride: &[u16; STRIDE]) -> Option<usize> {
    let (first, second) = split_u16_stride(stride);
    let first_simd: u16x8 = (*first).into();
    let second_simd: u16x8 = (*second).into();
    validate_bmp_simd(first_simd, second_simd)
}

#[inline(always)]
pub(crate) fn validate_latin1_str_stride(stride: &[u8; STRIDE]) -> Option<usize> {
    let simd: u8x16 = (*stride).into();
    validate_latin1_str_simd(simd)
}

#[inline(always)]
pub(crate) fn is_half_stride_bidi(half_stride: &[u16; STRIDE / 2]) -> bool {
    let simd: u16x8 = (*half_stride).into();
    is_u16x8_bidi(simd)
}

#[cfg(test)]
#[cfg(feature = "alloc")]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_unpack() {
        let ascii: [u8; 16] = [
            0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x76,
        ];
        let basic_latin: [u16; 16] = [
            0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x76,
        ];
        let simd = unsafe { load16_unaligned(ascii.as_ptr()) };
        let mut vec = Vec::with_capacity(16);
        vec.resize(16, 0u16);
        let (first, second) = simd_unpack(simd);
        let ptr = vec.as_mut_ptr();
        unsafe {
            store8_unaligned(ptr, first);
            store8_unaligned(ptr.add(8), second);
        }
        assert_eq!(&vec[..], &basic_latin[..]);
    }

    #[test]
    fn test_simd_is_basic_latin_success() {
        let ascii: [u8; 16] = [
            0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x76,
        ];
        let basic_latin: [u16; 16] = [
            0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x76,
        ];
        let first = unsafe { load8_unaligned(basic_latin.as_ptr()) };
        let second = unsafe { load8_unaligned(basic_latin.as_ptr().add(8)) };
        let mut vec = Vec::with_capacity(16);
        vec.resize(16, 0u8);
        let ptr = vec.as_mut_ptr();
        assert!(simd_is_basic_latin(first | second));
        unsafe {
            store16_unaligned(ptr, simd_pack(first, second));
        }
        assert_eq!(&vec[..], &ascii[..]);
    }

    #[test]
    fn test_simd_is_basic_latin_c0() {
        let input: [u16; 16] = [
            0x61, 0x62, 0x63, 0x81, 0x65, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x76,
        ];
        let first = unsafe { load8_unaligned(input.as_ptr()) };
        let second = unsafe { load8_unaligned(input.as_ptr().add(8)) };
        assert!(!simd_is_basic_latin(first | second));
    }

    #[test]
    fn test_simd_is_basic_latin_0fff() {
        let input: [u16; 16] = [
            0x61, 0x62, 0x63, 0x0FFF, 0x65, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x76,
        ];
        let first = unsafe { load8_unaligned(input.as_ptr()) };
        let second = unsafe { load8_unaligned(input.as_ptr().add(8)) };
        assert!(!simd_is_basic_latin(first | second));
    }

    #[test]
    fn test_simd_is_basic_latin_ffff() {
        let input: [u16; 16] = [
            0x61, 0x62, 0x63, 0xFFFF, 0x65, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x76,
        ];
        let first = unsafe { load8_unaligned(input.as_ptr()) };
        let second = unsafe { load8_unaligned(input.as_ptr().add(8)) };
        assert!(!simd_is_basic_latin(first | second));
    }

    #[test]
    fn test_simd_is_ascii_success() {
        let ascii: [u8; 16] = [
            0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x76,
        ];
        let simd = unsafe { load16_unaligned(ascii.as_ptr()) };
        assert!(simd_is_ascii(simd));
    }

    #[test]
    fn test_simd_is_ascii_failure() {
        let input: [u8; 16] = [
            0x61, 0x62, 0x63, 0x64, 0x81, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x76,
        ];
        let simd = unsafe { load16_unaligned(input.as_ptr()) };
        assert!(!simd_is_ascii(simd));
    }

    #[cfg(target_feature = "sse2")]
    #[test]
    fn test_check_ascii() {
        let input: [u8; 16] = [
            0x61, 0x62, 0x63, 0x64, 0x81, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x76,
        ];
        let simd = unsafe { load16_unaligned(input.as_ptr()) };
        let mask = mask_ascii(simd);
        assert_ne!(mask, 0);
        assert_eq!(mask.trailing_zeros(), 4);
    }

    #[test]
    fn test_alu() {
        let input: [u8; 16] = [
            0x61, 0x62, 0x63, 0x64, 0x81, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x76,
        ];
        let mut alu = 0u64;
        unsafe {
            ::core::ptr::copy_nonoverlapping(input.as_ptr(), &mut alu as *mut u64 as *mut u8, 8);
        }
        let masked = alu & 0x8080808080808080;
        assert_eq!(masked.trailing_zeros(), 39);
    }
}
