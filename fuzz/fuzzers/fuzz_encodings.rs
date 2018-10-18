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

use encoding_rs::*;

// Doesn't included ISO-8859-8-I.
static ENCODINGS: [&'static Encoding; 39] = [&UTF_8_INIT,
                                             &REPLACEMENT_INIT,
                                             &GBK_INIT,
                                             &BIG5_INIT,
                                             &EUC_JP_INIT,
                                             &GB18030_INIT,
                                             &UTF_16BE_INIT,
                                             &UTF_16LE_INIT,
                                             &SHIFT_JIS_INIT,
                                             &EUC_KR_INIT,
                                             &ISO_2022_JP_INIT,
                                             &X_USER_DEFINED_INIT,
                                             &WINDOWS_1250_INIT,
                                             &WINDOWS_1251_INIT,
                                             &WINDOWS_1252_INIT,
                                             &WINDOWS_1253_INIT,
                                             &WINDOWS_1254_INIT,
                                             &WINDOWS_1255_INIT,
                                             &WINDOWS_1256_INIT,
                                             &WINDOWS_1257_INIT,
                                             &WINDOWS_1258_INIT,
                                             &KOI8_U_INIT,
                                             &MACINTOSH_INIT,
                                             &IBM866_INIT,
                                             &KOI8_R_INIT,
                                             &ISO_8859_2_INIT,
                                             &ISO_8859_3_INIT,
                                             &ISO_8859_4_INIT,
                                             &ISO_8859_5_INIT,
                                             &ISO_8859_6_INIT,
                                             &ISO_8859_7_INIT,
                                             &ISO_8859_10_INIT,
                                             &ISO_8859_13_INIT,
                                             &ISO_8859_14_INIT,
                                             &WINDOWS_874_INIT,
                                             &ISO_8859_15_INIT,
                                             &ISO_8859_16_INIT,
                                             &ISO_8859_8_I_INIT,
                                             &X_MAC_CYRILLIC_INIT];

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
            (ptr.add(1), (len - 1) / 2)
        };
        ::std::slice::from_raw_parts(adj_ptr as *const u16, adj_len)
    }
}

fn decode(encoding: &'static Encoding, data: &[u8]) {
    let (cow, _, _) = encoding.decode(data);
    check_utf8(cow.as_bytes());
}

fn decode_with_bom_removal(encoding: &'static Encoding, data: &[u8]) {
    let (cow, _) = encoding.decode_with_bom_removal(data);
    check_utf8(cow.as_bytes());
}

fn decode_without_bom_handling(encoding: &'static Encoding, data: &[u8]) {
    let (cow, _) = encoding.decode_without_bom_handling(data);
    check_utf8(cow.as_bytes());
}

fn decode_without_bom_handling_and_without_replacement(encoding: &'static Encoding, data: &[u8]) {
    if let Some(cow) = encoding.decode_without_bom_handling_and_without_replacement(data) {
        check_utf8(cow.as_bytes());
    }
}

fn encode(encoding: &'static Encoding, data: &[u8]) {
    if let Ok(s) = ::std::str::from_utf8(data) {
        let (_, _, _) = encoding.encode(s);
    }
}

fn encode_from_utf8(encoding: &'static Encoding, data: &[u8]) {
    if data.len() < 2 {
        return;
    }
    let first = data[0] as usize;
    let second = data[1] as usize;
    if first == 0 || second == 0 || first >= data.len() - 2 || second >= data.len() - 2 {
        return;
    }
    if let Ok(s) = ::std::str::from_utf8(&data[2..]) {
        let mut encoder = encoding.new_encoder();
        let mut cs = s.chars();
        let mut string = String::new();
        let mut dst = Vec::new();
        let mut phase = false;
        loop {
            phase = !phase;
            let chunk_size = if phase { first } else { second };
            string.clear();
            for _ in 0..chunk_size {
                if let Some(c) = cs.next() {
                    string.push(c);
                } else {
                    let mut total_read = 0;
                    loop {
                        if let Some(needed) = encoder
                               .max_buffer_length_from_utf8_if_no_unmappables(
                            string.len() - total_read,
                        ) {
                            dst.resize(needed, 0);
                            let (result, read, _, _) =
                                encoder.encode_from_utf8(&string[total_read..], &mut dst, true);
                            total_read += read;
                            if result == CoderResult::InputEmpty {
                                break;
                            }
                        }
                    }
                    return;
                }
            }
            let mut total_read = 0;
            loop {
                if let Some(needed) = encoder.max_buffer_length_from_utf8_if_no_unmappables(
                    string.len() - total_read,
                ) {
                    dst.resize(needed, 0);
                    let (result, read, _, _) =
                        encoder.encode_from_utf8(&string[total_read..], &mut dst, false);
                    total_read += read;
                    if result == CoderResult::InputEmpty {
                        break;
                    }
                } else {
                    return;
                }
            }
        }
    }
}

