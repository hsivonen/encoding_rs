// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate encoding_rs;
extern crate safe_encoding_rs_mem;

fn check_utf8(data: &[u8]) {
    if let Err(_) = ::std::str::from_utf8(data) {
        panic!("Bogus UTF-8.");
    }
}

fn check_utf16(data: &[u16]) {
    let mut prev_was_high_surrogate = false;
    for unit in data {
        if *unit >= 0xD800 && *unit <= 0xDBFF {
            assert!(!prev_was_high_surrogate);
            prev_was_high_surrogate = true;
        } else if *unit >= 0xDC00 && *unit <= 0xDFFF {
            assert!(prev_was_high_surrogate);
            prev_was_high_surrogate = false;
        } else {
            assert!(!prev_was_high_surrogate);
            prev_was_high_surrogate = false;
        }
    }
    assert!(!prev_was_high_surrogate);
}

fn as_u16_slice(data: &[u8]) -> &[u16] {
    unsafe {
        let ptr = data.as_ptr();
        let len = data.len();
        if len < 2 {
            return ::std::slice::from_raw_parts(ptr as *const u16, 0);
        }
        let (adj_ptr, adj_len) = if ptr as usize & 1 == 0 {
            (ptr, len / 2)
        } else {
            (ptr.offset(1), (len - 1) / 2)
        };
        ::std::slice::from_raw_parts(adj_ptr as *const u16, adj_len)
    }
}

trait EigthOrSixteen: Clone {
    fn zero() -> Self;
}

impl EigthOrSixteen for u8 {
    fn zero() -> u8 {
        0
    }
}

impl EigthOrSixteen for u16 {
    fn zero() -> u16 {
        0
    }
}

fn vec_with_len<T: EigthOrSixteen>(len: usize) -> Vec<T> {
    let mut vec: Vec<T> = Vec::with_capacity(len);
    vec.resize(len, T::zero());
    vec
}

fn string_with_len(len: usize) -> String {
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        s.push('\u{0}');
    }
    s
}

fn fuzz_is_ascii(data: &[u8]) {
    assert_eq!(encoding_rs::mem::is_ascii(data), safe_encoding_rs_mem::is_ascii(data));
}

fn fuzz_is_basic_latin(data: &[u16]) {
    assert_eq!(encoding_rs::mem::is_basic_latin(data), safe_encoding_rs_mem::is_basic_latin(data));
}

fn fuzz_is_utf8_latin1(data: &[u8]) {
    assert_eq!(encoding_rs::mem::is_utf8_latin1(data), safe_encoding_rs_mem::is_utf8_latin1(data));
}

fn fuzz_is_str_latin1(data: &[u8]) {
    if let Ok(s) = std::str::from_utf8(data) {
        assert_eq!(encoding_rs::mem::is_str_latin1(s), safe_encoding_rs_mem::is_str_latin1(s));
    }
}

fn fuzz_is_utf16_latin1(data: &[u16]) {
    assert_eq!(encoding_rs::mem::is_utf16_latin1(data), safe_encoding_rs_mem::is_utf16_latin1(data));
}

fn fuzz_convert_utf8_to_utf16(data: &[u8]) {
    let needed = data.len() + 1;
    let mut dst = vec_with_len::<u16>(needed);
    let mut safe_dst = vec_with_len::<u16>(needed);
    let len = encoding_rs::mem::convert_utf8_to_utf16(data, &mut dst[..]);
    let safe_len = safe_encoding_rs_mem::convert_utf8_to_utf16(data, &mut safe_dst[..]);
    dst.truncate(len);
    safe_dst.truncate(safe_len);
    assert_eq!(len, safe_len);
    assert_eq!(dst, safe_dst);
    check_utf16(&dst[..]);
}

fn fuzz_convert_str_to_utf16(data: &[u8]) {
    if let Ok(s) = std::str::from_utf8(data) {
        let needed = s.len();
        let mut dst = vec_with_len::<u16>(needed);
        let mut safe_dst = vec_with_len::<u16>(needed);
        let len = encoding_rs::mem::convert_str_to_utf16(s, &mut dst[..]);
        let safe_len = safe_encoding_rs_mem::convert_str_to_utf16(s, &mut safe_dst[..]);
        dst.truncate(len);
        safe_dst.truncate(safe_len);
        assert_eq!(len, safe_len);
        assert_eq!(dst, safe_dst);
        check_utf16(&dst[..]);
    }
}

