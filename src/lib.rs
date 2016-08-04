// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! encoding_rs is a Gecko-oriented Free Software / Open Source implementation
//! of the [Encoding Standard](https://encoding.spec.whatwg.org/) in Rust.
//! Gecko-oriented means that converting to and from UTF-16 is supported in
//! addition to converting to and from UTF-8, that the performance and
//! streamability goals are browser-oriented and that FFI-friendliness is a
//! goal.
//!
//! ## Availability
//!
//! The code is available under the
//! [Apache license, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
//! or the [MIT license](https://opensource.org/licenses/MIT), at your option.
//! See the
//! [`COPYRIGHT`](https://github.com/hsivonen/encoding_rs/blob/master/COPYRIGHT)
//! file for details.
//! The [repository is on GitHub](https://github.com/hsivonen/encoding_rs). The
//! [crate is available on crates.io](https://crates.io/crates/encoding_rs).
//!
//! ## Web / Browser Focus
//!
//! Both in terms of scope and performance, the focus is on the Web. For scope,
//! this means that encoding_rs implements the Encoding Standard fully and
//! doesn't implement encodings that are not specified in the Encoding
//! Standard. For performance, this means that decoding performance is
//! important as well as performance for encoding into UTF-8 or encoding the
//! Basic Latin range (ASCII) into legacy encodings. Non-Basic Latin needs to
//! be encoded into legacy encodings in only two places in the Web platform: in
//! the query part of URLs, in which case it's a matter of relatively rare
//! error handling, and in form submission, in which case the user action and
//! networking tend to hide the performance of the encoder.
//!
//! Deemphasizing performance of encoding non-Basic Latin text into legacy
//! encodings enables smaller code size thanks to the encoder side using the
//! decode-optimized data tables without having encode-optimized data tables at
//! all. Even in decoders, smaller lookup table size is preferred over avoiding
//! multiplication operations.
//!
//! Additionally, performance is a non-goal for the ASCII-incompatible
//! ISO-2022-JP and UTF-16 encodings, which are rarely used on the Web. For
//! clarity, this means that performance is a non-goal for UTF-16 as used on
//! the wire as an interchange encoding (UTF-16 on the `[u8]` side of the API).
//! Good performance for UTF-16 used as an in-RAM Unicode representation
//! (UTF-16 the `[u16]` side of the API) is a goal.
//!
//! Despite the focus on the Web, encoding_rs may well be useful for decoding
//! email, although you'll need to implement UTF-7 decoding and label handling
//! by other means. (Due to the Web focus, patches to add UTF-7 are unwelcome
//! in encoding_rs itself.) Also, despite the browser focus, the hope is that
//! non-browser applications that wish to consume Web content or submit Web
//! forms in a Web-compatible way will find encoding_rs useful.
//!
//! ## Streaming & Non-Streaming; Rust & C/C++
//!
//! The API in Rust has two modes of operation: streaming and non-streaming.
//! The streaming API is the foundation of the implementation and should be
//! used when processing data that arrives piecemeal from an i/o stream. The
//! streaming API has an FFI wrapper that exposes it to C callers. The
//! non-streaming part of the API is for Rust callers only and is implemented
//! on top of the streaming API and, as such, could be considered as merely a
//! set of convenience methods. There is no analogous C API exposed via FFI,
//! mainly because C doesn't have standard types for growable byte buffers and
//! Unicode strings that know their length.
//!
//! The C API (header file generated at `target/include/encoding_rs.h` when
//! building encoding_rs) can, in turn, be wrapped for use from C++. Such a
//! C++ wrapper could re-create the non-streaming API in C++ for C++ callers.
//! Currently, encoding_rs comes with a
//! [C++ wrapper](https://github.com/hsivonen/encoding_rs/blob/master/include/encoding_rs_cpp.h)
//! that uses STL+[GSL](https://github.com/Microsoft/GSL/) types, but this
//! wrapper doesn't provide non-streaming convenience methods at this time. A
//! C++ wrapper with XPCOM/MFBT types is planned but does not exist yet.
//!
//! The `Encoding` type is common to both the streaming and non-streaming
//! modes. In the streaming mode, decoding operations are performed with a
//! `Decoder` and encoding operations with an `Encoder` object obtained via
//! `Encoding`. In the non-streaming mode, decoding and encoding operations are
//! performed using methods on `Encoding` objects themselves, so the `Decoder`
//! and `Encoder` objects are not used at all.
//!
//! ## Memory management
//!
//! The non-streaming mode never performs heap allocations (even the methods
//! that write into a `Vec<u8>` or a `String` by taking them as arguments do
//! not reallocate the backing buffer of the `Vec<u8>` or the `String`. That
//! is, the non-streaming mode uses caller-allocated buffers exclusively.
//!
//! The methods of the streaming mode that return a `Vec<u8>` or a `String`
//! perform heap allocations but only to allocate the backing buffer of the
//! `Vec<u8>` or the `String`.
//!
//! `Encoding` is always statically allocated. `Decoder` and `Encoder` need no
//! `Drop` cleanup.
//!
//! ## Buffer reading and writing behavior
//!
//! Based on experience gathered with the `java.nio.charset` encoding converter
//! API and with the Gecko uconv encoding converter API, the buffer reading
//! and writing behaviors of encoding_rs are asymmetric: input buffers are
//! fully drained but output buffers are not always fully filled.
//!
//! When reading from an input buffer, encoding_rs always consumes all input
//! up to the next error or to the end of the buffer. In particular, when
//! decoding, even if the input buffer ends in the middle of a byte sequence
//! for a character, the decoder consumes all input. This has the benefit that
//! the caller of the API can always fill the next buffer from the start from
//! whatever source the bytes come from and never has to first copy the last
//! bytes of the previous buffer to the start of the next buffer. However, when
//! encoding, the UTF-8 input buffers have to end at a character boundary, which
//! is a requirement for the Rust `str` type anyway, and UTF-16 input buffer
//! boundaries falling in the middle of a surrogate pair result in both
//! suggorates being treated individually as unpaired surrogates.
//!
//! Additionally, decoders guarantee that they can be fed even one byte at a
//! time and encoders guarantee that they can be fed even one code point at a
//! time. This has the benefit of not placing restrictions on the size of
//! chunks the content arrives e.g. from network.
//!
//! When writing into an output buffer, encoding_rs makes sure that the code
//! unit sequence for a character is never split across output buffer
//! boundaries. This may result in wasted space at the end of an output buffer,
//! but the advantages are that the output side of both decoders and encoders
//! is greatly simplified compared to designs that attempt to fill output
//! buffers exactly even when that entails splitting a code unit sequence and
//! when encoding_rs methods return to the caller, the output produces thus
//! far is always valid taken as whole. (In the case of encoding to ISO-2022-JP,
//! the output needs to be considered as a whole, because the latest output
//! buffer taken alone might not be valid taken alone if the transition away
//! from the ASCII state occurred in an earlier output buffer. However, since
//! the ISO-2022-JP decoder doesn't treat streams that don't end in the ASCII
//! state as being in error despite the encoder generating a transition to the
//! ASCII state at the end, the claim about the partial output taken as a whole
//! being valid is true even for ISO-2022-JP.)
//!
//! ## Mapping Spec Concepts onto the API
//!
//! <table>
//! <thead>
//! <tr><th>Spec Concept</th><th>Streaming</th><th>Non-Streaming</th></tr>
//! </thead>
//! <tbody>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#encoding">encoding</a></td><td><code>&amp;'static Encoding</code></td><td><code>&amp;'static Encoding</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#utf-8">UTF-8 encoding</a></td><td><code>UTF_8</code></td><td><code>UTF_8</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#concept-encoding-get">get an encoding</a></td><td><code>Encoding::for_label(<var>label</var>)</code></td><td><code>Encoding::for_label(<var>label</var>)</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#name">name</a></td><td><code><var>encoding</var>.name()</code></td><td><code><var>encoding</var>.name()</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#get-an-output-encoding">get an output encoding</a></td><td><code><var>encoding</var>.output_encoding()</code></td><td><code><var>encoding</var>.output_encoding()</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#decode">decode</a></td><td><code>let d = <var>encoding</var>.new_decoder();<br>let res = d.decode_to_<var>*</var>(<var>src</var>, <var>dst</var>, false);<br>// &hellip;</br>let last_res = d.decode_to_<var>*</var>(<var>src</var>, <var>dst</var>, true);</code></td><td><code><var>encoding</var>.decode(<var>src</var>)</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#utf-8-decode">UTF-8 decode</a></td><td><code>let d = UTF_8.new_decoder_with_bom_removal();<br>let res = d.decode_to_<var>*</var>(<var>src</var>, <var>dst</var>, false);<br>// &hellip;</br>let last_res = d.decode_to_<var>*</var>(<var>src</var>, <var>dst</var>, true);</code></td><td><code>UTF_8.decode_with_bom_removal(<var>src</var>)</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#utf-8-decode-without-bom">UTF-8 decode without BOM</a></td><td><code>let d = UTF_8.new_decoder_without_bom_handling();<br>let res = d.decode_to_<var>*</var>(<var>src</var>, <var>dst</var>, false);<br>// &hellip;</br>let last_res = d.decode_to_<var>*</var>(<var>src</var>, <var>dst</var>, true);</code></td><td><code>UTF_8.decode_without_bom_handling(<var>src</var>)</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#utf-8-decode-without-bom-or-fail">UTF-8 decode without BOM or fail</a></td><td><code>let d = UTF_8.new_decoder_without_bom_handling();<br>let res = d.decode_to_<var>*</var>_without_replacement(<var>src</var>, <var>dst</var>, false);<br>// &hellip; (fail if malformed)</br>let last_res = d.decode_to_<var>*</var>_without_replacement(<var>src</var>, <var>dst</var>, true);<br>// (fail if malformed)</code></td><td><code>UTF_8.decode_without_bom_handling_and_without_replacement(<var>src</var>)</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#encode">encode</a></td><td><code>let e = <var>encoding</var>.new_encoder();<br>let res = e.encode_to_<var>*</var>(<var>src</var>, <var>dst</var>, false);<br>// &hellip;</br>let last_res = e.encode_to_<var>*</var>(<var>src</var>, <var>dst</var>, true);</code></td><td><code><var>encoding</var>.encode(<var>src</var>)</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#utf-8-encode">UTF-8 encode</a></td><td>Use the UTF-8 nature of Rust strings directly:<br><code><var>write</var>(<var>src</var>.as_bytes());<br>// refill src<br><var>write</var>(<var>src</var>.as_bytes());<br>// refill src<br><var>write</var>(<var>src</var>.as_bytes());<br>// &hellip;</code></td><td>Use the UTF-8 nature of Rust strings directly:<br><code><var>src</var>.as_bytes()</code></td></tr>
//! </tbody>
//! </table>
//!
//! ## Migrating from rust-encoding
//!
//! The crate
//! [encoding_rs_compat](https://github.com/hsivonen/encoding_rs_compat/)
//! is a drop-in replacement for rust-encoding 0.2.32 that implements (most of)
//! the API of rust-encoding 0.2.32 on top of encoding_rs.
//!
//! The following table provides a mapping from rust-encoding constructs to
//! encoding_rs ones.
//!
//! <table>
//! <thead>
//! <tr><th>rust-encoding</th><th>encoding_rs</th></tr>
//! </thead>
//! <tbody>
//! <tr><td><code>encoding::EncodingRef</code></td><td><code>&amp;'static encoding_rs::Encoding</code></td></tr>
//! <tr><td><code>encoding::all::<var>WINDOWS_31J</var></code> (not based on the WHATWG name for some encodings)</td><td><code>encoding_rs::<var>SHIFT_JIS</var></code> (always the WHATWG name uppercased and hyphens replaced with underscores)</td></tr>
//! <tr><td><code>encoding::all::ERROR</code></td><td>Not available because not in the Encoding Standard</td></tr>
//! <tr><td><code>encoding::all::ASCII</code></td><td>Not available because not in the Encoding Standard</td></tr>
//! <tr><td><code>encoding::all::ISO_8859_1</code></td><td>Not available because not in the Encoding Standard</td></tr>
//! <tr><td><code>encoding::all::HZ</code></td><td>Not available because not in the Encoding Standard</td></tr>
//! <tr><td><code>encoding::label::encoding_from_whatwg_label(<var>string</var>)</code></td><td><code>encoding_rs::Encoding::for_label(<var>string</var>)</code></td></tr>
//! <tr><td><code>encoding::all::<var>WINDOWS_31J</var>.whatwg_name()</code> (always lower case)</td><td><code>encoding_rs::<var>SHIFT_JIS</var>.name()</code> (potentially mixed case)</td></tr>
//! <tr><td><code>encoding::all::<var>WINDOWS_31J</var>.name()</code></td><td>Not available because not in the Encoding Standard</td></tr>
//! <tr><td><code>encoding::decode(<var>bytes</var>, encoding::DecoderTrap::Replace, encoding::all::<var>WINDOWS_31J</var>)</code></td><td><code>encoding_rs::<var>SHIFT_JIS</var>.decode(<var>bytes</var>)</code></td></tr>
//! <tr><td><code>encoding::all::<var>WINDOWS_31J</var>.decode(<var>bytes</var>, encoding::DecoderTrap::Replace)</code></td><td><code>encoding_rs::<var>SHIFT_JIS</var>.decode_without_bom_handling(<var>bytes</var>)</code></td></tr>
//! <tr><td><code>encoding::all::<var>WINDOWS_31J</var>.encode(<var>string</var>, encoding::EncoderTrap::Replace)</code></td><td><code>encoding_rs::<var>SHIFT_JIS</var>.encode(<var>string</var>)</code></td></tr>
//! <tr><td><code>encoding::all::<var>WINDOWS_31J</var>.raw_decoder()</code></td><td><code>encoding_rs::<var>SHIFT_JIS</var>.new_decoder_without_bom_handling()</code></td></tr>
//! <tr><td><code>encoding::all::<var>WINDOWS_31J</var>.raw_encoder()</code></td><td><code>encoding_rs::<var>SHIFT_JIS</var>.new_encoder()</code></td></tr>
//! <tr><td><code>encoding::RawDecoder</code></td><td><code>encoding_rs::Decoder</code></td></tr>
//! <tr><td><code>encoding::RawEncoder</code></td><td><code>encoding_rs::Encoder</code></td></tr>
//! <tr><td><code><var>raw_decoder</var>.raw_feed(<var>src</var>, <var>dst</var>)</code></td><td><code><var>decoder</var>.decode_to_utf8_without_replacement(<var>src</var>, <var>dst</var>, false)</code></td></tr>
//! <tr><td><code><var>raw_decoder</var>.raw_finish(<var>dst</var>)</code></td><td><code><var>decoder</var>.decode_to_utf8_without_replacement(b"", <var>dst</var>, true)</code></td></tr>
//! <tr><td><code><var>raw_encoder</var>.raw_feed(<var>src</var>, <var>dst</var>)</code></td><td><code><var>encoder</var>.encode_from_utf8_without_replacement(<var>src</var>, <var>dst</var>, false)</code></td></tr>
//! <tr><td><code><var>raw_encoder</var>.raw_finish(<var>dst</var>)</code></td><td><code><var>encoder</var>.encode_from_utf8_without_replacement("", <var>dst</var>, true)</code></td></tr>
//! </tbody>
//! </table>

