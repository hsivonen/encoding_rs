// Copyright 2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Result of a (potentially partial) ASCII acceleration operation.
#[derive(Debug)]
pub enum AsciiResult<T> {
    /// Everything was ASCII and the buffers were of the same length or the
    /// source was shorter.
    InputEmpty,

    /// Everything was ASCII and the destination was shorter.
    OutputFull,

    /// Non-ASCII was encountered. The wrapped `T` is the non-ASCII code unit.
    NonAscii(T),
}

macro_rules! ascii_function {
    ($name:ident,
     $src_unit:ty,
     $dst_unit:ty,
     $impl_name:ident) => (
    /// Copy ASCII from `src` to `dst`.
    ///
    /// Returns an `AsciiResult` and the number of ASCII code units copied.
    #[inline(always)]
    pub fn $name(src: &[$src_unit], dst: &mut [$dst_unit]) -> (AsciiResult<$src_unit>, usize) {
        let (pending, length) = if dst.len() < src.len() {
            (AsciiResult::OutputFull, dst.len())
        } else {
            (AsciiResult::InputEmpty, src.len())
        };
        match unsafe {$impl_name(src.as_ptr(), dst.as_mut_ptr(), length)} {
            None => (pending, length),
            Some((non_ascii, consumed)) => (AsciiResult::NonAscii(non_ascii), consumed)
        }
    });
}

ascii_function!(ascii_to_ascii, u8, u8, ascii_to_ascii_impl);
ascii_function!(ascii_to_basic_latin, u8, u16, ascii_to_basic_latin_impl);
ascii_function!(basic_latin_to_ascii, u16, u8, basic_latin_to_ascii_impl);

macro_rules! ascii_naive_impl {
    ($name:ident,
     $src_unit:ty,
     $dst_unit:ty) => (
    #[inline(always)]
    pub unsafe fn $name(src: *const $src_unit, dst: *mut $dst_unit, len: usize) -> Option<($src_unit, usize)> {
        let src_slice = ::std::slice::from_raw_parts(src, len);
        let mut dst_slice = ::std::slice::from_raw_parts_mut(dst, len);
        let mut it = src_slice.iter().enumerate();
        loop {
            match it.next() {
                Some((i, code_unit_ref)) => {
                    let code_unit = *code_unit_ref;
                    if code_unit > 127 {
                        return Some((code_unit, i));
                    }
                    *(dst_slice.as_mut_ptr().offset(i as isize)) = code_unit as $dst_unit;
                }
                None => {
                    return None;
                }
            }
        }
    });
}

ascii_naive_impl!(ascii_to_ascii_impl, u8, u8);
ascii_naive_impl!(ascii_to_basic_latin_impl, u8, u16);
ascii_naive_impl!(basic_latin_to_ascii_impl, u16, u8);
