# encoding_rs

[![Build Status](https://travis-ci.org/hsivonen/encoding_rs.svg?branch=master)](https://travis-ci.org/hsivonen/encoding_rs)

encoding_rs aspires to become an implementation of the
[Encoding Standard](https://encoding.spec.whatwg.org/) that

1. Is written in Rust.
2. Is suitable for use in Gecko as a replacement of uconv. (I.e. supports
   decoding to UTF-16 and encoding from UTF-16.)
3. Is suitable for use in Rust code (both in Gecko and independently of Gecko).
   (I.e. supports decoding to UTF-8 and encoding from UTF-8 and provides an API
   compatible with at least the most common ways of using
   [rust-encoding](https://github.com/lifthrasiir/rust-encoding/).)

_You should not expect the above-stated aspiration to be fulfilled yet._

## Licensing

Please see the file named COPYRIGHT.

## API Documentation

Generated [API documentation](https://hsivonen.fi/rs/encoding_rs/) is available
online.

## Design

For design considerations, please see the associated [technical proposal to
rewrite uconv in Rust](https://docs.google.com/document/d/13GCbdvKi83a77ZcKOxaEteXp1SOGZ_9Fmztb9iX22v0/edit#).

## Performance goals

For decoding to UTF-16, the goal is to perform at least as well as Gecko's old
uconv. For decoding to UTF-8, the goal is to perform at least as well as
rust-encoding.

Encoding to UTF-8 should be fast. (UTF-8 to UTF-8 encode should be equivalent
to `memcpy` and UTF-16 to UTF-8 should be fast.)

Speed is a non-goal when encoding to legacy encodings. Encoding to legacy
encodings should not be optimized for speed at the expense of code size as long
as form submission and URL parsing in Gecko don't become noticeably too slow
in real-world use.

## Relationship with rust-encoding

This code is being prototyped as a new project as opposed to patches to
rust-encoding both to avoid breaking rust-encoding with in-progress exploration
and to be able to see where the API design would go from scratch given the
goals.

It is expected that encoding_rs will use code from rust-encoding.

Evaluation of whether it makes sense to propose portions of encoding_rs to be
adopted into rust-encoding will be best deferred until encoding_rs is further
along as a prototype.

## Roadmap

- [x] Design the low-level API.
- [x] Provide Rust-only convenience features (some BOM sniffing variants still
      TODO).
- [x] Provide an stl/gsl-flavored C++ API.
- [x] Implement all decoders and encoders.
- [ ] Add unit tests for all decoders and encoders.
- [ ] Finish BOM sniffing variants in Rust-only convenience features.
- [ ] Document the API.
- [ ] Publish the crate on crates.io.
- [ ] Create a solution for measuring performance.
- [ ] Test the performance impact of omitting duplicate bound checks using
      `unsafe`.
- [ ] Accelerate ASCII conversions using SSE2 on x86.
- [ ] Accelerate ASCII conversions using ALU register-sized operations on
      non-x86 architectures (process an `usize` instead of `u8` at a time).
- [ ] Use Björn Höhrmann's lookup table acceleration for UTF-8 as adapted to
      Rust in rust-encoding.
- [ ] Compress consecutive zeros in CJK indices.
- [ ] Make lookups by label or name use binary search that searches from the
      end of the label/name to the start.
- [ ] Provide an XPCOM/MFBT-flavored C++ API.
- [ ] Replace uconv with encoding_rs in Gecko.
- [ ] Implement the rust-encoding API in terms of encoding_rs.
- [ ] Investigate the use of NEON on newer ARM CPUs that have a lesser penalty
      on data flow from NEON to ALU registers.