#[macro_use]
mod macros;

#[cfg(test)]
mod testing;

mod single_byte;
mod utf_8;
mod gb18030;
mod big5;
mod euc_jp;
mod iso_2022_jp;
mod shift_jis;
mod euc_kr;
mod replacement;
mod x_user_defined;
mod utf_16;

mod handles;
mod data;
mod variant;
pub mod ffi;

use variant::*;
pub use ffi::*;

const NCR_EXTRA: usize = 9; // #1114111;

// BEGIN GENERATED CODE. PLEASE DO NOT EDIT.
// Instead, please regenerate using generate-encoding-data.py

const LONGEST_LABEL_LENGTH: usize = 19; // cseucpkdfmtjapanese

const LONGEST_NAME_LENGTH: usize = 14; // x-mac-cyrillic

/// The Big5 encoding.
pub const BIG5: &'static Encoding = &Encoding {
    name: "Big5",
    variant: VariantEncoding::Big5,
};

/// The EUC-JP encoding.
pub const EUC_JP: &'static Encoding = &Encoding {
    name: "EUC-JP",
    variant: VariantEncoding::EucJp,
};

/// The EUC-KR encoding.
pub const EUC_KR: &'static Encoding = &Encoding {
    name: "EUC-KR",
    variant: VariantEncoding::EucKr,
};

/// The GBK encoding.
pub const GBK: &'static Encoding = &Encoding {
    name: "GBK",
    variant: VariantEncoding::Gbk,
};

/// The IBM866 encoding.
pub const IBM866: &'static Encoding = &Encoding {
    name: "IBM866",
    variant: VariantEncoding::SingleByte(data::IBM866_DATA),
};

/// The ISO-2022-JP encoding.
pub const ISO_2022_JP: &'static Encoding = &Encoding {
    name: "ISO-2022-JP",
    variant: VariantEncoding::Iso2022Jp,
};

/// The ISO-8859-10 encoding.
pub const ISO_8859_10: &'static Encoding = &Encoding {
    name: "ISO-8859-10",
    variant: VariantEncoding::SingleByte(data::ISO_8859_10_DATA),
};

/// The ISO-8859-13 encoding.
pub const ISO_8859_13: &'static Encoding = &Encoding {
    name: "ISO-8859-13",
    variant: VariantEncoding::SingleByte(data::ISO_8859_13_DATA),
};

/// The ISO-8859-14 encoding.
pub const ISO_8859_14: &'static Encoding = &Encoding {
    name: "ISO-8859-14",
    variant: VariantEncoding::SingleByte(data::ISO_8859_14_DATA),
};

/// The ISO-8859-15 encoding.
pub const ISO_8859_15: &'static Encoding = &Encoding {
    name: "ISO-8859-15",
    variant: VariantEncoding::SingleByte(data::ISO_8859_15_DATA),
};

/// The ISO-8859-16 encoding.
pub const ISO_8859_16: &'static Encoding = &Encoding {
    name: "ISO-8859-16",
    variant: VariantEncoding::SingleByte(data::ISO_8859_16_DATA),
};

/// The ISO-8859-2 encoding.
pub const ISO_8859_2: &'static Encoding = &Encoding {
    name: "ISO-8859-2",
    variant: VariantEncoding::SingleByte(data::ISO_8859_2_DATA),
};

/// The ISO-8859-3 encoding.
pub const ISO_8859_3: &'static Encoding = &Encoding {
    name: "ISO-8859-3",
    variant: VariantEncoding::SingleByte(data::ISO_8859_3_DATA),
};

/// The ISO-8859-4 encoding.
pub const ISO_8859_4: &'static Encoding = &Encoding {
    name: "ISO-8859-4",
    variant: VariantEncoding::SingleByte(data::ISO_8859_4_DATA),
};

/// The ISO-8859-5 encoding.
pub const ISO_8859_5: &'static Encoding = &Encoding {
    name: "ISO-8859-5",
    variant: VariantEncoding::SingleByte(data::ISO_8859_5_DATA),
};

/// The ISO-8859-6 encoding.
pub const ISO_8859_6: &'static Encoding = &Encoding {
    name: "ISO-8859-6",
    variant: VariantEncoding::SingleByte(data::ISO_8859_6_DATA),
};

/// The ISO-8859-7 encoding.
pub const ISO_8859_7: &'static Encoding = &Encoding {
    name: "ISO-8859-7",
    variant: VariantEncoding::SingleByte(data::ISO_8859_7_DATA),
};

/// The ISO-8859-8 encoding.
pub const ISO_8859_8: &'static Encoding = &Encoding {
    name: "ISO-8859-8",
    variant: VariantEncoding::SingleByte(data::ISO_8859_8_DATA),
};

/// The ISO-8859-8-I encoding.
pub const ISO_8859_8_I: &'static Encoding = &Encoding {
    name: "ISO-8859-8-I",
    variant: VariantEncoding::SingleByte(data::ISO_8859_8_DATA),
};

/// The KOI8-R encoding.
pub const KOI8_R: &'static Encoding = &Encoding {
    name: "KOI8-R",
    variant: VariantEncoding::SingleByte(data::KOI8_R_DATA),
};

/// The KOI8-U encoding.
pub const KOI8_U: &'static Encoding = &Encoding {
    name: "KOI8-U",
    variant: VariantEncoding::SingleByte(data::KOI8_U_DATA),
};

/// The Shift_JIS encoding.
pub const SHIFT_JIS: &'static Encoding = &Encoding {
    name: "Shift_JIS",
    variant: VariantEncoding::ShiftJis,
};

/// The UTF-16BE encoding.
pub const UTF_16BE: &'static Encoding = &Encoding {
    name: "UTF-16BE",
    variant: VariantEncoding::Utf16Be,
};

/// The UTF-16LE encoding.
pub const UTF_16LE: &'static Encoding = &Encoding {
    name: "UTF-16LE",
    variant: VariantEncoding::Utf16Le,
};

/// The UTF-8 encoding.
pub const UTF_8: &'static Encoding = &Encoding {
    name: "UTF-8",
    variant: VariantEncoding::Utf8,
};

/// The gb18030 encoding.
pub const GB18030: &'static Encoding = &Encoding {
    name: "gb18030",
    variant: VariantEncoding::Gb18030,
};

/// The macintosh encoding.
pub const MACINTOSH: &'static Encoding = &Encoding {
    name: "macintosh",
    variant: VariantEncoding::SingleByte(data::MACINTOSH_DATA),
};

/// The replacement encoding.
pub const REPLACEMENT: &'static Encoding = &Encoding {
    name: "replacement",
    variant: VariantEncoding::Replacement,
};

/// The windows-1250 encoding.
pub const WINDOWS_1250: &'static Encoding = &Encoding {
    name: "windows-1250",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1250_DATA),
};

/// The windows-1251 encoding.
pub const WINDOWS_1251: &'static Encoding = &Encoding {
    name: "windows-1251",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1251_DATA),
};

/// The windows-1252 encoding.
pub const WINDOWS_1252: &'static Encoding = &Encoding {
    name: "windows-1252",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1252_DATA),
};

/// The windows-1253 encoding.
pub const WINDOWS_1253: &'static Encoding = &Encoding {
    name: "windows-1253",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1253_DATA),
};

/// The windows-1254 encoding.
pub const WINDOWS_1254: &'static Encoding = &Encoding {
    name: "windows-1254",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1254_DATA),
};

/// The windows-1255 encoding.
pub const WINDOWS_1255: &'static Encoding = &Encoding {
    name: "windows-1255",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1255_DATA),
};

/// The windows-1256 encoding.
pub const WINDOWS_1256: &'static Encoding = &Encoding {
    name: "windows-1256",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1256_DATA),
};

/// The windows-1257 encoding.
pub const WINDOWS_1257: &'static Encoding = &Encoding {
    name: "windows-1257",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1257_DATA),
};

/// The windows-1258 encoding.
pub const WINDOWS_1258: &'static Encoding = &Encoding {
    name: "windows-1258",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1258_DATA),
};

/// The windows-874 encoding.
pub const WINDOWS_874: &'static Encoding = &Encoding {
    name: "windows-874",
    variant: VariantEncoding::SingleByte(data::WINDOWS_874_DATA),
};

