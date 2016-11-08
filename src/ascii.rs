// Copyright 2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use simd_funcs::*;

macro_rules! ascii_naive {
    ($name:ident,
     $src_unit:ty,
     $dst_unit:ty) => (
    #[inline(always)]
    pub unsafe fn $name(src: *const $src_unit, dst: *mut $dst_unit, len: usize) -> Option<($src_unit, usize)> {
// Yes, manually omitting the bound check here matters
// a lot for perf.
        for i in 0..len {
            let code_unit = *(src.offset(i as isize));
            if code_unit > 127 {
                return Some((code_unit, i));
            }
            *(dst.offset(i as isize)) = code_unit as $dst_unit;
        }
        return None;
    });
}

macro_rules! ascii_alu {
    ($name:ident,
     $src_unit:ty,
     $dst_unit:ty,
     $stride_fn:ident) => (
    #[inline(always)]
    pub unsafe fn $name(src: *const $src_unit, dst: *mut $dst_unit, len: usize) -> Option<($src_unit, usize)> {
        let mut offset = 0usize;
// XXX should we have more branchy code to move the pointers to
// alignment if they aren't aligned but could align after
// processing a few code units?
        if (STRIDE_SIZE <= len && ((src as usize) & ALIGNMENT_MASK) == 0) &&
           (((dst as usize) & ALIGNMENT_MASK) == 0) {
// XXX stdlib's UTF-8 validation reads two words at a time
            loop {
                if !$stride_fn(src.offset(offset as isize) as *const usize,
                               dst.offset(offset as isize) as *mut usize) {
                    break;
                }
                offset += STRIDE_SIZE;
                if offset + STRIDE_SIZE > len {
                    break;
                }
            }
        }
        while offset < len {
            let code_unit = *(src.offset(offset as isize));
            if code_unit > 127 {
                return Some((code_unit, offset));
            }
            *(dst.offset(offset as isize)) = code_unit as $dst_unit;
            offset += 1;
        }
        return None;
    });
}

macro_rules! ascii_simd {
    ($name:ident,
     $src_unit:ty,
     $dst_unit:ty,
     $stride_both_aligned:ident,
     $stride_src_aligned:ident,
     $stride_dst_aligned:ident,
     $stride_neither_aligned:ident) => (
    #[inline(always)]
    pub unsafe fn $name(src: *const $src_unit, dst: *mut $dst_unit, len: usize) -> Option<($src_unit, usize)> {
        let mut offset = 0usize;
// XXX should we have more branchy code to move the pointers to
// alignment if they aren't aligned but could align after
// processing a few code units?
        if STRIDE_SIZE <= len {
// XXX Should we first process one stride unconditinoally as unaligned to
// avoid the cost of the branchiness below if the first stride fails anyway?
// XXX Should we just use unaligned SSE2 access unconditionally? It seems that
// on Haswell, it would make sense to just use unaligned and not bother
// checking. Need to benchmark older architectures before deciding.
            let dst_masked = (dst as usize) & ALIGNMENT_MASK;
            if ((src as usize) & ALIGNMENT_MASK) == 0 {
                if dst_masked == 0 {
                    loop {
                        if !$stride_both_aligned(src.offset(offset as isize),
                                                 dst.offset(offset as isize)) {
                            break;
                        }
                        offset += STRIDE_SIZE;
                        if offset + STRIDE_SIZE > len {
                            break;
                        }
                    }
                } else {
                    loop {
                        if !$stride_src_aligned(src.offset(offset as isize),
                                                dst.offset(offset as isize)) {
                            break;
                        }
                        offset += STRIDE_SIZE;
                        if offset + STRIDE_SIZE > len {
                            break;
                        }
                    }
                }
            } else {
                if dst_masked == 0 {
                    loop {
                        if !$stride_dst_aligned(src.offset(offset as isize),
                                                dst.offset(offset as isize)) {
                            break;
                        }
                        offset += STRIDE_SIZE;
                        if offset + STRIDE_SIZE > len {
                            break;
                        }
                    }
                } else {
                    loop {
                        if !$stride_neither_aligned(src.offset(offset as isize),
                                                    dst.offset(offset as isize)) {
                            break;
                        }
                        offset += STRIDE_SIZE;
                        if offset + STRIDE_SIZE > len {
                            break;
                        }
                    }
                }
            }
        }
        while offset < len {
            let code_unit = *(src.offset(offset as isize));
            if code_unit > 127 {
                return Some((code_unit, offset));
            }
            *(dst.offset(offset as isize)) = code_unit as $dst_unit;
            offset += 1;
        }
        return None;
    });
}

