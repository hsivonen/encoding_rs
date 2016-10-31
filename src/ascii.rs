// Copyright 2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

macro_rules! ascii_naive {
    ($name:ident,
     $src_unit:ty,
     $dst_unit:ty) => (
    #[inline(always)]
    pub unsafe fn $name(src: *const $src_unit, dst: *mut $dst_unit, len: usize) -> Option<($src_unit, usize)> {
        let src_slice = ::std::slice::from_raw_parts(src, len);
        let mut it = src_slice.iter().enumerate();
        loop {
            match it.next() {
                Some((i, code_unit_ref)) => {
                    let code_unit = *code_unit_ref;
                    if code_unit > 127 {
                        return Some((code_unit, i));
                    }
// Yes, manually omitting the bound check here matters
// a lot for perf.
                    *(dst.offset(i as isize)) = code_unit as $dst_unit;
                }
                None => {
                    return None;
                }
            }
        }
    });
}

cfg_if! {
// Compile out
    if #[cfg(all(target_pointer_width = "64", target_pointer_width = "32"))] {
//    if #[cfg(all(target_endian = "little", target_pointer_width = "64"))] {
// Aligned ALU word, little endian, 64-bit

        const STRIDE_SIZE: usize = 8;

        const ALIGNMENT_MASK: usize = 7;

        #[inline(always)]
        unsafe fn ascii_to_basic_latin_stride_little_64(src: *const usize, dst: *mut usize) -> bool {
            let word = *src;
// Check if the word contains non-ASCII
            if (word & 0x80808080_80808080usize) != 0 {
                return false;
            }
            let first = (0xFF000000_00000000usize & word)
            |          ((0x00FF0000_00000000usize & word) >> 8)
            |          ((0x0000FF00_00000000usize & word) >> 16)
            |          ((0x000000FF_00000000usize & word) >> 24);
            let second = ((0x00000000_FF000000usize & word) << 32)
            |           ((0x00000000_00FF0000usize & word) << 24)
            |           ((0x00000000_0000FF00usize & word) << 16)
            |           ((0x00000000_000000FFusize & word) << 8);
            *dst = first;
            *(dst.offset(1)) = second;
            return true;
        }

        #[inline(always)]
        pub unsafe fn ascii_to_basic_latin(src: *const u8, dst: *mut u16, len: usize) -> Option<(u8, usize)> {
            let mut offset = 0usize;
// XXX should we have more branchy code to move the pointers to
// alignment if they aren't aligned but could align after
// processing a few code units?
            if (STRIDE_SIZE <= len && ((src as usize) & ALIGNMENT_MASK) == 0) && (((dst as usize) & ALIGNMENT_MASK) == 0) {
// XXX stdlib's UTF-8 validation reads two words at a time
                loop {
                    if !ascii_to_basic_latin_stride_little_64(src.offset(offset as isize) as *const usize, dst.offset(offset as isize) as *mut usize) {
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
                *(dst.offset(offset as isize)) = code_unit as u16;
                offset += 1;
            }
            return None;
        }
        ascii_naive!(ascii_to_ascii, u8, u8);
        ascii_naive!(basic_latin_to_ascii, u16, u8);
    } else {
        ascii_naive!(ascii_to_ascii, u8, u8);
        ascii_naive!(ascii_to_basic_latin, u8, u16);
        ascii_naive!(basic_latin_to_ascii, u16, u8);
    }
}