/// The x-mac-cyrillic encoding.
pub const X_MAC_CYRILLIC: &'static Encoding = &Encoding {
    name: "x-mac-cyrillic",
    variant: VariantEncoding::SingleByte(data::X_MAC_CYRILLIC_DATA),
};

/// The x-user-defined encoding.
pub const X_USER_DEFINED: &'static Encoding = &Encoding {
    name: "x-user-defined",
    variant: VariantEncoding::UserDefined,
};

static ENCODINGS_SORTED_BY_NAME: [&'static Encoding; 40] = [BIG5,
                                                            EUC_JP,
                                                            EUC_KR,
                                                            GBK,
                                                            IBM866,
                                                            ISO_2022_JP,
                                                            ISO_8859_10,
                                                            ISO_8859_13,
                                                            ISO_8859_14,
                                                            ISO_8859_15,
                                                            ISO_8859_16,
                                                            ISO_8859_2,
                                                            ISO_8859_3,
                                                            ISO_8859_4,
                                                            ISO_8859_5,
                                                            ISO_8859_6,
                                                            ISO_8859_7,
                                                            ISO_8859_8,
                                                            ISO_8859_8_I,
                                                            KOI8_R,
                                                            KOI8_U,
                                                            SHIFT_JIS,
                                                            UTF_16BE,
                                                            UTF_16LE,
                                                            UTF_8,
                                                            GB18030,
                                                            MACINTOSH,
                                                            REPLACEMENT,
                                                            WINDOWS_1250,
                                                            WINDOWS_1251,
                                                            WINDOWS_1252,
                                                            WINDOWS_1253,
                                                            WINDOWS_1254,
                                                            WINDOWS_1255,
                                                            WINDOWS_1256,
                                                            WINDOWS_1257,
                                                            WINDOWS_1258,
                                                            WINDOWS_874,
                                                            X_MAC_CYRILLIC,
                                                            X_USER_DEFINED];

static LABELS_SORTED: [&'static str; 218] = ["866",
                                             "ansi_x3.4-1968",
                                             "arabic",
                                             "ascii",
                                             "asmo-708",
                                             "big5",
                                             "big5-hkscs",
                                             "chinese",
                                             "cn-big5",
                                             "cp1250",
                                             "cp1251",
                                             "cp1252",
                                             "cp1253",
                                             "cp1254",
                                             "cp1255",
                                             "cp1256",
                                             "cp1257",
                                             "cp1258",
                                             "cp819",
                                             "cp866",
                                             "csbig5",
                                             "cseuckr",
                                             "cseucpkdfmtjapanese",
                                             "csgb2312",
                                             "csibm866",
                                             "csiso2022jp",
                                             "csiso2022kr",
                                             "csiso58gb231280",
                                             "csiso88596e",
                                             "csiso88596i",
                                             "csiso88598e",
                                             "csiso88598i",
                                             "csisolatin1",
                                             "csisolatin2",
                                             "csisolatin3",
                                             "csisolatin4",
                                             "csisolatin5",
                                             "csisolatin6",
                                             "csisolatin9",
                                             "csisolatinarabic",
                                             "csisolatincyrillic",
                                             "csisolatingreek",
                                             "csisolatinhebrew",
                                             "cskoi8r",
                                             "csksc56011987",
                                             "csmacintosh",
                                             "csshiftjis",
                                             "cyrillic",
                                             "dos-874",
                                             "ecma-114",
                                             "ecma-118",
                                             "elot_928",
                                             "euc-jp",
                                             "euc-kr",
                                             "gb18030",
                                             "gb2312",
                                             "gb_2312",
                                             "gb_2312-80",
                                             "gbk",
                                             "greek",
                                             "greek8",
                                             "hebrew",
                                             "hz-gb-2312",
                                             "ibm819",
                                             "ibm866",
                                             "iso-2022-cn",
                                             "iso-2022-cn-ext",
                                             "iso-2022-jp",
                                             "iso-2022-kr",
                                             "iso-8859-1",
                                             "iso-8859-10",
                                             "iso-8859-11",
                                             "iso-8859-13",
                                             "iso-8859-14",
                                             "iso-8859-15",
                                             "iso-8859-16",
                                             "iso-8859-2",
                                             "iso-8859-3",
                                             "iso-8859-4",
                                             "iso-8859-5",
                                             "iso-8859-6",
                                             "iso-8859-6-e",
                                             "iso-8859-6-i",
                                             "iso-8859-7",
                                             "iso-8859-8",
                                             "iso-8859-8-e",
                                             "iso-8859-8-i",
                                             "iso-8859-9",
                                             "iso-ir-100",
                                             "iso-ir-101",
                                             "iso-ir-109",
                                             "iso-ir-110",
                                             "iso-ir-126",
                                             "iso-ir-127",
                                             "iso-ir-138",
                                             "iso-ir-144",
                                             "iso-ir-148",
                                             "iso-ir-149",
                                             "iso-ir-157",
                                             "iso-ir-58",
                                             "iso8859-1",
                                             "iso8859-10",
                                             "iso8859-11",
                                             "iso8859-13",
                                             "iso8859-14",
                                             "iso8859-15",
                                             "iso8859-2",
                                             "iso8859-3",
                                             "iso8859-4",
                                             "iso8859-5",
                                             "iso8859-6",
                                             "iso8859-7",
                                             "iso8859-8",
                                             "iso8859-9",
                                             "iso88591",
                                             "iso885910",
                                             "iso885911",
                                             "iso885913",
                                             "iso885914",
                                             "iso885915",
                                             "iso88592",
                                             "iso88593",
                                             "iso88594",
                                             "iso88595",
                                             "iso88596",
                                             "iso88597",
                                             "iso88598",
                                             "iso88599",
                                             "iso_8859-1",
                                             "iso_8859-15",
                                             "iso_8859-1:1987",
                                             "iso_8859-2",
                                             "iso_8859-2:1987",
                                             "iso_8859-3",
                                             "iso_8859-3:1988",
                                             "iso_8859-4",
                                             "iso_8859-4:1988",
                                             "iso_8859-5",
                                             "iso_8859-5:1988",
                                             "iso_8859-6",
                                             "iso_8859-6:1987",
                                             "iso_8859-7",
                                             "iso_8859-7:1987",
                                             "iso_8859-8",
                                             "iso_8859-8:1988",
                                             "iso_8859-9",
                                             "iso_8859-9:1989",
                                             "koi",
                                             "koi8",
                                             "koi8-r",
                                             "koi8-ru",
                                             "koi8-u",
                                             "koi8_r",
                                             "korean",
                                             "ks_c_5601-1987",
                                             "ks_c_5601-1989",
                                             "ksc5601",
                                             "ksc_5601",
                                             "l1",
                                             "l2",
                                             "l3",
                                             "l4",
                                             "l5",
                                             "l6",
                                             "l9",
                                             "latin1",
                                             "latin2",
                                             "latin3",
                                             "latin4",
                                             "latin5",
                                             "latin6",
                                             "logical",
                                             "mac",
                                             "macintosh",
                                             "ms932",
                                             "ms_kanji",
                                             "shift-jis",
                                             "shift_jis",
                                             "sjis",
                                             "sun_eu_greek",
                                             "tis-620",
                                             "unicode-1-1-utf-8",
                                             "us-ascii",
                                             "utf-16",
                                             "utf-16be",
                                             "utf-16le",
                                             "utf-8",
                                             "utf8",
                                             "visual",
                                             "windows-1250",
                                             "windows-1251",
                                             "windows-1252",
                                             "windows-1253",
                                             "windows-1254",
                                             "windows-1255",
                                             "windows-1256",
                                             "windows-1257",
                                             "windows-1258",
                                             "windows-31j",
                                             "windows-874",
                                             "windows-949",
                                             "x-cp1250",
                                             "x-cp1251",
                                             "x-cp1252",
                                             "x-cp1253",
                                             "x-cp1254",
                                             "x-cp1255",
                                             "x-cp1256",
                                             "x-cp1257",
                                             "x-cp1258",
                                             "x-euc-jp",
                                             "x-gbk",
                                             "x-mac-cyrillic",
                                             "x-mac-roman",
                                             "x-mac-ukrainian",
                                             "x-sjis",
                                             "x-user-defined",
                                             "x-x-big5"];

