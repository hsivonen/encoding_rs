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
use super::Decoder;

pub use std::io::Write as WriteBytes;
pub use std::io::Read as ReadBytes;
pub use std::fmt::Write as WriteUnicode;

pub trait ReadUnicode {
    fn read(&mut self, buf: &mut str) -> io::Result<usize>;
}

impl<'a, S: ReadUnicode> ReadUnicode for &'a mut S {
    fn read(&mut self, buf: &mut str) -> io::Result<usize> {
        (**self).read(buf)
    }
}

/// Similar to `std::io::Read for &'a [u8]`.
impl<'a> ReadUnicode for &'a str {
    fn read(&mut self, buf: &mut str) -> io::Result<usize> {
        let self_bytes = self.as_bytes();
        let bytes_to_copy = if buf.len() >= self.len() {
            self.len()
        } else {
            let mut i = buf.len() - 1;
            while (self_bytes[i] & 0xC0) == 0x80 {
                i -= 1;
            }
            i
        };
        let buf_bytes = unsafe { mem::transmute::<&mut str, &mut [u8]>(buf) };
        let (to_copy_from, remaining_self) = self.split_at(bytes_to_copy);
        let (to_copy_into, remaining_buf) = buf_bytes.split_at_mut(bytes_to_copy);
        to_copy_into.copy_from_slice(to_copy_from.as_bytes());
        for byte in remaining_buf {
            if (*byte & 0xC0) == 0x80 {
                *byte = 0
            } else {
                break
            }
        }
        *self = remaining_self;
        Ok(bytes_to_copy)
    }
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
        let dst = unsafe { mem::transmute::<&mut str, &mut [u8]>(dst) };
        ReadBytes::read(self, dst)
    }
}

impl<Stream: ReadBytes, Buffer: AsMut<[u8]>> ReadBytes for ReadDecoder<Stream, Buffer> {
    fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
        let buffer = self.buffer.as_mut();
        if self.unused_buffer_slice.end > self.unused_buffer_slice.start {
            let buffer_slice = &buffer[self.unused_buffer_slice.clone()];
            let mut buffer_slice = unsafe { str::from_utf8_unchecked(buffer_slice) };

            // This transmute is a lie: `dst` might not be UTF-8 at all.
            // But we know that `<&str as ReadUnicode>::read` doesn’t rely on it being UTF-8.
            let dst = unsafe { mem::transmute::<&mut [u8], &mut str>(dst) };

            let bytes_copied = ReadUnicode::read(&mut buffer_slice, dst)?;
            self.unused_buffer_slice.start += bytes_copied;
            Ok(bytes_copied)
        } else if !self.reached_stream_eof {
            let bytes_in_buffer = ReadBytes::read(&mut self.stream, buffer)?;
            if bytes_in_buffer == 0 {
                self.reached_stream_eof = true;
                let (_, _, written, _) = self.decoder.decode_to_utf8(b"", dst, true);
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
