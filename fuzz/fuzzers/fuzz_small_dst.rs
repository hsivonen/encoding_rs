// Copyright Mozilla Foundation. See the COPYRIGHT
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

// Only encodings whose decoders can carry state across calls.
static ENCODINGS: [&'static Encoding; 10] = [&UTF_8_INIT,
                                             &GBK_INIT,
                                             &BIG5_INIT,
                                             &EUC_JP_INIT,
                                             &GB18030_INIT,
                                             &UTF_16BE_INIT,
                                             &UTF_16LE_INIT,
                                             &SHIFT_JIS_INIT,
                                             &EUC_KR_INIT,
                                             &ISO_2022_JP_INIT];

// Decodes in alternating chunks into an output buffer of the documented
// minimum size (4 bytes) plus `extra` and compares with a one-shot decode.
fn decode_to_utf8(encoding: &'static Encoding,
                  extra: usize,
                  chunks: [usize; 2],
                  bytes: &[u8],
                  expected: &str) {
    let mut decoder = encoding.new_decoder_without_bom_handling();
    let mut dst = vec![0u8; 4 + extra];
    let mut out = Vec::new();
    let mut offset = 0;
    let mut phase = false;
    loop {
        phase = !phase;
        let end = ::std::cmp::min(offset + chunks[phase as usize], bytes.len());
        let last = end == bytes.len();
        let mut src = &bytes[offset..end];
        loop {
            let (result, read, written, _) = decoder.decode_to_utf8(src, &mut dst, last);
            out.extend_from_slice(&dst[..written]);
            src = &src[read..];
            if result == CoderResult::InputEmpty {
                break;
            }
        }
        offset = end;
        if last {
            break;
        }
    }
    assert_eq!(::std::str::from_utf8(&out).unwrap(), expected);
}

// Same as above, with the documented minimum of two UTF-16 code units.
fn decode_to_utf16(encoding: &'static Encoding,
                   extra: usize,
                   chunks: [usize; 2],
                   bytes: &[u8],
                   expected: &str) {
    let mut decoder = encoding.new_decoder_without_bom_handling();
    let mut dst = vec![0u16; 2 + extra];
    let mut out = Vec::new();
    let mut offset = 0;
    let mut phase = false;
    loop {
        phase = !phase;
        let end = ::std::cmp::min(offset + chunks[phase as usize], bytes.len());
        let last = end == bytes.len();
        let mut src = &bytes[offset..end];
        loop {
            let (result, read, written, _) = decoder.decode_to_utf16(src, &mut dst, last);
            out.extend_from_slice(&dst[..written]);
            src = &src[read..];
            if result == CoderResult::InputEmpty {
                break;
            }
        }
        offset = end;
        if last {
            break;
        }
    }
    assert_eq!(String::from_utf16(&out).unwrap(), expected);
}

fuzz_target!(
    |data: &[u8]| {
        if data.len() < 4 {
            return;
        }
        let index = data[0] as usize;
        if index >= ENCODINGS.len() {
            return;
        }
        let encoding = ENCODINGS[index];
        let extra = (data[1] & 0x7) as usize;
        let chunks = [(data[2] & 0x7) as usize + 1, (data[3] & 0x7) as usize + 1];
        let bytes = &data[4..];
        let (expected, _) = encoding.decode_without_bom_handling(bytes);
        decode_to_utf8(encoding, extra, chunks, bytes, &expected);
        decode_to_utf16(encoding, extra, chunks, bytes, &expected);
        // Comment to make rustfmt not introduce a compilation error
    }
);