static ENCODINGS_IN_LABEL_SORT: [&'static Encoding; 218] = [IBM866,
                                                            WINDOWS_1252,
                                                            ISO_8859_6,
                                                            WINDOWS_1252,
                                                            ISO_8859_6,
                                                            BIG5,
                                                            BIG5,
                                                            GBK,
                                                            BIG5,
                                                            WINDOWS_1250,
                                                            WINDOWS_1251,
                                                            WINDOWS_1252,
                                                            WINDOWS_1253,
                                                            WINDOWS_1254,
                                                            WINDOWS_1255,
                                                            WINDOWS_1256,
                                                            WINDOWS_1257,
                                                            WINDOWS_1258,
                                                            WINDOWS_1252,
                                                            IBM866,
                                                            BIG5,
                                                            EUC_KR,
                                                            EUC_JP,
                                                            GBK,
                                                            IBM866,
                                                            ISO_2022_JP,
                                                            REPLACEMENT,
                                                            GBK,
                                                            ISO_8859_6,
                                                            ISO_8859_6,
                                                            ISO_8859_8,
                                                            ISO_8859_8_I,
                                                            WINDOWS_1252,
                                                            ISO_8859_2,
                                                            ISO_8859_3,
                                                            ISO_8859_4,
                                                            WINDOWS_1254,
                                                            ISO_8859_10,
                                                            ISO_8859_15,
                                                            ISO_8859_6,
                                                            ISO_8859_5,
                                                            ISO_8859_7,
                                                            ISO_8859_8,
                                                            KOI8_R,
                                                            EUC_KR,
                                                            MACINTOSH,
                                                            SHIFT_JIS,
                                                            ISO_8859_5,
                                                            WINDOWS_874,
                                                            ISO_8859_6,
                                                            ISO_8859_7,
                                                            ISO_8859_7,
                                                            EUC_JP,
                                                            EUC_KR,
                                                            GB18030,
                                                            GBK,
                                                            GBK,
                                                            GBK,
                                                            GBK,
                                                            ISO_8859_7,
                                                            ISO_8859_7,
                                                            ISO_8859_8,
                                                            REPLACEMENT,
                                                            WINDOWS_1252,
                                                            IBM866,
                                                            REPLACEMENT,
                                                            REPLACEMENT,
                                                            ISO_2022_JP,
                                                            REPLACEMENT,
                                                            WINDOWS_1252,
                                                            ISO_8859_10,
                                                            WINDOWS_874,
                                                            ISO_8859_13,
                                                            ISO_8859_14,
                                                            ISO_8859_15,
                                                            ISO_8859_16,
                                                            ISO_8859_2,
                                                            ISO_8859_3,
                                                            ISO_8859_4,
                                                            ISO_8859_5,
                                                            ISO_8859_6,
                                                            ISO_8859_6,
                                                            ISO_8859_6,
                                                            ISO_8859_7,
                                                            ISO_8859_8,
                                                            ISO_8859_8,
                                                            ISO_8859_8_I,
                                                            WINDOWS_1254,
                                                            WINDOWS_1252,
                                                            ISO_8859_2,
                                                            ISO_8859_3,
                                                            ISO_8859_4,
                                                            ISO_8859_7,
                                                            ISO_8859_6,
                                                            ISO_8859_8,
                                                            ISO_8859_5,
                                                            WINDOWS_1254,
                                                            EUC_KR,
                                                            ISO_8859_10,
                                                            GBK,
                                                            WINDOWS_1252,
                                                            ISO_8859_10,
                                                            WINDOWS_874,
                                                            ISO_8859_13,
                                                            ISO_8859_14,
                                                            ISO_8859_15,
                                                            ISO_8859_2,
                                                            ISO_8859_3,
                                                            ISO_8859_4,
                                                            ISO_8859_5,
                                                            ISO_8859_6,
                                                            ISO_8859_7,
                                                            ISO_8859_8,
                                                            WINDOWS_1254,
                                                            WINDOWS_1252,
                                                            ISO_8859_10,
                                                            WINDOWS_874,
                                                            ISO_8859_13,
                                                            ISO_8859_14,
                                                            ISO_8859_15,
                                                            ISO_8859_2,
                                                            ISO_8859_3,
                                                            ISO_8859_4,
                                                            ISO_8859_5,
                                                            ISO_8859_6,
                                                            ISO_8859_7,
                                                            ISO_8859_8,
                                                            WINDOWS_1254,
                                                            WINDOWS_1252,
                                                            ISO_8859_15,
                                                            WINDOWS_1252,
                                                            ISO_8859_2,
                                                            ISO_8859_2,
                                                            ISO_8859_3,
                                                            ISO_8859_3,
                                                            ISO_8859_4,
                                                            ISO_8859_4,
                                                            ISO_8859_5,
                                                            ISO_8859_5,
                                                            ISO_8859_6,
                                                            ISO_8859_6,
                                                            ISO_8859_7,
                                                            ISO_8859_7,
                                                            ISO_8859_8,
                                                            ISO_8859_8,
                                                            WINDOWS_1254,
                                                            WINDOWS_1254,
                                                            KOI8_R,
                                                            KOI8_R,
                                                            KOI8_R,
                                                            KOI8_U,
                                                            KOI8_U,
                                                            KOI8_R,
                                                            EUC_KR,
                                                            EUC_KR,
                                                            EUC_KR,
                                                            EUC_KR,
                                                            EUC_KR,
                                                            WINDOWS_1252,
                                                            ISO_8859_2,
                                                            ISO_8859_3,
                                                            ISO_8859_4,
                                                            WINDOWS_1254,
                                                            ISO_8859_10,
                                                            ISO_8859_15,
                                                            WINDOWS_1252,
                                                            ISO_8859_2,
                                                            ISO_8859_3,
                                                            ISO_8859_4,
                                                            WINDOWS_1254,
                                                            ISO_8859_10,
                                                            ISO_8859_8_I,
                                                            MACINTOSH,
                                                            MACINTOSH,
                                                            SHIFT_JIS,
                                                            SHIFT_JIS,
                                                            SHIFT_JIS,
                                                            SHIFT_JIS,
                                                            SHIFT_JIS,
                                                            ISO_8859_7,
                                                            WINDOWS_874,
                                                            UTF_8,
                                                            WINDOWS_1252,
                                                            UTF_16LE,
                                                            UTF_16BE,
                                                            UTF_16LE,
                                                            UTF_8,
                                                            UTF_8,
                                                            ISO_8859_8,
                                                            WINDOWS_1250,
                                                            WINDOWS_1251,
                                                            WINDOWS_1252,
                                                            WINDOWS_1253,
                                                            WINDOWS_1254,
                                                            WINDOWS_1255,
                                                            WINDOWS_1256,
                                                            WINDOWS_1257,
                                                            WINDOWS_1258,
                                                            SHIFT_JIS,
                                                            WINDOWS_874,
                                                            EUC_KR,
                                                            WINDOWS_1250,
                                                            WINDOWS_1251,
                                                            WINDOWS_1252,
                                                            WINDOWS_1253,
                                                            WINDOWS_1254,
                                                            WINDOWS_1255,
                                                            WINDOWS_1256,
                                                            WINDOWS_1257,
                                                            WINDOWS_1258,
                                                            EUC_JP,
                                                            GBK,
                                                            X_MAC_CYRILLIC,
                                                            MACINTOSH,
                                                            X_MAC_CYRILLIC,
                                                            SHIFT_JIS,
                                                            X_USER_DEFINED,
                                                            BIG5];

// END GENERATED CODE

/// An encoding as defined in the
/// [Encoding Standard](https://encoding.spec.whatwg.org/).
///
/// An _encoding_ defines a mapping from a `u8` sequence to a `char` sequence
/// and, in most cases, vice versa. Each encoding has a name, an output
/// encoding, and one or more labels.
///
/// _Labels_ are ASCII-case-insensitive strings that are used to identify an
/// encoding in formats and protocols. The _name_ of the encoding is the
/// preferred label in the case appropriate for returning from the
/// [`characterSet`](https://dom.spec.whatwg.org/#dom-document-characterset)
/// property of the `Document` DOM interface, except for the replacement
/// encoding whose name is not one of its labels.
///
/// The _output encoding_ is the encoding used for form submission and URL
/// parsing on Web pages in the encoding. This is UTF-8 for the replacement,
/// UTF-16LE and UTF-16BE encodings and the encoding itself for other
/// encodings.
///
/// ## Instances
///
/// All instances of `Encoding` are statically allocated and have the `'static`
/// lifetime. There is precisely one unique `Encoding` instance for each
/// encoding defined in the Encoding Standard.
///
/// To obtain a reference to a particular encoding whose identity you know at
/// compile time, use a constant. There is a constant for each encoding. The
/// constants are named in all caps with hyphens replaced with underscores (and
/// in C/C++ have `_ENCODING` appended to the name). For example, if you know
/// at compile time that you will want to decode using the UTF-8 encoding, use
/// the `UTF_8` constant (`UTF_8_ENCODING` in C/C++).
///
/// If you don't know what encoding you need at compile time and need to
/// dynamically get an encoding by label, use
/// Encoding::for_label(<var>label</var>).
///
/// Instances of `Encoding` can be compared with `==` (in both Rust and in
/// C/C++).
///
/// ## Streaming vs. Non-Streaming
///
/// When you have the entire input in a single buffer, you can use the
/// convenience methods XXX. (These methods are available to Rust callers
/// only and are not available in the C API.) Unlike the rest of the API
/// available to Rust, these methods perform heap allocations. You should
/// the `Decoder` and `Encoder` objects when your input is split into multiple
/// buffers or when you want to control the allocation of the output buffers.
pub struct Encoding {
    name: &'static str,
    variant: VariantEncoding,
}