fn encode_from_utf8_without_replacement(encoding: &'static Encoding, data: &[u8]) {
    if data.len() < 2 {
        return;
    }
    let first = data[0] as usize;
    let second = data[1] as usize;
    if first == 0 || second == 0 || first >= data.len() - 2 || second >= data.len() - 2 {
        return;
    }
    if let Ok(s) = ::std::str::from_utf8(&data[2..]) {
        let mut encoder = encoding.new_encoder();
        let mut cs = s.chars();
        let mut string = String::new();
        let mut dst = Vec::new();
        let mut phase = false;
        loop {
            phase = !phase;
            let chunk_size = if phase { first } else { second };
            string.clear();
            for _ in 0..chunk_size {
                if let Some(c) = cs.next() {
                    string.push(c);
                } else {
                    if let Some(needed) =
                        encoder.max_buffer_length_from_utf8_without_replacement(string.len()) {
                        dst.resize(needed, 0);
                        let (result, _, _) =
                            encoder.encode_from_utf8_without_replacement(&string, &mut dst, true);
                        assert_ne!(result, EncoderResult::OutputFull);
                    }
                    return;
                }
            }
            if let Some(needed) =
                encoder.max_buffer_length_from_utf8_without_replacement(string.len()) {
                dst.resize(needed, 0);
                let (result, _, _) =
                    encoder.encode_from_utf8_without_replacement(&string, &mut dst, false);
                match result {
                    EncoderResult::InputEmpty => {}
                    EncoderResult::OutputFull => unreachable!("Bogus max size math"),
                    EncoderResult::Unmappable(_) => return,
                }
            } else {
                return;
            }
        }
    }
}

fn encode_from_utf16(encoding: &'static Encoding, data: &[u8]) {
    if data.len() < 2 {
        return;
    }
    let first = data[0] as usize;
    let second = data[1] as usize;
    if first == 0 || second == 0 || first >= data.len() - 2 || second >= data.len() - 2 {
        return;
    }
    let s = as_u16_slice(&data[2..]);
    let mut encoder = encoding.new_encoder();
    let mut offset = 0;
    let mut dst = Vec::new();
    let mut phase = false;
    loop {
        phase = !phase;
        let mut chunk_size = if phase { first } else { second };
        let mut last = false;
        if offset + chunk_size >= s.len() {
            last = true;
            chunk_size = s.len() - offset;
        }
        let new_offset = offset + chunk_size;
        let chunk = &s[offset..new_offset];
        offset = new_offset;
        let mut total_read = 0;
        loop {
            if let Some(needed) =
                encoder.max_buffer_length_from_utf16_if_no_unmappables(chunk.len() - total_read) {
                dst.resize(needed, 0);
                let (result, read, _, _) =
                    encoder.encode_from_utf16(&chunk[total_read..], &mut dst, last);
                total_read += read;
                if result == CoderResult::InputEmpty {
                    if last {
                        return;
                    }
                    break;
                }
            }
        }
    }
}