macro_rules! ascii_to_ascii_simd_stride {
    ($name:ident,
     $load:ident,
     $store:ident) => (
    #[inline(always)]
    pub unsafe fn $name(src: *const u8, dst: *mut u8) -> bool {
        let simd = $load(src);
        if !is_ascii(simd) {
            return false;
        }
        $store(dst, simd);
        return true;
    });
}

macro_rules! ascii_to_basic_latin_simd_stride {
    ($name:ident,
     $load:ident,
     $store:ident) => (
    #[inline(always)]
    pub unsafe fn $name(src: *const u8, dst: *mut u16) -> bool {
        let simd = $load(src);
        if !is_ascii(simd) {
            return false;
        }
        let (first, second) = unpack(simd);
        $store(dst, first);
        $store(dst.offset(8), second);
        return true;
    });
}

macro_rules! basic_latin_to_ascii_simd_stride {
    ($name:ident,
     $load:ident,
     $store:ident) => (
    #[inline(always)]
    pub unsafe fn $name(src: *const u16, dst: *mut u8) -> bool {
        let first = $load(src);
        let second = $load(src.offset(8));
        match pack_basic_latin(first, second) {
            Some(packed) => {
                $store(dst, packed);
                true
            },
            None => false,
        }
    });
}

//    let first = (0xFF000000_00000000usize & word) | ((0x00FF0000_00000000usize & word) >> 8) |
//                ((0x0000FF00_00000000usize & word) >> 16) |
//                ((0x000000FF_00000000usize & word) >> 24);
//    let second = ((0x00000000_FF000000usize & word) << 32) |
//                 ((0x00000000_00FF0000usize & word) << 24) |
//                 ((0x00000000_0000FF00usize & word) << 16) |
//                 ((0x00000000_000000FFusize & word) << 8);

cfg_if! {
    if #[cfg(target_feature = "sse2")] {
// SIMD

        const STRIDE_SIZE: usize = 16;

        const ALIGNMENT_MASK: usize = 15;

        ascii_to_ascii_simd_stride!(ascii_to_ascii_stride_both_aligned, load16_aligned, store16_aligned);
        ascii_to_ascii_simd_stride!(ascii_to_ascii_stride_src_aligned, load16_aligned, store16_unaligned);
        ascii_to_ascii_simd_stride!(ascii_to_ascii_stride_dst_aligned, load16_unaligned, store16_aligned);
        ascii_to_ascii_simd_stride!(ascii_to_ascii_stride_neither_aligned, load16_unaligned, store16_unaligned);

        ascii_to_basic_latin_simd_stride!(ascii_to_basic_latin_stride_both_aligned, load16_aligned, store8_aligned);
        ascii_to_basic_latin_simd_stride!(ascii_to_basic_latin_stride_src_aligned, load16_aligned, store8_unaligned);
        ascii_to_basic_latin_simd_stride!(ascii_to_basic_latin_stride_dst_aligned, load16_unaligned, store8_aligned);
        ascii_to_basic_latin_simd_stride!(ascii_to_basic_latin_stride_neither_aligned, load16_unaligned, store8_unaligned);

        basic_latin_to_ascii_simd_stride!(basic_latin_to_ascii_stride_both_aligned, load8_aligned, store16_aligned);
        basic_latin_to_ascii_simd_stride!(basic_latin_to_ascii_stride_src_aligned, load8_aligned, store16_unaligned);
        basic_latin_to_ascii_simd_stride!(basic_latin_to_ascii_stride_dst_aligned, load8_unaligned, store16_aligned);
        basic_latin_to_ascii_simd_stride!(basic_latin_to_ascii_stride_neither_aligned, load8_unaligned, store16_unaligned);

        ascii_simd!(ascii_to_ascii, u8, u8, ascii_to_ascii_stride_both_aligned, ascii_to_ascii_stride_src_aligned, ascii_to_ascii_stride_dst_aligned, ascii_to_ascii_stride_neither_aligned);
        ascii_simd!(ascii_to_basic_latin, u8, u16, ascii_to_basic_latin_stride_both_aligned, ascii_to_basic_latin_stride_src_aligned, ascii_to_basic_latin_stride_dst_aligned, ascii_to_basic_latin_stride_neither_aligned);
        ascii_simd!(basic_latin_to_ascii, u16, u8, basic_latin_to_ascii_stride_both_aligned, basic_latin_to_ascii_stride_src_aligned, basic_latin_to_ascii_stride_dst_aligned, basic_latin_to_ascii_stride_neither_aligned);
    } else if #[cfg(all(target_endian = "little", target_pointer_width = "64"))] {