impl Encoding {
    /// Implements the
    /// [_get an encoding_](https://encoding.spec.whatwg.org/#concept-encoding-get)
    /// algorithm.
    ///
    /// If, after ASCII-lowercasing and removing leading and trailing
    /// whitespace, the argument matches a label defined in the Encoding
    /// Standard, `Some(&'static Encoding)` representing the corresponding
    /// encoding is returned. If there is no match, `None` is returned.
    ///
    /// The argument is of type `&[u8]` instead of `&str` to save callers
    /// that are extracting the label from a non-UTF-8 protocol the trouble
    /// of conversion to UTF-8. (If you have a `&str`, just call `.as_bytes()`
    /// on it.)
    ///
    /// Available via the C wrapper.
    pub fn for_label(label: &[u8]) -> Option<&'static Encoding> {
        let mut trimmed = [0u8; LONGEST_LABEL_LENGTH];
        let mut trimmed_pos = 0usize;
        let mut iter = label.into_iter();
        // before
        loop {
            match iter.next() {
                None => {
                    return None;
                }
                Some(byte) => {
                    match *byte {
                        0x09u8 | 0x0Au8 | 0x0Cu8 | 0x0Du8 | 0x20u8 => {
                            continue;
                        }
                        b'A'...b'Z' => {
                            trimmed[trimmed_pos] = *byte + 0x20u8;
                            trimmed_pos = 1usize;
                            break;
                        }
                        _ => {
                            trimmed[trimmed_pos] = *byte;
                            trimmed_pos = 1usize;
                            break;
                        }
                    }
                }
            }
        }
        // inside
        loop {
            match iter.next() {
                None => {
                    break;
                }
                Some(byte) => {
                    match *byte {
                        0x09u8 | 0x0Au8 | 0x0Cu8 | 0x0Du8 | 0x20u8 => {
                            break;
                        }
                        b'A'...b'Z' => {
                            trimmed[trimmed_pos] = *byte + 0x20u8;
                            trimmed_pos += 1usize;
                            if trimmed_pos > LONGEST_LABEL_LENGTH {
                                // There's no encoding with a label this long
                                return None;
                            }
                            continue;
                        }
                        _ => {
                            trimmed[trimmed_pos] = *byte;
                            trimmed_pos += 1usize;
                            if trimmed_pos > LONGEST_LABEL_LENGTH {
                                // There's no encoding with a label this long
                                return None;
                            }
                            continue;
                        }
                    }
                }
            }

        }
        // after
        loop {
            match iter.next() {
                None => {
                    break;
                }
                Some(byte) => {
                    match *byte {
                        0x09u8 | 0x0Au8 | 0x0Cu8 | 0x0Du8 | 0x20u8 => {
                            continue;
                        }
                        _ => {
                            // There's no label with space in the middle
                            return None;
                        }
                    }
                }
            }

        }
        let candidate = &trimmed[..trimmed_pos];
        // XXX optimize this to binary search, potentially with a comparator
        // that reads the name from the end to start.
        for i in 0..LABELS_SORTED.len() {
            let l = LABELS_SORTED[i];
            if candidate == l.as_bytes() {
                return Some(ENCODINGS_IN_LABEL_SORT[i]);
            }
        }
        return None;
    }

    /// This method behaves the same as `for_label()`, except when `for_label()`
    /// would return `Some(REPLACEMENT)`, this method returns `None` instead.
    ///
    /// This method is useful in scenarios where a fatal error is required
    /// upon invalid label, because in those cases the caller typically wishes
    /// to treat the labels that map to the replacement encoding as fatal
    /// errors, too.
    ///
    /// Available via the C wrapper.
    pub fn for_label_no_replacement(label: &[u8]) -> Option<&'static Encoding> {
        match Encoding::for_label(label) {
            None => None,
            Some(encoding) => {
                if encoding == REPLACEMENT {
                    None
                } else {
                    Some(encoding)
                }
            }
        }
    }

    /// If the argument matches exactly (case-sensitively; no whitespace
    /// removal performed) the name of an encoding, returns
    /// `Some(&'static Encoding)` representing that encoding. Otherwise,
    /// return `None`.
    ///
    /// The motivating use case for this method is interoperability with
    /// legacy Gecko code that represents encodings as name string instead of
    /// type-safe `Encoding` objects. Using this method for other purposes is
    /// most likely the wrong thing to do.
    ///
    /// XXX: Should this method be made FFI-only to discourage Rust callers?
    ///
    /// Available via the C wrapper.
    pub fn for_name(name: &[u8]) -> Option<&'static Encoding> {
        // XXX optimize this to binary search, potentially with a comparator
        // that reads the name from the end to start.
        for i in 0..ENCODINGS_SORTED_BY_NAME.len() {
            let encoding = ENCODINGS_SORTED_BY_NAME[i];
            if name == encoding.name().as_bytes() {
                return Some(ENCODINGS_IN_LABEL_SORT[i]);
            }
        }
        return None;
    }

    /// Performs non-incremental BOM sniffing.
    ///
    /// The argument must either be a buffer representing the entire input
    /// stream (non-streaming case) or a buffer representing at least the first
    /// three bytes of the input stream (streaming case).
    ///
    /// Returns `Some(UTF_8)`, `Some(UTF_16LE)` or `Some(UTF_16BE)` if the
    /// argument starts with the UTF-8, UTF-16LE or UTF-16BE BOM or `None`
    /// otherwise.
    ///
    /// Available via the C wrapper.
    pub fn for_bom(buffer: &[u8]) -> Option<&'static Encoding> {
        if buffer.starts_with(b"\xEF\xBB\xBF") {
            Some(UTF_8)
        } else if buffer.starts_with(b"\xFF\xFE") {
            Some(UTF_16LE)
        } else if buffer.starts_with(b"\xFE\xFF") {
            Some(UTF_16BE)
        } else {
            None
        }
    }

    /// Returns the name of this encoding.
    ///
    /// This name is appropriate to return as-is from the DOM
    /// `document.characterSet` property.
    ///
    /// Available via the C wrapper.
    pub fn name(&'static self) -> &'static str {
        self.name
    }

    /// Checks whether the _output encoding_ of this encoding can encode every
    /// `char`. (Only true if the output encoding is UTF-8.)
    ///
    /// Available via the C wrapper.
    pub fn can_encode_everything(&'static self) -> bool {
        self.output_encoding() == UTF_8
    }

    /// Checks whether the bytes 0x00...0x7F map exclusively to the characters
    /// U+0000...U+007F and vice versa.
    ///
    /// Available via the C wrapper.
    pub fn is_ascii_compatible(&'static self) -> bool {
        if self == REPLACEMENT || self == UTF_16BE || self == UTF_16LE || self == ISO_2022_JP {
            false
        } else {
            true
        }
    }

    /// Returns the _output encoding_ of this encoding. This is UTF-8 for
    /// UTF-16BE, UTF-16LE and replacement and the encoding itself otherwise.
    ///
    /// Available via the C wrapper.
    pub fn output_encoding(&'static self) -> &'static Encoding {
        if self == REPLACEMENT || self == UTF_16BE || self == UTF_16LE {
            UTF_8
        } else {
            self
        }
    }

    fn new_variant_decoder(&'static self) -> VariantDecoder {
        self.variant.new_variant_decoder()
    }

    /// Instantiates a new decoder for this encoding with BOM sniffing enabled.
    ///
    /// BOM sniffing may cause the returned decoder to morph into a decoder
    /// for UTF-8, UTF-16LE or UTF-16BE instead of this encoding.
    ///
    /// Available via the C wrapper.
    pub fn new_decoder(&'static self) -> Decoder {
        Decoder::new(self, self.new_variant_decoder(), BomHandling::Sniff)
    }

    /// Instantiates a new decoder for this encoding with BOM removal.
    ///
    /// If the input starts with bytes that are the BOM for this encoding,
    /// those bytes are removed. However, the decoder never morphs into a
    /// decoder for another encoding: A BOM for another encoding is treated as
    /// (potentially malformed) input to the decoding algorithm for this
    /// encoding.
    ///
    /// Available via the C wrapper.
    pub fn new_decoder_with_bom_removal(&'static self) -> Decoder {
        Decoder::new(self, self.new_variant_decoder(), BomHandling::Remove)
    }

    /// Instantiates a new decoder for this encoding with BOM handling disabled.
    ///
    /// If the input starts with bytes that look like a BOM, those bytes are
    /// not treated as a BOM. (Hence, the decoder never morphs into a decoder
    /// for another encoding.)
    ///
    /// _Note:_ If the caller has performed BOM sniffing on its own but has not
    /// removed the BOM, the caller should use `new_decoder_with_bom_removal()`
    /// instead of this method to cause the BOM to be removed.
    ///
    /// Available via the C wrapper.
    pub fn new_decoder_without_bom_handling(&'static self) -> Decoder {
        Decoder::new(self, self.new_variant_decoder(), BomHandling::Off)
    }

    /// Instantiates a new encoder for the output encoding of this encoding.
    ///
    /// Available via the C wrapper.
    pub fn new_encoder(&'static self) -> Encoder {
        let enc = self.output_encoding();
        enc.variant.new_encoder(enc)
    }

    /// Convenience method for decoding to `String` _with BOM sniffing_ and with
    /// malformed sequences replaced with the REPLACEMENT CHARACTER when the
    /// entire input is available as a single buffer (i.e. the end of the
    /// buffer marks the end of the stream).
    ///
    /// This method implements the (non-streaming version of) the
    /// [_decode_](https://encoding.spec.whatwg.org/#decode) spec concept.
    ///
    /// The second item in the returned tuple is the encoding that was actually
    /// used (which may differ from this encoding thanks to BOM sniffing).
    ///
    /// The third item in the returned tuple indicates whether there were
    /// malformed sequences (that were replaced with the REPLACEMENT CHARACTER).
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input. Use `new_decoder()`
    /// when decoding segmented input.
    ///
    /// This method performs a single heap allocation for the backing buffer
    /// of the `String`.
    ///
    /// Available to Rust only.
    pub fn decode(&'static self, bytes: &[u8]) -> (String, &'static Encoding, bool) {
        let mut decoder = self.new_decoder();
        let mut string = String::with_capacity(decoder.max_utf8_buffer_length(bytes.len()));
        let (result, read, had_errors) = decoder.decode_to_string(bytes, &mut string, true);
        match result {
            CoderResult::InputEmpty => {
                debug_assert_eq!(read, bytes.len());
                (string, decoder.encoding(), had_errors)
            }
            CoderResult::OutputFull => unreachable!(),
        }
    }

    /// Convenience method for decoding to `String` _with BOM removal_ and with
    /// malformed sequences replaced with the REPLACEMENT CHARACTER when the
    /// entire input is available as a single buffer (i.e. the end of the
    /// buffer marks the end of the stream).
    ///
    /// When invoked on `UTF_8`, this method implements the (non-streaming
    /// version of) the
    /// [_UTF-8 decode_](https://encoding.spec.whatwg.org/#utf-8-decode) spec
    /// concept.
    ///
    /// The second item in the returned pair indicates whether there were
    /// malformed sequences (that were replaced with the REPLACEMENT CHARACTER).
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input. Use
    /// `new_decoder_with_bom_removal()` when decoding segmented input.
    ///
    /// This method performs a single heap allocation for the backing buffer
    /// of the `String`.
    ///
    /// Available to Rust only.
    pub fn decode_with_bom_removal(&'static self, bytes: &[u8]) -> (String, bool) {
        let mut decoder = self.new_decoder_with_bom_removal();
        let mut string = String::with_capacity(decoder.max_utf8_buffer_length(bytes.len()));
        let (result, read, had_errors) = decoder.decode_to_string(bytes, &mut string, true);
        match result {
            CoderResult::InputEmpty => {
                debug_assert_eq!(read, bytes.len());
                (string, had_errors)
            }
            CoderResult::OutputFull => unreachable!(),
        }
    }

    /// Convenience method for decoding to `String` _without BOM handling_ and
    /// with malformed sequences replaced with the REPLACEMENT CHARACTER when
    /// the entire input is available as a single buffer (i.e. the end of the
    /// buffer marks the end of the stream).
    ///
    /// When invoked on `UTF_8`, this method implements the (non-streaming
    /// version of) the
    /// [_UTF-8 decode without BOM_](https://encoding.spec.whatwg.org/#utf-8-decode-without-bom)
    /// spec concept.
    ///
    /// The second item in the returned pair indicates whether there were
    /// malformed sequences (that were replaced with the REPLACEMENT CHARACTER).
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input. Use
    /// `new_decoder_without_bom_handling()` when decoding segmented input.
    ///
    /// This method performs a single heap allocation for the backing buffer
    /// of the `String`.
    ///
    /// Available to Rust only.
    pub fn decode_without_bom_handling(&'static self, bytes: &[u8]) -> (String, bool) {
        let mut decoder = self.new_decoder_without_bom_handling();
        let mut string = String::with_capacity(decoder.max_utf8_buffer_length(bytes.len()));
        let (result, read, had_errors) = decoder.decode_to_string(bytes, &mut string, true);
        match result {
            CoderResult::InputEmpty => {
                debug_assert_eq!(read, bytes.len());
                (string, had_errors)
            }
            CoderResult::OutputFull => unreachable!(),
        }
    }

    /// Convenience method for decoding to `String` _without BOM handling_ and
    /// _with malformed sequences treated as fatal_ when the entire input is
    /// available as a single buffer (i.e. the end of the buffer marks the end
    /// of the stream).
    ///
    /// When invoked on `UTF_8`, this method implements the (non-streaming
    /// version of) the
    /// [_UTF-8 decode without BOM or fail_](https://encoding.spec.whatwg.org/#utf-8-decode-without-bom-or-fail)
    /// spec concept.
    ///
    /// Returns `None` if a malformed sequence was encountered and the result
    /// of the decode as `Some(String)` otherwise.
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input. Use
    /// `new_decoder_without_bom_handling()` when decoding segmented input.
    ///
    /// This method performs a single heap allocation for the backing buffer
    /// of the `String`.
    ///
    /// Available to Rust only.
    pub fn decode_without_bom_handling_and_without_replacement(&'static self,
                                                               bytes: &[u8])
                                                               -> Option<String> {
        let mut decoder = self.new_decoder_without_bom_handling();
        let mut string =
            String::with_capacity(decoder.max_utf8_buffer_length_without_replacement(bytes.len()));
        let (result, read) = decoder.decode_to_string_without_replacement(bytes, &mut string, true);
        match result {
            DecoderResult::InputEmpty => {
                debug_assert_eq!(read, bytes.len());
                Some(string)
            }
            DecoderResult::Malformed(_, _) => None,
            DecoderResult::OutputFull => unreachable!(),
        }
    }

    /// Convenience method for encoding to `Vec<u8>` with unmappable characters
    /// replaced with decimal numeric character references when the entire input
    /// is available as a single buffer (i.e. the end of the buffer marks the
    /// end of the stream).
    ///
    /// This method implements the (non-streaming version of) the
    /// [_encode_](https://encoding.spec.whatwg.org/#encode) spec concept. For
    /// the [_UTF-8 encode_](https://encoding.spec.whatwg.org/#utf-8-encode)
    /// spec concept, use <code><var>string</var>.as_bytes()</code> instead of
    /// invoking this method on `UTF_8`.
    ///
    /// The second item in the returned tuple is the encoding that was actually
    /// used (which may differ from this encoding thanks to some encodings
    /// having UTF-8 as their output encoding).
    ///
    /// The third item in the returned tuple indicates whether there were
    /// unmappable characters (that were replaced with HTML numeric character
    /// references).
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input. Use `new_encoder()`
    /// when encoding segmented output.
    ///
    /// This method performs a single heap allocation for the backing buffer
    /// of the `Vec<u8>` if there are no unmappable characters and potentially
    /// multiple heap allocations if there are. These allocations are tuned
    /// for jemalloc and may not be optimal when using a different allocator
    /// that doesn't use power-of-two buckets.
    ///
    /// Available to Rust only.
    pub fn encode(&'static self, string: &str) -> (Vec<u8>, &'static Encoding, bool) {
        let mut encoder = self.new_encoder();
        let mut total_read = 0usize;
        let mut vec: Vec<u8> =
            Vec::with_capacity(encoder.max_buffer_length_from_utf8_if_no_unmappables(string.len())
                .next_power_of_two());
        let mut total_had_errors = false;
        loop {
            let (result, read, had_errors) =
                encoder.encode_from_utf8_to_vec(&string[total_read..], &mut vec, true);
            total_read += read;
            if had_errors {
                total_had_errors = true;
            }
            match result {
                CoderResult::InputEmpty => {
                    debug_assert_eq!(total_read, string.len());
                    return (vec, encoder.encoding(), total_had_errors);
                }
                CoderResult::OutputFull => {
                    // reserve_exact wants to know how much more on top of current
                    // length--not current capacity.
                    let needed =
                        encoder.max_buffer_length_from_utf8_if_no_unmappables(string.len() -
                                                                              total_read);
                    let rounded = (vec.capacity() + needed).next_power_of_two();
                    let additional = rounded - vec.len();
                    vec.reserve_exact(additional);
                }
            }
        }
    }
}

impl PartialEq for Encoding {
    fn eq(&self, other: &Encoding) -> bool {
        //        (self as *const Encoding) == (other as *const Encoding)
        // Compare by name rather than by pointer pending an answer to
        // https://users.rust-lang.org/t/how-to-expose-a-static-reference-and-an-unmangled-static-pointer-to-the-same-memory-location/6529
        self.name == other.name
    }
}

impl Eq for Encoding {}

impl std::fmt::Debug for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Encoding {{ {} }}", self.name)
    }
}

