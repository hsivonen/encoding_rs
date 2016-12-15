// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// The initial revision of this file was extracted from the "UTF-8 validation"
// section of the file src/libcore/str/mod.rs from Rust project at revision
// 7ad7232422f7e5bbfa0e52dabe36c12677df19e2. The Utf8Error struct also comes
// from that file.

/// Errors which can occur when attempting to interpret a sequence of `u8`
/// as a string.
///
/// As such, the `from_utf8` family of functions and methods for both `String`s
/// and `&str`s make use of this error, for example.
#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub struct Utf8Error {
    valid_up_to: usize,
}

impl Utf8Error {
    /// Returns the index in the given string up to which valid UTF-8 was
    /// verified.
    ///
    /// It is the maximum index such that `from_utf8(input[..index])`
    /// would return `Ok(_)`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::str;
    ///
    /// // some invalid bytes, in a vector
    /// let sparkle_heart = vec![0, 159, 146, 150];
    ///
    /// // std::str::from_utf8 returns a Utf8Error
    /// let error = str::from_utf8(&sparkle_heart).unwrap_err();
    ///
    /// // the second byte is invalid here
    /// assert_eq!(1, error.valid_up_to());
    /// ```
    pub fn valid_up_to(&self) -> usize {
        self.valid_up_to
    }
}

// use truncation to fit u64 into usize
const NONASCII_MASK: usize = 0x80808080_80808080u64 as usize;

/// Return `true` if any byte in the word `x` is nonascii (>= 128).
#[inline]
fn contains_nonascii(x: usize) -> bool {
    (x & NONASCII_MASK) != 0
}

/// Walk through `iter` checking that it's a valid UTF-8 sequence,
/// returning `true` in that case, or, if it is invalid, `false` with
/// `iter` reset such that it is pointing at the first byte in the
/// invalid sequence.
#[inline(always)]
pub fn run_utf8_validation(v: &[u8]) -> Result<(), Utf8Error> {
    let mut offset = 0;
    let len = v.len();
    while offset < len {
        let old_offset = offset;
        macro_rules! err { () => {{
            return Err(Utf8Error {
                valid_up_to: old_offset
            })
        }}}

        macro_rules! next { () => {{
            offset += 1;
            // we needed data, but there was none: error!
            if offset >= len {
                err!()
            }
            v[offset]
        }}}

        let first = v[offset];
        if first >= 128 {
            let w = UTF8_CHAR_WIDTH[first as usize];
            let second = next!();
            // 2-byte encoding is for codepoints  \u{0080} to  \u{07ff}
            //        first  C2 80        last DF BF
            // 3-byte encoding is for codepoints  \u{0800} to  \u{ffff}
            //        first  E0 A0 80     last EF BF BF
            //   excluding surrogates codepoints  \u{d800} to  \u{dfff}
            //               ED A0 80 to       ED BF BF
            // 4-byte encoding is for codepoints \u{1000}0 to \u{10ff}ff
            //        first  F0 90 80 80  last F4 8F BF BF
            //
            // Use the UTF-8 syntax from the RFC
            //
            // https://tools.ietf.org/html/rfc3629
            // UTF8-1      = %x00-7F
            // UTF8-2      = %xC2-DF UTF8-tail
            // UTF8-3      = %xE0 %xA0-BF UTF8-tail / %xE1-EC 2( UTF8-tail ) /
            //               %xED %x80-9F UTF8-tail / %xEE-EF 2( UTF8-tail )
            // UTF8-4      = %xF0 %x90-BF 2( UTF8-tail ) / %xF1-F3 3( UTF8-tail ) /
            //               %xF4 %x80-8F 2( UTF8-tail )
            match w {
                2 => {
                    if second & !CONT_MASK != TAG_CONT_U8 {
                        err!()
                    }
                }
                3 => {
                    match (first, second, next!() & !CONT_MASK) {
                        (0xE0, 0xA0...0xBF, TAG_CONT_U8) |
                        (0xE1...0xEC, 0x80...0xBF, TAG_CONT_U8) |
                        (0xED, 0x80...0x9F, TAG_CONT_U8) |
                        (0xEE...0xEF, 0x80...0xBF, TAG_CONT_U8) => {}
                        _ => err!(),
                    }
                }
                4 => {
                    match (first, second, next!() & !CONT_MASK, next!() & !CONT_MASK) {
                        (0xF0, 0x90...0xBF, TAG_CONT_U8, TAG_CONT_U8) |
                        (0xF1...0xF3, 0x80...0xBF, TAG_CONT_U8, TAG_CONT_U8) |
                        (0xF4, 0x80...0x8F, TAG_CONT_U8, TAG_CONT_U8) => {}
                        _ => err!(),
                    }
                }
                _ => err!(),
            }
            offset += 1;
        } else {
            // Ascii case, try to skip forward quickly.
            // When the pointer is aligned, read 2 words of data per iteration
            // until we find a word containing a non-ascii byte.
            let usize_bytes = ::std::mem::size_of::<usize>();
            let bytes_per_iteration = 2 * usize_bytes;
            let ptr = v.as_ptr();
            let align = (ptr as usize + offset) & (usize_bytes - 1);
            if align == 0 {
                if len >= bytes_per_iteration {
                    while offset <= len - bytes_per_iteration {
                        unsafe {
                            let u = *(ptr.offset(offset as isize) as *const usize);
                            let v = *(ptr.offset((offset + usize_bytes) as isize) as *const usize);

                            // break if there is a nonascii byte
                            let zu = contains_nonascii(u);
                            let zv = contains_nonascii(v);
                            if zu || zv {
                                break;
                            }
                        }
                        offset += bytes_per_iteration;
                    }
                }
                // step from the point where the wordwise loop stopped
                while offset < len && v[offset] < 128 {
                    offset += 1;
                }
            } else {
                offset += 1;
            }
        }
    }

    Ok(())
}

// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                                     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 /* 0x1F */, 1, 1, 1, 1,
                                     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                                     1, 1, 1, 1, 1, 1, 1, 1 /* 0x3F */, 1, 1, 1, 1, 1, 1, 1, 1,
                                     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                                     1, 1, 1, 1 /* 0x5F */, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                                     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                                     1 /* 0x7F */, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                     0 /* 0x9F */, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                     0 /* 0xBF */, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                                     2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                                     2 /* 0xDF */, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                                     3 /* 0xEF */, 4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                     0 /* 0xFF */];

/// Mask of the value bits of a continuation byte
const CONT_MASK: u8 = 0b0011_1111;
/// Value of the tag bits (tag mask is !CONT_MASK) of a continuation byte
const TAG_CONT_U8: u8 = 0b1000_0000;