fn fuzz_convert_utf16_to_utf8(data: &[u16]) {
    let needed = data.len() * 3 + 1;
    let mut dst = vec_with_len::<u8>(needed);
    let mut safe_dst = vec_with_len::<u8>(needed);
    let len = encoding_rs::mem::convert_utf16_to_utf8(data, &mut dst[..]);
    let safe_len = safe_encoding_rs_mem::convert_utf16_to_utf8(data, &mut safe_dst[..]);
    dst.truncate(len);
    safe_dst.truncate(safe_len);
    assert_eq!(len, safe_len);
    assert_eq!(dst, safe_dst);
    check_utf8(&dst[..]);
}

fn fuzz_convert_utf16_to_str(data: &[u16]) {
    let needed = data.len() * 3 + 1;
    let mut dst = string_with_len(needed);
    let mut safe_dst = string_with_len(needed);
    let len = encoding_rs::mem::convert_utf16_to_str(data, &mut dst[..]);
    let safe_len = safe_encoding_rs_mem::convert_utf16_to_str(data, &mut safe_dst[..]);
    check_utf8(dst.as_bytes());
    check_utf8(safe_dst.as_bytes());
    dst.truncate(len);
    safe_dst.truncate(safe_len);
    assert_eq!(len, safe_len);
    assert_eq!(dst, safe_dst);
}

fn fuzz_convert_latin1_to_utf16(data: &[u8]) {
    let needed = data.len();
    let mut dst = vec_with_len::<u16>(needed);
    let mut safe_dst = vec_with_len::<u16>(needed);
    encoding_rs::mem::convert_latin1_to_utf16(data, &mut dst[..]);
    safe_encoding_rs_mem::convert_latin1_to_utf16(data, &mut safe_dst[..]);
    assert_eq!(dst, safe_dst);
    check_utf16(&dst[..]);
}

fn fuzz_convert_latin1_to_utf8(data: &[u8]) {
    let needed = data.len() * 2;
    let mut dst = vec_with_len::<u8>(needed);
    let mut safe_dst = vec_with_len::<u8>(needed);
    let len = encoding_rs::mem::convert_latin1_to_utf8(data, &mut dst[..]);
    let safe_len = safe_encoding_rs_mem::convert_latin1_to_utf8(data, &mut safe_dst[..]);
    dst.truncate(len);
    safe_dst.truncate(safe_len);
    assert_eq!(len, safe_len);
    assert_eq!(dst, safe_dst);
    check_utf8(&dst[..]);
}

fn fuzz_convert_latin1_to_str(data: &[u8]) {
    let needed = data.len() * 2;
    let mut dst = string_with_len(needed);
    let mut safe_dst = string_with_len(needed);
    let len = encoding_rs::mem::convert_latin1_to_str(data, &mut dst[..]);
    let safe_len = safe_encoding_rs_mem::convert_latin1_to_str(data, &mut safe_dst[..]);
    check_utf8(dst.as_bytes());
    check_utf8(safe_dst.as_bytes());
    dst.truncate(len);
    safe_dst.truncate(safe_len);
    assert_eq!(len, safe_len);
    assert_eq!(dst, safe_dst);
}

fn fuzz_convert_utf8_to_latin1_lossy(data: &[u8]) {
    let needed = data.len();
    let mut dst = vec_with_len::<u8>(needed);
    let mut safe_dst = vec_with_len::<u8>(needed);
    let len = encoding_rs::mem::convert_utf8_to_latin1_lossy(data, &mut dst[..]);
    let safe_len = safe_encoding_rs_mem::convert_utf8_to_latin1_lossy(data, &mut safe_dst[..]);
    if safe_encoding_rs_mem::is_utf8_latin1(data) {
        dst.truncate(len);
        safe_dst.truncate(safe_len);
        assert_eq!(len, safe_len);
        assert_eq!(dst, safe_dst);
    }
}

fn fuzz_convert_utf16_to_latin1_lossy(data: &[u16]) {
    let needed = data.len();
    let mut dst = vec_with_len::<u8>(needed);
    let mut safe_dst = vec_with_len::<u8>(needed);
    encoding_rs::mem::convert_utf16_to_latin1_lossy(data, &mut dst[..]);
    safe_encoding_rs_mem::convert_utf16_to_latin1_lossy(data, &mut safe_dst[..]);
    if safe_encoding_rs_mem::is_utf16_latin1(data) {
        assert_eq!(dst, safe_dst);
    }
}

fn fuzz_utf16_valid_up_to(data: &[u16]) {
    let up_to = encoding_rs::mem::utf16_valid_up_to(data);
    let safe_up_to = safe_encoding_rs_mem::utf16_valid_up_to(data);
    assert_eq!(up_to, safe_up_to);
}