fn encode_from_utf16_without_replacement(encoding: &'static Encoding, data: &[u8]) {
    if data.len() < 2 {
        return;
    }
    let first = data[0] as usize;
    let second = data[1] as usize;
    if first == 0 || second == 0 || first >= data.len() - 2 || second >= data.len() - 2 {
        return;
    }
    let s = as_u16_slice(&data[2..]);
    let mut encoder = encoding.new_encoder();
    let mut offset = 0;
    let mut dst = Vec::new();
    let mut phase = false;
    loop {
        phase = !phase;
        let mut chunk_size = if phase { first } else { second };
        let mut last = false;
        if offset + chunk_size >= s.len() {
            last = true;
            chunk_size = s.len() - offset;
        }
        let new_offset = offset + chunk_size;
        let chunk = &s[offset..new_offset];
        offset = new_offset;
        if let Some(needed) = encoder
               .max_buffer_length_from_utf16_without_replacement(chunk.len()) {
            dst.resize(needed, 0);
            let (result, _, _) = encoder
                .encode_from_utf16_without_replacement(&chunk, &mut dst, last);
            match result {
                EncoderResult::InputEmpty => {
                    if last {
                        return;
                    }
                }
                EncoderResult::OutputFull => unreachable!("Bogus max size math"),
                EncoderResult::Unmappable(_) => return,
            }
        }
    }
}

fn decode_to_utf16(encoding: &'static Encoding, data: &[u8]) {
    if data.len() < 3 {
        return;
    }
    let first = data[0] as usize;
    let second = data[1] as usize;
    if first == 0 || second == 0 || first >= data.len() - 3 || second >= data.len() - 3 {
        return;
    }
    let mut decoder = match data[2] {
        0 => encoding.new_decoder(),
        1 => encoding.new_decoder_with_bom_removal(),
        2 => encoding.new_decoder_without_bom_handling(),
        _ => return,
    };
    let s = &data[3..];
    let mut offset = 0;
    let mut dst = Vec::new();
    let mut phase = false;
    loop {
        phase = !phase;
        let mut chunk_size = if phase { first } else { second };
        let mut last = false;
        if offset + chunk_size >= s.len() {
            last = true;
            chunk_size = s.len() - offset;
        }
        let new_offset = offset + chunk_size;
        let chunk = &s[offset..new_offset];
        offset = new_offset;
        if let Some(needed) = decoder.max_utf16_buffer_length(chunk.len()) {
            dst.resize(needed, 0);
            let (result, read, written, _) = decoder.decode_to_utf16(&chunk, &mut dst, last);
            assert!(read <= chunk.len());
            check_utf16(&dst[..written]);
            assert_ne!(result, CoderResult::OutputFull);
            if last {
                return;
            }
        }
    }
}

fn decode_to_utf16_without_replacement(encoding: &'static Encoding, data: &[u8]) {
    if data.len() < 3 {
        return;
    }
    let first = data[0] as usize;
    let second = data[1] as usize;
    if first == 0 || second == 0 || first >= data.len() - 3 || second >= data.len() - 3 {
        return;
    }
    let mut decoder = match data[2] {
        0 => encoding.new_decoder(),
        1 => encoding.new_decoder_with_bom_removal(),
        2 => encoding.new_decoder_without_bom_handling(),
        _ => return,
    };
    let s = &data[3..];
    let mut offset = 0;
    let mut dst = Vec::new();
    let mut phase = false;
    loop {
        phase = !phase;
        let mut chunk_size = if phase { first } else { second };
        let mut last = false;
        if offset + chunk_size >= s.len() {
            last = true;
            chunk_size = s.len() - offset;
        }
        let new_offset = offset + chunk_size;
        let chunk = &s[offset..new_offset];
        offset = new_offset;
        if let Some(needed) = decoder.max_utf16_buffer_length(chunk.len()) {
            dst.resize(needed, 0);
            let (result, read, written) =
                decoder.decode_to_utf16_without_replacement(&chunk, &mut dst, last);
            assert!(read <= chunk.len());
            check_utf16(&dst[..written]);
            match result {
                DecoderResult::InputEmpty => {
                    if last {
                        return;
                    }
                }
                DecoderResult::OutputFull => unreachable!("Bogus max size math"),
                DecoderResult::Malformed(_, _) => return,
            }
        }
    }
}

