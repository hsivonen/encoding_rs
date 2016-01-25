// Copyright 2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::*;

pub fn decode_to_utf16(encoding: &'static Encoding, bytes: &[u8], expect: &[u16]) {
    let mut decoder = encoding.new_decoder();
    let mut dest: Vec<u16> = Vec::with_capacity(decoder.max_utf16_buffer_length(expect.len()));
    let capacity = dest.capacity();
    dest.resize(capacity, 0u16);
    let (complete, read, written, _) = decoder.decode_to_utf16_with_replacement(bytes,
                                                                                &mut dest,
                                                                                true);
    assert_eq!(complete, WithReplacementResult::InputEmpty);
    assert_eq!(read, bytes.len());
    assert_eq!(written, expect.len());
    dest.truncate(written);
    assert_eq!(&dest[..], expect);
}

pub fn decode_to_utf8(encoding: &'static Encoding, bytes: &[u8], expect: &str) {
    let mut decoder = encoding.new_decoder();
    let mut dest: Vec<u8> =
        Vec::with_capacity(decoder.max_utf8_buffer_length_with_replacement(expect.len()));
    let capacity = dest.capacity();
    dest.resize(capacity, 0u8);
    let (complete, read, written, _) = decoder.decode_to_utf8_with_replacement(bytes,
                                                                               &mut dest,
                                                                               true);
    assert_eq!(complete, WithReplacementResult::InputEmpty);
    assert_eq!(read, bytes.len());
    assert_eq!(written, expect.len());
    dest.truncate(written);
    assert_eq!(&dest[..], expect.as_bytes());
}
