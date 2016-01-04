# encoding-rs

encoding-rs aspires to become an implementation of the
[Encoding Standard](https://encoding.spec.whatwg.org/) that

1. Is written in Rust.
2. Is suitable for use in Gecko as a replacement of uconv. (I.e. supports
   decoding to UTF-16 and encoding from UTF-16.)
3. Is suitable for use in Rust code (both in Gecko and independently of Gecko).
   (I.e. supports decoding to UTF-8 and encoding from UTF-8 and provides an API
   compatible with at least the most common ways of using
   [rust-encoding](https://github.com/lifthrasiir/rust-encoding/).)

_This project is not in a usable or useful stage yet! You should not expect
the above-stated aspiration to be fulfilled yet and you shouldn't expect the
code here to work at all._

## Licensing

Please see the file named COPYRIGHT.

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

It is expected that encoding-rs will use code from rust-encoding.

Evaluation of whether it makes sense to propose portions of encoding-rs to be
adopted into rust-encoding will be best deferred until encoding-rs is further
along as a prototype.