/// Tracks the life cycle of a decoder from BOM sniffing to conversion to end.
#[derive(PartialEq)]
enum DecoderLifeCycle {
    /// The decoder has seen no input yet.
    AtStart,
    /// The decoder has seen no input yet but expects UTF-8.
    AtUtf8Start,
    /// The decoder has seen no input yet but expects UTF-16BE.
    AtUtf16BeStart,
    /// The decoder has seen no input yet but expects UTF-16LE.
    AtUtf16LeStart,
    /// The decoder has seen EF.
    SeenUtf8First,
    /// The decoder has seen EF, BB.
    SeenUtf8Second,
    /// The decoder has seen FE.
    SeenUtf16BeFirst,
    /// The decoder has seen FF.
    SeenUtf16LeFirst,
    /// Saw EF, BB but not BF, there was a buffer boundary after BB and the
    /// underlying decoder reported EF as an error, so we need to remember to
    /// push BB before the next buffer.
    ConvertingWithPendingBB,
    /// No longer looking for a BOM and EOF not yet seen.
    Converting,
    /// EOF has been seen.
    Finished,
}

/// Communicate the BOM handling mode.
enum BomHandling {
    /// Don't handle the BOM
    Off,
    /// Sniff for UTF-8, UTF-16BE or UTF-16LE BOM
    Sniff,
    /// Remove the BOM only if it's the BOM for this encoding
    Remove,
}

/// Result of a (potentially partial) decode or encode operation with
/// replacement.
#[derive(Debug)]
pub enum CoderResult {
    /// The input was exhausted.
    ///
    /// If this result was returned from a call where `last` was `true`, the
    /// conversion process has completed. Otherwise, the caller should call a
    /// decode or encode method again with more input.
    InputEmpty,

    /// The converter cannot produce another unit of output, because the output
    /// buffer does not have enough space left.
    ///
    /// The caller must provide more output space upon the next call and re-push
    /// the remaining input to the converter.
    OutputFull,
}

/// Result of a (potentially partial) decode operation without replacement.
#[derive(Debug)]
pub enum DecoderResult {
    /// The input was exhausted.
    ///
    /// If this result was returned from a call where `last` was `true`, the
    /// decoding process has completed. Otherwise, the caller should call a
    /// decode method again with more input.
    InputEmpty,

    /// The decoder cannot produce another unit of output, because the output
    /// buffer does not have enough space left.
    ///
    /// The caller must provide more output space upon the next call and re-push
    /// the remaining input to the decoder.
    OutputFull,

    /// The decoder encountered a malformed byte sequence.
    ///
    /// The caller must either treat this as a fatal error or must append one
    /// REPLACEMENT CHARACTER (U+FFFD) to the output and then re-push the
    /// the remaining input to the decoder.
    ///
    /// The first wrapped integer indicates the length of the malformed byte
    /// sequence. The second wrapped integer indicates the number of bytes
    /// that were consumed after the malformed sequence. If the second
    /// integer is zero, the last byte that was consumed is the last byte of
    /// the malformed sequence. Note that the malformed bytes may have been part
    /// of an earlier input buffer.
    Malformed(u8, u8), // u8 instead of usize to avoid useless bloat
}

/// A converter that decodes a byte stream into Unicode according to a
/// character encoding in a streaming (incremental) manner.
///
/// The various `decode_*` methods take an input buffer (`src`) and an output
/// buffer `dst` both of which are caller-allocated. There are variants for
/// both UTF-8 and UTF-16 output buffers.
///
/// A `decode_*` method decodes bytes from `src` into Unicode characters stored
/// into `dst` until one of the following three things happens:
///
/// 1. A malformed byte sequence is encountered.
///
/// 2. The output buffer has been filled so near capacity that the decoder
///    cannot be sure that processing an additional byte of input wouldn't
///    cause so much output that the output buffer would overflow.
///
/// 3. All the input bytes have been processed.
///
/// The `decode_*` method then returns tuple of a status indicating which one
/// of the three reasons to return happened, how many input bytes were read,
/// how many output code units (`u8` when decoding into UTF-8 and `u16`
/// when decoding to UTF-16) were written (except when decoding into `String`,
/// whose length change indicates this), and in the case of the
/// variants performing replacement, a boolean indicating whether an error was
/// replaced with the REPLACEMENT CHARACTER during the call.
///
/// In the case of the `*_without_replacement` variants, the status is a
/// `DecoderResult` enumeration (possibilities `Malformed`, `OutputFull` and
/// `InputEmpty` corresponding to the three cases listed above).
///
/// In the case of methods whose name does not end with
/// `*_without_replacement`, malformed sequences are automatically replaced
/// with the REPLACEMENT CHARACTER and errors do not cause the methods to
/// return early.
///
/// When decoding to UTF-8, the output buffer must have at least 4 bytes of
/// space. When decoding to UTF-16, the output buffer must have at least two
/// UTF-16 code units (`u16`) of space.
///
/// When decoding to UTF-8 without replacement, the methods are guaranteed
/// not to return indicating that more output space is needed if the length
/// of the ouput buffer is at least the length returned by
/// `max_utf8_buffer_length_without_replacement()`. When decoding to UTF-8 with
/// replacement, the the length of the output buffer that guarantees the
/// methods not to return indicating that more output space is needed is given
/// by `max_utf8_buffer_length()`. When decoding to UTF-16 with
/// or without replacement, the length of the output buffer that guarantees
/// the methods not to return indicating that more output space is needed is
/// given by `max_utf16_buffer_length()`.
///
/// The output written into `dst` is guaranteed to be valid UTF-8 or UTF-16,
/// and the output after each `decode_*` call is guaranteed to consist of
/// complete characters. (I.e. the code unit sequence for the last character is
/// guaranteed not to be split across output buffers.)
///
/// The boolean argument `last` indicates that the end of the stream is reached
/// when all the bytes in `src` have been consumed.
///
/// A `Decoder` object can be used to incrementally decode a byte stream.
///
/// During the processing of a single stream, the caller must call `decode_*`
/// zero or more times with `last` set to `false` and then call `decode_*` at
/// least once with `last` set to `true`. If `decode_*` returns `InputEmpty`,
/// the processing of the stream has ended. Otherwise, the caller must call
/// `decode_*` again with `last` set to `true` (or treat a `Malformed` result as
///  a fatal error).
///
/// Once the stream has ended, the `Decoder` object must not be used anymore.
/// That is, you need to create another one to process another stream.
///
/// When the decoder returns `OutputFull` or the decoder returns `Malformed` and
/// the caller does not wish to treat it as a fatal error, the input buffer
/// `src` may not have been completely consumed. In that case, the caller must
/// pass the unconsumed contents of `src` to `decode_*` again upon the next
/// call.
pub struct Decoder {
    encoding: &'static Encoding,
    variant: VariantDecoder,
    life_cycle: DecoderLifeCycle,
}

impl Decoder {
    fn new(enc: &'static Encoding, decoder: VariantDecoder, sniffing: BomHandling) -> Decoder {
        Decoder {
            encoding: enc,
            variant: decoder,
            life_cycle: match sniffing {
                BomHandling::Off => DecoderLifeCycle::Converting,
                BomHandling::Sniff => DecoderLifeCycle::AtStart,
                BomHandling::Remove => {
                    if enc == UTF_8 {
                        DecoderLifeCycle::AtUtf8Start
                    } else if enc == UTF_16BE {
                        DecoderLifeCycle::AtUtf16BeStart
                    } else if enc == UTF_16LE {
                        DecoderLifeCycle::AtUtf16LeStart
                    } else {
                        DecoderLifeCycle::Converting
                    }
                }
            },
        }
    }

