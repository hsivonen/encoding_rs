If you send a pull request / patch, please observe the following.

## Licensing

Since this crate is dual-licensed,
[section 5 of the Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0#contributions)
is considered to apply in the sense of Contributions being automatically
under the Apache License 2.0 or MIT dual license (see the `COPYRIGHT` file).
That is, by the act of offering a Contribution, place your Contribution under
the Apache License 2.0 or MIT dual license stated in the `COPYRIGHT` file.
Please do not contribute if you aren't willing or allowed to license your
contributions in this manner.

## Copyright Notices

If you require the addition of your copyright notice, it's up to you to edit in
your notice as part of your Contribution. Not adding a copyright notice is
taken as a waiver of copyright notice.

## No Encodings Beyond The Encoding Standard

Please do not contribute implementations of encodings that are not specified
in the [Encoding Standard](https://encoding.spec.whatwg.org/).

For example, an implementation of UTF-7 would be explicitly not welcome.

## Compatibility with Stable Rust

Please ensure that your Contribution compiles with the latest stable-channel
rustc.

## rustfmt

Please install [`rustfmt`](https://github.com/rust-lang-nursery/rustfmt) and
run `cargo fmt` before creating a pull request. (It's OK for `cargo fmt` to
exit with an error due to too long lines.)

## Unit tests

Please ensure that `cargo test` succeeds.