fn fuzz_ensure_utf16_validity(data: &[u16]) {
    let needed = data.len();
    let mut dst = vec_with_len::<u16>(needed);
    let mut safe_dst = vec_with_len::<u16>(needed);
    dst.copy_from_slice(data);
    safe_dst.copy_from_slice(data);
    encoding_rs::mem::ensure_utf16_validity(&mut dst[..]);
    safe_encoding_rs_mem::ensure_utf16_validity(&mut safe_dst[..]);
    assert_eq!(dst, safe_dst);
    check_utf16(&dst[..]);
}

fn fuzz_copy_ascii_to_ascii(data: &[u8]) {
    let needed = data.len();
    let mut dst = vec_with_len::<u8>(needed);
    let mut safe_dst = vec_with_len::<u8>(needed);
    let len = encoding_rs::mem::copy_ascii_to_ascii(data, &mut dst[..]);
    let safe_len = safe_encoding_rs_mem::copy_ascii_to_ascii(data, &mut safe_dst[..]);
    dst.truncate(len);
    safe_dst.truncate(safe_len);
    assert_eq!(len, safe_len);
    assert_eq!(dst, safe_dst);
}

fn fuzz_copy_ascii_to_basic_latin(data: &[u8]) {
    let needed = data.len();
    let mut dst = vec_with_len::<u16>(needed);
    let mut safe_dst = vec_with_len::<u16>(needed);
    let len = encoding_rs::mem::copy_ascii_to_basic_latin(data, &mut dst[..]);
    let safe_len = safe_encoding_rs_mem::copy_ascii_to_basic_latin(data, &mut safe_dst[..]);
    dst.truncate(len);
    safe_dst.truncate(safe_len);
    assert_eq!(len, safe_len);
    assert_eq!(dst, safe_dst);
}

fn fuzz_is_utf8_bidi(data: &[u8]) {
    assert_eq!(encoding_rs::mem::is_utf8_bidi(data), safe_encoding_rs_mem::is_utf8_bidi(data));
}

fn fuzz_copy_basic_latin_to_ascii(data: &[u16]) {
    let needed = data.len();
    let mut dst = vec_with_len::<u8>(needed);
    let mut safe_dst = vec_with_len::<u8>(needed);
    let len = encoding_rs::mem::copy_basic_latin_to_ascii(data, &mut dst[..]);
    let safe_len = safe_encoding_rs_mem::copy_basic_latin_to_ascii(data, &mut safe_dst[..]);
    dst.truncate(len);
    safe_dst.truncate(safe_len);
    assert_eq!(len, safe_len);
    assert_eq!(dst, safe_dst);
}

fuzz_target!(
    |data: &[u8]| {
        if let Some(first) = data.first() {
            match *first {
                0 => fuzz_is_ascii(&data[1..]),
                1 => fuzz_is_basic_latin(as_u16_slice(&data[1..])),
                2 => fuzz_is_utf8_latin1(&data[1..]),
                3 => fuzz_is_str_latin1(&data[1..]),
                4 => fuzz_is_utf16_latin1(as_u16_slice(&data[1..])),
                5 => fuzz_convert_utf8_to_utf16(&data[1..]),
                6 => fuzz_convert_str_to_utf16(&data[1..]),
                7 => fuzz_convert_utf16_to_utf8(as_u16_slice(&data[1..])),
                8 => fuzz_convert_utf16_to_str(as_u16_slice(&data[1..])),
                9 => fuzz_convert_latin1_to_utf16(&data[1..]),
                10 => fuzz_convert_latin1_to_utf8(&data[1..]),
                11 => fuzz_convert_latin1_to_str(&data[1..]),
                12 => fuzz_convert_utf8_to_latin1_lossy(&data[1..]),
                13 => fuzz_convert_utf16_to_latin1_lossy(as_u16_slice(&data[1..])),
                14 => fuzz_utf16_valid_up_to(as_u16_slice(&data[1..])),
                15 => fuzz_ensure_utf16_validity(as_u16_slice(&data[1..])),
                16 => fuzz_copy_ascii_to_ascii(&data[1..]),
                17 => fuzz_copy_ascii_to_basic_latin(&data[1..]),
                18 => fuzz_copy_basic_latin_to_ascii(as_u16_slice(&data[1..])),
                19 => fuzz_is_utf8_bidi(&data[1..]),
                _ => return,
            }
        }
        // Comment to make rustfmt not introduce a compilation error
    }
);