    /// The `Encoding` this `Decoder` is for.
    ///
    /// BOM sniffing can change the return value of this method during the life
    /// of the decoder.
    pub fn encoding(&self) -> &'static Encoding {
        self.encoding
    }

    /// Query the worst-case UTF-16 output size (with or without replacement).
    ///
    /// Returns the size of the output buffer in UTF-16 code units (`u16`)
    /// that will not overflow given the current state of the decoder and
    /// `byte_length` number of additional input bytes.
    ///
    /// Since the REPLACEMENT CHARACTER fits into one UTF-16 code unit, the
    /// return value of this method applies also in the
    /// `_with_replacement` case.
    ///
    /// Available via the C wrapper.
    pub fn max_utf16_buffer_length(&self, byte_length: usize) -> usize {
        self.variant.max_utf16_buffer_length(byte_length)
    }

    /// Query the worst-case UTF-8 output size _without replacement_.
    ///
    /// Returns the size of the output buffer in UTF-8 code units (`u8`)
    /// that will not overflow given the current state of the decoder and
    /// `byte_length` number of additional input bytes when decoding without
    /// replacement error handling.
    ///
    /// Note that this value may be too small for the `_with_replacement` case.
    /// Use `max_utf8_buffer_length` for that case.
    ///
    /// Available via the C wrapper.
    pub fn max_utf8_buffer_length_without_replacement(&self, byte_length: usize) -> usize {
        self.variant.max_utf8_buffer_length_without_replacement(byte_length)
    }

    /// Query the worst-case UTF-8 output size _with replacement_.
    ///
    /// Returns the size of the output buffer in UTF-8 code units (`u8`)
    /// that will not overflow given the current state of the decoder and
    /// `byte_length` number of additional input bytes when decoding with
    /// errors handled by outputting a REPLACEMENT CHARACTER for each malformed
    /// sequence.
    ///
    /// Available via the C wrapper.
    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        self.variant.max_utf8_buffer_length(byte_length)
    }

    public_decode_function!(/// Incrementally decode a byte stream into UTF-16.
                            ///
                            /// See the documentation of the struct for
                            /// documentation for `decode_*` methods
                            /// collectively.
                            ///
                            /// Available via the C wrapper.
                            ,
                            decode_to_utf16_without_replacement,
                            decode_to_utf16_raw,
                            decode_to_utf16_checking_end,
                            decode_to_utf16_after_one_potential_bom_byte,
                            decode_to_utf16_after_two_potential_bom_bytes,
                            decode_to_utf16_checking_end_with_offset,
                            u16);

    public_decode_function!(/// Incrementally decode a byte stream into UTF-8.
                            ///
                            /// See the documentation of the struct for
                            /// documentation for `decode_*` methods
                            /// collectively.
                            ///
                            /// Available via the C wrapper.
                            ,
                            decode_to_utf8_without_replacement,
                            decode_to_utf8_raw,
                            decode_to_utf8_checking_end,
                            decode_to_utf8_after_one_potential_bom_byte,
                            decode_to_utf8_after_two_potential_bom_bytes,
                            decode_to_utf8_checking_end_with_offset,
                            u8);

    /// Incrementally decode a byte stream into UTF-8 with type system signaling
    /// of UTF-8 validity.
    ///
    /// This methods calls `decode_to_utf8` and then zeroes out up to three
    /// bytes that aren't logically part of the write in order to retain the
    /// UTF-8 validity even for the unwritten part of the buffer.
    ///
    /// See the documentation of the struct for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    pub fn decode_to_str_without_replacement(&mut self,
                                             src: &[u8],
                                             dst: &mut str,
                                             last: bool)
                                             -> (DecoderResult, usize, usize) {
        let bytes: &mut [u8] = unsafe { std::mem::transmute(dst) };
        let (result, read, written) = self.decode_to_utf8_without_replacement(src, bytes, last);
        let len = bytes.len();
        let mut trail = written;
        while trail < len && ((bytes[trail] & 0xC0) == 0x80) {
            bytes[trail] = 0;
            trail += 1;
        }
        (result, read, written)
    }

    /// Incrementally decode a byte stream into UTF-8 using a `String` receiver.
    ///
    /// Like the others, this method follows the logic that the output buffer is
    /// caller-allocated. This method treats the capacity of the `String` as
    /// the output limit. That is, this method guarantees not to cause a
    /// reallocation of the backing buffer of `String`.
    ///
    /// The return value is a pair that contains the `DecoderResult` and the
    /// number of bytes read. The number of bytes written is signaled via
    /// the length of the `String` changing.
    ///
    /// See the documentation of the struct for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    pub fn decode_to_string_without_replacement(&mut self,
                                                src: &[u8],
                                                dst: &mut String,
                                                last: bool)
                                                -> (DecoderResult, usize) {
        unsafe {
            let vec = dst.as_mut_vec();
            let old_len = vec.len();
            let capacity = vec.capacity();
            vec.set_len(capacity);
            let (result, read, written) =
                self.decode_to_utf8_without_replacement(src, &mut vec[old_len..], last);
            vec.set_len(old_len + written);
            (result, read)
        }
    }

    /// Incrementally decode a byte stream into UTF-16 with malformed sequences
    /// replaced with the REPLACEMENT CHARACTER.
    ///
    /// See the documentation of the struct for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn decode_to_utf16(&mut self,
                           src: &[u8],
                           dst: &mut [u16],
                           last: bool)
                           -> (CoderResult, usize, usize, bool) {
        let mut had_errors = false;
        let mut total_read = 0usize;
        let mut total_written = 0usize;
        loop {
            let (result, read, written) =
                self.decode_to_utf16_without_replacement(&src[total_read..],
                                                         &mut dst[total_written..],
                                                         last);
            total_read += read;
            total_written += written;
            match result {
                DecoderResult::InputEmpty => {
                    return (CoderResult::InputEmpty, total_read, total_written, had_errors);
                }
                DecoderResult::OutputFull => {
                    return (CoderResult::OutputFull, total_read, total_written, had_errors);
                }
                DecoderResult::Malformed(_, _) => {
                    had_errors = true;
                    // There should always be space for the U+FFFD, because
                    // otherwise we'd have gotten OutputFull already.
                    dst[total_written] = 0xFFFD;
                    total_written += 1;
                }
            }
        }
    }

    /// Incrementally decode a byte stream into UTF-8 with malformed sequences
    /// replaced with the REPLACEMENT CHARACTER.
    ///
    /// See the documentation of the struct for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn decode_to_utf8(&mut self,
                          src: &[u8],
                          dst: &mut [u8],
                          last: bool)
                          -> (CoderResult, usize, usize, bool) {
        let mut had_errors = false;
        let mut total_read = 0usize;
        let mut total_written = 0usize;
        loop {
            let (result, read, written) =
                self.decode_to_utf8_without_replacement(&src[total_read..],
                                                        &mut dst[total_written..],
                                                        last);
            total_read += read;
            total_written += written;
            match result {
                DecoderResult::InputEmpty => {
                    return (CoderResult::InputEmpty, total_read, total_written, had_errors);
                }
                DecoderResult::OutputFull => {
                    return (CoderResult::OutputFull, total_read, total_written, had_errors);
                }
                DecoderResult::Malformed(_, _) => {
                    had_errors = true;
                    // There should always be space for the U+FFFD, because
                    // otherwise we'd have gotten OutputFull already.
                    // XXX: is the above comment actually true for UTF-8 itself?
                    // TODO: Consider having fewer bound checks here.
                    dst[total_written] = 0xEFu8;
                    total_written += 1;
                    dst[total_written] = 0xBFu8;
                    total_written += 1;
                    dst[total_written] = 0xBDu8;
                    total_written += 1;
                }
            }
        }
    }

    /// Incrementally decode a byte stream into UTF-8 with malformed sequences
    /// replaced with the REPLACEMENT CHARACTER with type system signaling
    /// of UTF-8 validity.
    ///
    /// This methods calls `decode_to_utf8` and then zeroes
    /// out up to three bytes that aren't logically part of the write in order
    /// to retain the UTF-8 validity even for the unwritten part of the buffer.
    ///
    /// See the documentation of the struct for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    pub fn decode_to_str(&mut self,
                         src: &[u8],
                         dst: &mut str,
                         last: bool)
                         -> (CoderResult, usize, usize, bool) {
        let bytes: &mut [u8] = unsafe { std::mem::transmute(dst) };
        let (result, read, written, replaced) = self.decode_to_utf8(src, bytes, last);
        let len = bytes.len();
        let mut trail = written;
        while trail < len && ((bytes[trail] & 0xC0) == 0x80) {
            bytes[trail] = 0;
            trail += 1;
        }
        (result, read, written, replaced)
    }

    /// Incrementally decode a byte stream into UTF-8 with malformed sequences
    /// replaced with the REPLACEMENT CHARACTER using a `String` receiver.
    ///
    /// Like the others, this method follows the logic that the output buffer is
    /// caller-allocated. This method treats the capacity of the `String` as
    /// the output limit. That is, this method guarantees not to cause a
    /// reallocation of the backing buffer of `String`.
    ///
    /// The return value is a tuple that contains the `DecoderResult`, the
    /// number of bytes read and a boolean indicating whether replacements
    /// were done. The number of bytes written is signaled via the length of
    /// the `String` changing.
    ///
    /// See the documentation of the struct for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    pub fn decode_to_string(&mut self,
                            src: &[u8],
                            dst: &mut String,
                            last: bool)
                            -> (CoderResult, usize, bool) {
        unsafe {
            let vec = dst.as_mut_vec();
            let old_len = vec.len();
            let capacity = vec.capacity();
            vec.set_len(capacity);
            let (result, read, written, replaced) =
                self.decode_to_utf8(src, &mut vec[old_len..], last);
            vec.set_len(old_len + written);
            (result, read, replaced)
        }
    }
}

/// Result of a (potentially partial) encode operation without replacement.
#[derive(Debug)]
pub enum EncoderResult {
    /// The input was exhausted.
    ///
    /// If this result was returned from a call where `last` was `true`, the
    /// decoding process has completed. Otherwise, the caller should call a
    /// decode method again with more input.
    InputEmpty,

    /// The encoder cannot produce another unit of output, because the output
    /// buffer does not have enough space left.
    ///
    /// The caller must provide more output space upon the next call and re-push
    /// the remaining input to the decoder.
    OutputFull,

    /// The encoder encountered an unmappable character.
    ///
    /// The caller must either treat this as a fatal error or must append
    /// a placeholder to the output and then re-push the the remaining input to
    /// the encoder.
    Unmappable(char),
}

/// A converter that encodes a Unicode stream into bytes according to a
/// character encoding in a streaming (incremental) manner.
///
/// The various `encode_*` methods take an input buffer (`src`) and an output
/// buffer `dst` both of which are caller-allocated. There are variants for
/// both UTF-8 and UTF-16 input buffers.
///
/// A `encode_*` methods encode characters from `src` into bytes characters
/// stored into `dst` until one of the following three things happens:
///
/// 1. An unmappable character is encountered.
///
/// 2. The output buffer has been filled so near capacity that the decoder
///    cannot be sure that processing an additional character of input wouldn't
///    cause so much output that the output buffer would overflow.
///
/// 3. All the input characters have been processed.
///
/// The `encode_*` method then returns tuple of a status indicating which one
/// of the three reasons to return happened, how many input code units (`u8`
/// when encoding from UTF-8 and `u16` when encoding from UTF-16) were read,
/// how many output bytes were written (except when encoding into `Vec<u8>`,
/// whose length change indicates this), and in the case of the variants that
/// perform replacement, a boolean indicating whether an unmappable
/// character was replaced with a numeric character reference during the call.
///
/// In the case of the methods whose name ends with
/// `*_without_replacement`, the status is an `EncoderResult` enumeration
/// (possibilities `Unmappable`, `OutputFull` and `InputEmpty` corresponding to
/// the three cases listed above).
///
/// In the case of methods whose name does not end with
/// `*_without_replacement`, unmappable characters are automatically replaced
/// with the corresponding numeric character references and unmappable
/// characters do not cause the methods to return early.
///
/// XXX: When decoding to UTF-8 without replacement, the methods are guaranteed
/// not to return indicating that more output space is needed if the length
/// of the ouput buffer is at least the length returned by
/// `max_utf8_buffer_length_without_replacement()`. When decoding to UTF-8 with replacement, the
/// the length of the output buffer that guarantees the methods not to return
/// indicating that more output space is needed is given by
/// `max_utf8_buffer_length()`. When decoding to UTF-16 with
/// or without replacement, the length of the output buffer that guarantees
/// the methods not to return indicating that more output space is needed is
/// given by `max_utf16_buffer_length()`.
///
/// When encoding from UTF-8, each `src` buffer _must_ be valid UTF-8. (When
/// calling from Rust, the type system takes care of this.) When encoding from
/// UTF-16, unpaired surrogates in the input are treated as U+FFFD REPLACEMENT
/// CHARACTERS. Therefore, in order for astral characters not to turn into a
/// pair of REPLACEMENT CHARACTERS, the caller must ensure that surrogate pairs
/// are not split across input buffer boundaries.
///
/// XXX: Except in the case of ISO-2022-JP, the output of each `encode_*` call is
/// guaranteed to consist of a valid byte sequence of complete characters.
/// (I.e. the code unit sequence for the last character is guaranteed not to be
/// split across output buffers.)
///
/// The boolean argument `last` indicates that the end of the stream is reached
/// when all the characters in `src` have been consumed. This argument is needed
/// for ISO-2022-JP and is ignored for other encodings.
///
/// An `Encoder` object can be used to incrementally encode a byte stream.
///
/// During the processing of a single stream, the caller must call `encode_*`
/// zero or more times with `last` set to `false` and then call `encode_*` at
/// least once with `last` set to `true`. If `encode_*` returns `InputEmpty`,
/// the processing of the stream has ended. Otherwise, the caller must call
/// `encode_*` again with `last` set to `true` (or treat an `Unmappable` result
/// as a fatal error).
///
/// Once the stream has ended, the `Encoder` object must not be used anymore.
/// That is, you need to create another one to process another stream.
///
/// When the encoder returns `OutputFull` or the encoder returns `Unmappable`
/// and the caller does not wish to treat it as a fatal error, the input buffer
/// `src` may not have been completely consumed. In that case, the caller must
/// pass the unconsumed contents of `src` to `encode_*` again upon the next
/// call.
pub struct Encoder {
    encoding: &'static Encoding,
    variant: VariantEncoder,
}

impl Encoder {
    fn new(enc: &'static Encoding, encoder: VariantEncoder) -> Encoder {
        Encoder {
            encoding: enc,
            variant: encoder,
        }
    }