// Aligned ALU word, little endian, 64-bit

        const STRIDE_SIZE: usize = 8;

        const ALIGNMENT_MASK: usize = 7;

        #[inline(always)]
        unsafe fn ascii_to_ascii_stride_64(src: *const usize, dst: *mut usize) -> bool {
            let word = *src;
// Check if the word contains non-ASCII
            if (word & 0x80808080_80808080usize) != 0 {
                return false;
            }
            *dst = word;
            return true;
        }

        #[inline(always)]
        unsafe fn ascii_to_basic_latin_stride_little_64(src: *const usize, dst: *mut usize) -> bool {
            let word = *src;
// Check if the word contains non-ASCII
            if (word & 0x80808080_80808080usize) != 0 {
                return false;
            }
            let first = ((0x00000000_FF000000usize & word) << 24) |
                        ((0x00000000_00FF0000usize & word) << 16) |
                        ((0x00000000_0000FF00usize & word) << 8) |
                        (0x00000000_000000FFusize & word);
            let second = ((0xFF000000_00000000usize & word) >> 8) |
                         ((0x00FF0000_00000000usize & word) >> 16) |
                         ((0x0000FF00_00000000usize & word) >> 24) |
                         ((0x000000FF_00000000usize & word) >> 32);
            *dst = first;
            *(dst.offset(1)) = second;
            return true;
        }

        ascii_alu!(ascii_to_ascii, u8, u8, ascii_to_ascii_stride_64);
        ascii_alu!(ascii_to_basic_latin, u8, u16, ascii_to_basic_latin_stride_little_64);
        ascii_naive!(basic_latin_to_ascii, u16, u8);
    } else {
        ascii_naive!(ascii_to_ascii, u8, u8);
        ascii_naive!(ascii_to_basic_latin, u8, u16);
        ascii_naive!(basic_latin_to_ascii, u16, u8);
    }
}

// Any copyright to the test code below this comment is dedicated to the
// Public Domain. http://creativecommons.org/publicdomain/zero/1.0/

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_ascii {
        ($test_name:ident,
         $fn_tested:ident,
         $src_unit:ty,
         $dst_unit:ty) => (
        #[test]
        fn $test_name() {
            let mut src: Vec<$src_unit> = Vec::with_capacity(32);
            let mut dst: Vec<$dst_unit> = Vec::with_capacity(32);
            for i in 0..32 {
                src.clear();
                dst.clear();
                dst.resize(32, 0);
                for j in 0..32 {
                    let c = if i == j {
                        0xAA
                    } else {
                        j + 0x40
                    };
                    src.push(c as $src_unit);
                }
                match unsafe { $fn_tested(src.as_ptr(), dst.as_mut_ptr(), 32) } {
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
        });
    }

    test_ascii!(test_ascii_to_ascii, ascii_to_ascii, u8, u8);
    test_ascii!(test_ascii_to_basic_latin, ascii_to_basic_latin, u8, u16);
    test_ascii!(test_basic_latin_to_ascii, basic_latin_to_ascii, u16, u8);
}
