// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::mem;
use std::ops::Range;
use std::str;
use super::{Decoder, Encoder};

pub use std::io::Write as WriteBytes;
pub use std::io::Read as ReadBytes;
pub use std::fmt::Write as WriteUnicode;

pub trait ReadUnicode {
    fn read(&mut self, dst: &mut str) -> io::Result<usize>;
}

impl<'a, S: ReadUnicode> ReadUnicode for &'a mut S {
    fn read(&mut self, dst: &mut str) -> io::Result<usize> {
        (**self).read(dst)
    }
}

/// Similar to `std::io::Read for &'a [u8]`.
impl<'a> ReadUnicode for &'a str {
    fn read(&mut self, dst: &mut str) -> io::Result<usize> {
        // Unsafe: copy_maximum_utf8_prefix carefully preserves UTF-8 wel-formedness.
        let dst_bytes = unsafe { mem::transmute::<&mut str, &mut [u8]>(dst) };
        let (bytes_copied, remaining_self) = copy_maximum_utf8_prefix(*self, dst_bytes);
        *self = remaining_self;
        Ok(bytes_copied)
    }
}

fn copy_maximum_utf8_prefix<'a>(src: &'a str, dst: &mut [u8]) -> (usize, &'a str) {
    let src_bytes = src.as_bytes();
    let bytes_to_copy = if dst.len() >= src.len() {
        src.len()
    } else {
        let mut i = dst.len() - 1;
        while (src_bytes[i] & 0xC0) == 0x80 {
            i -= 1;
        }
        i
    };
    let (to_copy_from, remaining_src) = src.split_at(bytes_to_copy);
    let (to_copy_into, remaining_dst) = dst.split_at_mut(bytes_to_copy);
    to_copy_into.copy_from_slice(to_copy_from.as_bytes());
    for byte in remaining_dst {
        if (*byte & 0xC0) == 0x80 {
            *byte = 0
        } else {
            break
        }
    }
    (bytes_to_copy, remaining_src)
}


// FIXME: with vs without replacement

pub struct ReadDecoder<Stream: ReadBytes, Buffer: AsMut<[u8]>> {
    decoder: Decoder,
    stream: Stream,
    reached_stream_eof: bool,
    buffer: Buffer,
    unused_buffer_slice: Range<usize>,
}

impl<Stream: ReadBytes, Buffer: AsMut<[u8]>> ReadDecoder<Stream, Buffer> {
    pub fn new(decoder: Decoder, stream: Stream, buffer: Buffer) -> Self {
        ReadDecoder {
            decoder: decoder,
            stream: stream,
            reached_stream_eof: false,
            buffer: buffer,
            unused_buffer_slice: 0..0,
        }
    }
}

impl<Stream: ReadBytes, Buffer: AsMut<[u8]>> ReadUnicode for ReadDecoder<Stream, Buffer> {
    fn read(&mut self, dst: &mut str) -> io::Result<usize> {
        // Unsafe: Like Decoder::decode_to_utf8, <ReadDecoder as ReadBytes>::read’s contract
        // is to preserve UTF-8 well-formedness.
        let dst = unsafe { mem::transmute::<&mut str, &mut [u8]>(dst) };
        ReadBytes::read(self, dst)
    }
}

impl<Stream: ReadBytes, Buffer: AsMut<[u8]>> ReadBytes for ReadDecoder<Stream, Buffer> {
    fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
        let buffer = self.buffer.as_mut();
        if self.unused_buffer_slice.end > self.unused_buffer_slice.start {
            let buffer_slice = &buffer[self.unused_buffer_slice.clone()];
            // Unsafe: this slice was written by Decoder::decode_to_utf8,
            // whose contract is to make it well-formed UTF-8.
            let buffer_slice = unsafe { str::from_utf8_unchecked(buffer_slice) };
            let (bytes_copied, _) = copy_maximum_utf8_prefix(buffer_slice, dst);
            self.unused_buffer_slice.start += bytes_copied;
            Ok(bytes_copied)
        } else if !self.reached_stream_eof {
            let bytes_in_buffer = ReadBytes::read(&mut self.stream, buffer)?;
            if bytes_in_buffer == 0 {
                self.reached_stream_eof = true;
                let (_, _, written, _) = self.decoder.decode_to_utf8(b"", dst, true);
                // FIXME: deal with CoderResult::OutputFull here
                Ok(written)
            } else {
                let (_, bytes_read, bytes_written, _) = self.decoder.decode_to_utf8(
                    &buffer[..bytes_in_buffer], dst, false);
                self.unused_buffer_slice = bytes_read..bytes_in_buffer;
                Ok(bytes_written)
            }
        } else {
            Ok(0)
        }
    }
}

pub struct ReadEncoder<Stream: ReadUnicode, Buffer: AsMut<str>> {
    encoder: Encoder,
    stream: Stream,
    reached_stream_eof: bool,
    buffer: Buffer,
    unused_buffer_slice: Range<usize>,
}

impl<Stream: ReadUnicode, Buffer: AsMut<str>> ReadBytes for ReadEncoder<Stream, Buffer> {
    fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
        let buffer = self.buffer.as_mut();
        if self.unused_buffer_slice.end > self.unused_buffer_slice.start {
            let buffer_slice = &buffer[self.unused_buffer_slice.clone()];
            let bytes_copied = ReadBytes::read(&mut buffer_slice.as_bytes(), dst)?;
            self.unused_buffer_slice.start += bytes_copied;
            Ok(bytes_copied)
        } else if !self.reached_stream_eof {
            let bytes_in_buffer = ReadUnicode::read(&mut self.stream, buffer)?;
            if bytes_in_buffer == 0 {
                self.reached_stream_eof = true;
                let (_, _, written, _) = self.encoder.encode_from_utf8("", dst, true);
                // FIXME: deal with CoderResult::OutputFull here
                Ok(written)
            } else {
                let (_, bytes_read, bytes_written, _) = self.encoder.encode_from_utf8(
                    &buffer[..bytes_in_buffer], dst, false);
                self.unused_buffer_slice = bytes_read..bytes_in_buffer;
                Ok(bytes_written)
            }
        } else {
            Ok(0)
        }
    }
}