    /// The `Encoding` this `Encoder` is for.
    pub fn encoding(&self) -> &'static Encoding {
        self.encoding
    }

    /// Query the worst-case output size when encoding from UTF-16 without
    /// replacement.
    ///
    /// Returns the size of the output buffer in bytes that will not overflow
    /// given the current state of the encoder and `u16_length` number of
    /// additional input code units.
    ///
    /// Available via the C wrapper.
    pub fn max_buffer_length_from_utf16_without_replacement(&self, u16_length: usize) -> usize {
        self.variant.max_buffer_length_from_utf16_without_replacement(u16_length)
    }

    /// Query the worst-case output size when encoding from UTF-8 without
    /// replacement.
    ///
    /// Returns the size of the output buffer in bytes that will not overflow
    /// given the current state of the encoder and `byte_length` number of
    /// additional input code units.
    ///
    /// Available via the C wrapper.
    pub fn max_buffer_length_from_utf8_without_replacement(&self, byte_length: usize) -> usize {
        self.variant.max_buffer_length_from_utf8_without_replacement(byte_length)
    }

    /// Query the worst-case output size when encoding from UTF-16 with
    /// replacement.
    ///
    /// Returns the size of the output buffer in bytes that will not overflow
    /// given the current state of the encoder and `u16_length` number of
    /// additional input code units.
    ///
    /// Available via the C wrapper.
    pub fn max_buffer_length_from_utf16_if_no_unmappables(&self, u16_length: usize) -> usize {
        self.max_buffer_length_from_utf16_without_replacement(u16_length) +
        if self.encoding().can_encode_everything() {
            0
        } else {
            NCR_EXTRA
        }
    }

    /// Query the worst-case output size when encoding from UTF-8 with
    /// replacement.
    ///
    /// Returns the size of the output buffer in bytes that will not overflow
    /// given the current state of the encoder and `byte_length` number of
    /// additional input code units.
    ///
    /// Available via the C wrapper.
    pub fn max_buffer_length_from_utf8_if_no_unmappables(&self, byte_length: usize) -> usize {
        self.max_buffer_length_from_utf8_without_replacement(byte_length) +
        if self.encoding().can_encode_everything() {
            0
        } else {
            NCR_EXTRA
        }
    }

    /// Incrementally encode into byte stream from UTF-16.
    ///
    /// See the documentation of the struct for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn encode_from_utf16_without_replacement(&mut self,
                                                 src: &[u16],
                                                 dst: &mut [u8],
                                                 last: bool)
                                                 -> (EncoderResult, usize, usize) {
        self.variant.encode_from_utf16_raw(src, dst, last)
    }

    /// Incrementally encode into byte stream from UTF-8.
    ///
    /// See the documentation of the struct for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn encode_from_utf8_without_replacement(&mut self,
                                                src: &str,
                                                dst: &mut [u8],
                                                last: bool)
                                                -> (EncoderResult, usize, usize) {
        self.variant.encode_from_utf8_raw(src, dst, last)
    }

    /// Incrementally encode into byte stream from UTF-16 with replacement.
    ///
    /// See the documentation of the struct for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn encode_from_utf16(&mut self,
                             src: &[u16],
                             dst: &mut [u8],
                             last: bool)
                             -> (CoderResult, usize, usize, bool) {
        let effective_dst_len = dst.len() -
                                if self.encoding().can_encode_everything() {
            0
        } else {
            NCR_EXTRA
        };
        let mut had_unmappables = false;
        let mut total_read = 0usize;
        let mut total_written = 0usize;
        loop {
            let (result, read, written) = self.encode_from_utf16_without_replacement(&src[total_read..],
                                   &mut dst[total_written..effective_dst_len],
                                   last);
            total_read += read;
            total_written += written;
            match result {
                EncoderResult::InputEmpty => {
                    return (CoderResult::InputEmpty, total_read, total_written, had_unmappables);
                }
                EncoderResult::OutputFull => {
                    return (CoderResult::OutputFull, total_read, total_written, had_unmappables);
                }
                EncoderResult::Unmappable(unmappable) => {
                    had_unmappables = true;
                    debug_assert!(dst.len() - total_written >= NCR_EXTRA + 1);
                    // There are no UTF-16 encoders and even if there were,
                    // they'd never have unmappables.
                    debug_assert!(self.encoding() != UTF_16BE);
                    debug_assert!(self.encoding() != UTF_16LE);
                    // Additionally, Iso2022JpEncoder is responsible for
                    // transitioning to ASCII when returning with Unmappable
                    // from the jis0208 state. That is, when we encode
                    // ISO-2022-JP and come here, the encoder is in either the
                    // ASCII or the Roman state. We are allowed to generate any
                    // printable ASCII excluding \ and ~.
                    total_written += write_ncr(unmappable, &mut dst[total_written..]);
                }
            }
        }
    }

    /// Incrementally encode into byte stream from UTF-8 with replacement.
    ///
    /// See the documentation of the struct for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn encode_from_utf8(&mut self,
                            src: &str,
                            dst: &mut [u8],
                            last: bool)
                            -> (CoderResult, usize, usize, bool) {
        let effective_dst_len = dst.len() -
                                if self.encoding().can_encode_everything() {
            0
        } else {
            NCR_EXTRA
        };
        let mut had_unmappables = false;
        let mut total_read = 0usize;
        let mut total_written = 0usize;
        loop {
            let (result, read, written) = self.encode_from_utf8_without_replacement(&src[total_read..],
                                  &mut dst[total_written..effective_dst_len],
                                  last);
            total_read += read;
            total_written += written;
            match result {
                EncoderResult::InputEmpty => {
                    return (CoderResult::InputEmpty, total_read, total_written, had_unmappables);
                }
                EncoderResult::OutputFull => {
                    return (CoderResult::OutputFull, total_read, total_written, had_unmappables);
                }
                EncoderResult::Unmappable(unmappable) => {
                    had_unmappables = true;
                    debug_assert!(dst.len() - total_written >= NCR_EXTRA + 1);
                    debug_assert!(self.encoding() != UTF_16BE);
                    debug_assert!(self.encoding() != UTF_16LE);
                    // Additionally, Iso2022JpEncoder is responsible for
                    // transitioning to ASCII when returning with Unmappable.
                    total_written += write_ncr(unmappable, &mut dst[total_written..]);
                    if total_written >= effective_dst_len {
                        return (CoderResult::OutputFull,
                                total_read,
                                total_written,
                                had_unmappables);
                    }
                }
            }
        }
    }

    /// Incrementally encode into byte stream from UTF-8 with replacement.
    ///
    /// See the documentation of the struct for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn encode_from_utf8_to_vec(&mut self,
                                   src: &str,
                                   dst: &mut Vec<u8>,
                                   last: bool)
                                   -> (CoderResult, usize, bool) {
        unsafe {
            let old_len = dst.len();
            let capacity = dst.capacity();
            dst.set_len(capacity);
            let (result, read, written, replaced) =
                self.encode_from_utf8(src, &mut dst[old_len..], last);
            dst.set_len(old_len + written);
            (result, read, replaced)
        }
    }
}

/// Format an unmappable as NCR without heap allocation.
fn write_ncr(unmappable: char, dst: &mut [u8]) -> usize {
    // len is the number of decimal digits needed to represent unmappable plus
    // 3 (the length of "&#" and ";").
    let mut number = unmappable as u32;
    let len = if number >= 1000000u32 {
        10usize
    } else if number >= 100000u32 {
        9usize
    } else if number >= 10000u32 {
        8usize
    } else if number >= 1000u32 {
        7usize
    } else if number >= 100u32 {
        6usize
    } else {
        // Review the outcome of https://github.com/whatwg/encoding/issues/15
        // to see if this case is possible
        5usize
    };
    debug_assert!(number >= 10u32);
    debug_assert!(len <= dst.len());
    let mut pos = len - 1;
    dst[pos] = b';';
    pos -= 1;
    loop {
        let rightmost = number % 10;
        dst[pos] = rightmost as u8 + b'0';
        pos -= 1;
        if number < 10 {
            break;
        }
        number /= 10;
    }
    dst[1] = b'#';
    dst[0] = b'&';
    len
}

// ############## TESTS ###############

#[cfg(test)]
mod tests {
    use super::*;

    fn sniff_to_utf16(initial_encoding: &'static Encoding,
                      expected_encoding: &'static Encoding,
                      bytes: &[u8],
                      expect: &[u16],
                      breaks: &[usize]) {
        let mut decoder = initial_encoding.new_decoder();

        let mut dest: Vec<u16> = Vec::with_capacity(decoder.max_utf16_buffer_length(bytes.len()));
        let capacity = dest.capacity();
        dest.resize(capacity, 0u16);

        let mut total_written = 0usize;
        let mut start = 0usize;
        for br in breaks {
            let (result, read, written, _) =
                decoder.decode_to_utf16(&bytes[start..*br], &mut dest[total_written..], false);
            total_written += written;
            assert_eq!(read, *br - start);
            match result {
                CoderResult::InputEmpty => {}
                CoderResult::OutputFull => {
                    unreachable!();
                }
            }
            start = *br;
        }
        let (result, read, written, _) =
            decoder.decode_to_utf16(&bytes[start..], &mut dest[total_written..], true);
        total_written += written;
        match result {
            CoderResult::InputEmpty => {}
            CoderResult::OutputFull => {
                unreachable!();
            }
        }
        assert_eq!(read, bytes.len() - start);
        assert_eq!(total_written, expect.len());
        assert_eq!(&dest[..total_written], expect);
        assert_eq!(decoder.encoding(), expected_encoding);
    }

    // Any copyright to the test code below this comment is dedicated to the
    // Public Domain. http://creativecommons.org/publicdomain/zero/1.0/

    #[test]
    fn test_bom_sniffing() {
        // ASCII
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[]);
        // UTF-8
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[1]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[2]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[3]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[4]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[2, 3]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[1, 2]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[1, 3]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[1, 2, 3, 4]);
        sniff_to_utf16(WINDOWS_1252, UTF_8, b"\xEF\xBB\xBF", &[], &[]);
        // Not UTF-8
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xEF\xBB\x61\x62",
                       &[0x00EFu16, 0x00BBu16, 0x0061u16, 0x0062u16],
                       &[]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xEF\xBB\x61\x62",
                       &[0x00EFu16, 0x00BBu16, 0x0061u16, 0x0062u16],
                       &[1]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xEF\x61\x62",
                       &[0x00EFu16, 0x0061u16, 0x0062u16],
                       &[]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xEF\x61\x62",
                       &[0x00EFu16, 0x0061u16, 0x0062u16],
                       &[1]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xEF\xBB",
                       &[0x00EFu16, 0x00BBu16],
                       &[]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xEF\xBB",
                       &[0x00EFu16, 0x00BBu16],
                       &[1]);
        sniff_to_utf16(WINDOWS_1252, WINDOWS_1252, b"\xEF", &[0x00EFu16], &[]);
        // Not UTF-16
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xFE\x61\x62",
                       &[0x00FEu16, 0x0061u16, 0x0062u16],
                       &[]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xFE\x61\x62",
                       &[0x00FEu16, 0x0061u16, 0x0062u16],
                       &[1]);
        sniff_to_utf16(WINDOWS_1252, WINDOWS_1252, b"\xFE", &[0x00FEu16], &[]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xFF\x61\x62",
                       &[0x00FFu16, 0x0061u16, 0x0062u16],
                       &[]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xFF\x61\x62",
                       &[0x00FFu16, 0x0061u16, 0x0062u16],
                       &[1]);
        sniff_to_utf16(WINDOWS_1252, WINDOWS_1252, b"\xFF", &[0x00FFu16], &[]);
        // UTF-16
        sniff_to_utf16(WINDOWS_1252, UTF_16BE, b"\xFE\xFF", &[], &[]);
        sniff_to_utf16(WINDOWS_1252, UTF_16BE, b"\xFE\xFF", &[], &[1]);
        sniff_to_utf16(WINDOWS_1252, UTF_16LE, b"\xFF\xFE", &[], &[]);
        sniff_to_utf16(WINDOWS_1252, UTF_16LE, b"\xFF\xFE", &[], &[1]);
    }

    #[test]
    fn test_output_encoding() {
        assert_eq!(REPLACEMENT.output_encoding(), UTF_8);
        assert_eq!(UTF_16BE.output_encoding(), UTF_8);
        assert_eq!(UTF_16LE.output_encoding(), UTF_8);
        assert_eq!(UTF_8.output_encoding(), UTF_8);
        assert_eq!(WINDOWS_1252.output_encoding(), WINDOWS_1252);
        assert_eq!(REPLACEMENT.new_encoder().encoding(), UTF_8);
        assert_eq!(UTF_16BE.new_encoder().encoding(), UTF_8);
        assert_eq!(UTF_16LE.new_encoder().encoding(), UTF_8);
        assert_eq!(UTF_8.new_encoder().encoding(), UTF_8);
        assert_eq!(WINDOWS_1252.new_encoder().encoding(), WINDOWS_1252);
    }
}