fn decode_to_utf8(encoding: &'static Encoding, data: &[u8]) {
    if data.len() < 3 {
        return;
    }
    let first = data[0] as usize;
    let second = data[1] as usize;
    if first == 0 || second == 0 || first >= data.len() - 3 || second >= data.len() - 3 {
        return;
    }
    let mut decoder = match data[2] {
        0 => encoding.new_decoder(),
        1 => encoding.new_decoder_with_bom_removal(),
        2 => encoding.new_decoder_without_bom_handling(),
        _ => return,
    };
    let s = &data[3..];
    let mut offset = 0;
    let mut dst = Vec::new();
    let mut phase = false;
    loop {
        phase = !phase;
        let mut chunk_size = if phase { first } else { second };
        let mut last = false;
        if offset + chunk_size >= s.len() {
            last = true;
            chunk_size = s.len() - offset;
        }
        let new_offset = offset + chunk_size;
        let chunk = &s[offset..new_offset];
        offset = new_offset;
        if let Some(needed) = decoder.max_utf8_buffer_length(chunk.len()) {
            dst.resize(needed, 0);
            let (result, read, written, _) = decoder.decode_to_utf8(&chunk, &mut dst, last);
            assert!(read <= chunk.len());
            check_utf8(&dst[..written]);
            assert_ne!(result, CoderResult::OutputFull);
            if last {
                return;
            }
        }
    }
}

fn decode_to_utf8_without_replacement(encoding: &'static Encoding, data: &[u8]) {
    if data.len() < 3 {
        return;
    }
    let first = data[0] as usize;
    let second = data[1] as usize;
    if first == 0 || second == 0 || first >= data.len() - 3 || second >= data.len() - 3 {
        return;
    }
    let mut decoder = match data[2] {
        0 => encoding.new_decoder(),
        1 => encoding.new_decoder_with_bom_removal(),
        2 => encoding.new_decoder_without_bom_handling(),
        _ => return,
    };
    let s = &data[3..];
    let mut offset = 0;
    let mut dst = Vec::new();
    let mut phase = false;
    loop {
        phase = !phase;
        let mut chunk_size = if phase { first } else { second };
        let mut last = false;
        if offset + chunk_size >= s.len() {
            last = true;
            chunk_size = s.len() - offset;
        }
        let new_offset = offset + chunk_size;
        let chunk = &s[offset..new_offset];
        offset = new_offset;
        if let Some(needed) = decoder.max_utf8_buffer_length_without_replacement(chunk.len()) {
            dst.resize(needed, 0);
            let (result, read, written) =
                decoder.decode_to_utf8_without_replacement(&chunk, &mut dst, last);
            assert!(read <= chunk.len());
            check_utf8(&dst[..written]);
            match result {
                DecoderResult::InputEmpty => {
                    if last {
                        return;
                    }
                }
                DecoderResult::OutputFull => unreachable!("Bogus max size math"),
                DecoderResult::Malformed(_, _) => return,
            }
        }
    }
}

fn dispatch_test(encoding: &'static Encoding, data: &[u8]) {
    if let Some(first) = data.first() {
        match *first {
            0 => decode(encoding, &data[1..]),
            1 => decode_with_bom_removal(encoding, &data[1..]),
            2 => decode_without_bom_handling(encoding, &data[1..]),
            3 => decode_without_bom_handling_and_without_replacement(encoding, &data[1..]),
            4 => encode(encoding, &data[1..]),
            5 => encode_from_utf8(encoding, &data[1..]),
            6 => encode_from_utf8_without_replacement(encoding, &data[1..]),
            7 => encode_from_utf16(encoding, &data[1..]),
            8 => encode_from_utf16_without_replacement(encoding, &data[1..]),
            9 => decode_to_utf16(encoding, &data[1..]),
            10 => decode_to_utf16_without_replacement(encoding, &data[1..]),
            11 => decode_to_utf8(encoding, &data[1..]),
            12 => decode_to_utf8_without_replacement(encoding, &data[1..]),
            _ => return,
        }
    }
}

fuzz_target!(
    |data: &[u8]| {
        if let Some(first) = data.first() {
            let index = *first as usize;
            if index >= ENCODINGS.len() {
                return;
            }
            let encoding = ENCODINGS[index];
            dispatch_test(encoding, &data[1..]);
        }
        // Comment to make rustfmt not introduce a compilation error
    }
);
