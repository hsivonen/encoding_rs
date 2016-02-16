#!/usr/bin/python

# Copyright 2013-2016 Mozilla Foundation. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import json
import subprocess

class Label:
  def __init__(self, label, preferred):
    self.label = label
    self.preferred = preferred
  def __cmp__(self, other):
    return cmp(self.label, other.label)

# If a multi-byte encoding is on this list, it is assumed to have a
# non-generated implementation class
MULTI_BYTE_IMPLEMENTED = [
  u"big5",
]

preferred = []

dom = []

labels = []

data = json.load(open("../encoding/encodings.json", "r"))

indexes = json.load(open("../encoding/indexes.json", "r"))

single_byte = []

multi_byte = []

def to_camel_name(name):
  if name == u"iso-8859-8-i":
    return u"Iso8I"
  if name.startswith(u"iso-8859-"):
    return name.replace(u"iso-8859-", u"Iso")
  return name.title().replace(u"X-", u"").replace(u"-", u"").replace(u"_", u"")

def to_constant_name(name):
  return name.replace(u"-", u"_").upper()

def to_snake_name(name):
  return name.replace(u"-", u"_").lower()

def to_dom_name(name):
  if name == u"big5":
    return u"Big5"
  if name == u"shift_jis":
    return u"Shift_JIS"
  if name == u"gbk":
    return u"GBK"
  if name.startswith(u"iso-"):
    return name.upper()
  if name.startswith(u"utf-"):
    return name.upper()
  if name.startswith(u"euc-"):
    return name.upper()
  if name.startswith(u"koi"):
    return name.upper()
  if name.startswith(u"ibm"):
    return name.upper()
  return name

# 

for group in data:
  if group["heading"] == "Legacy single-byte encodings":
    single_byte = group["encodings"]
  else:
    multi_byte.extend(group["encodings"])
  for encoding in group["encodings"]:
    preferred.append(encoding["name"])
    for label in encoding["labels"]:
      labels.append(Label(label, encoding["name"]))

for name in preferred:
  dom.append(to_dom_name(name))

preferred.sort()
labels.sort()
dom.sort()

longest_label_length = 0
longest_name_length = 0
longest_label = None
longest_name = None

for name in preferred:
  if len(name) > longest_name_length:
    longest_name_length = len(name)
    longest_name = name

for label in labels:
  if len(label.label) > longest_label_length:
    longest_label_length = len(label.label)
    longest_label = label.label

def is_single_byte(name):
  for encoding in single_byte:
    if name == encoding["name"]:
      return True
  return False

def read_non_generated(path):
  partially_generated_file = open(path, "r")
  full = partially_generated_file.read()
  partially_generated_file.close()

  generated_begin = "// BEGIN GENERATED CODE. PLEASE DO NOT EDIT."
  generated_end = "// END GENERATED CODE"

  generated_begin_index = full.find(generated_begin)
  if generated_begin_index < 0:
    print "Can't find generated code start marker in lib.rs. Exiting."
    sys.exit(-1)
  generated_end_index = full.find(generated_end)
  if generated_end_index < 0:
    print "Can't find generated code end marker in lib.rs. Exiting."
    sys.exit(-1)

  return (full[0:generated_begin_index + len(generated_begin)],
          full[generated_end_index:])

(lib_rs_begin, lib_rs_end) = read_non_generated("src/lib.rs")

label_file = open("src/lib.rs", "w")

label_file.write(lib_rs_begin)
label_file.write("""
// Instead, please regenerate using generate-encoding-data.py

const LONGEST_LABEL_LENGTH: usize = %d; // %s

const LONGEST_NAME_LENGTH: usize = %d; // %s

""" % (longest_label_length, longest_label, longest_name_length, longest_name))

for name in preferred:
  variant = None
  if is_single_byte(name):
    variant = "SingleByte(data::%s_DATA)" % to_constant_name(u"iso-8859-8" if name == u"iso-8859-8-i" else name)
  else:
    variant = to_camel_name(name)

  label_file.write('''/// The %s encoding.
pub const %s: &'static Encoding = &Encoding {
    name: "%s",
    variant: VariantEncoding::%s,
};

''' % (to_dom_name(name), to_constant_name(name), to_dom_name(name), variant))

label_file.write("""static ENCODINGS_SORTED_BY_NAME: [&'static Encoding; %d] = [
""" % len(dom))

for dom_name in dom:
  label_file.write("%s,\n" % to_constant_name(dom_name))

label_file.write("""];

static LABELS_SORTED: [&'static str; %d] = [
""" % len(labels))

for label in labels:
  label_file.write('''"%s",\n''' % label.label)
  
label_file.write("""];

static ENCODINGS_IN_LABEL_SORT: [&'static Encoding; %d] = [
""" % len(labels))

for label in labels:
  label_file.write('''%s,\n''' % to_constant_name(label.preferred))
  
label_file.write('''];

''')
label_file.write(lib_rs_end)
label_file.close()

def null_to_zero(code_point):
  if not code_point:
    code_point = 0
  return code_point

data_file = open("src/data.rs", "w")
data_file.write('''// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// THIS IS A GENERATED FILE. PLEASE DO NOT EDIT.
// Instead, please regenerate using generate-encoding-data.py

''')

# Single-byte

for encoding in single_byte:
  name = encoding["name"]
  if name == u"iso-8859-8-i":
    continue

  data_file.write('''pub const %s_DATA: &'static [u16; 128] = &[
''' % to_constant_name(name))

  for code_point in indexes[name]:
    data_file.write('0x%04X,\n' % null_to_zero(code_point))

  data_file.write('''];

''')

# Big5

index = []

for code_point in indexes["big5"]:
  index.append(null_to_zero(code_point))  

index_first = 0

for i in xrange(len(index)):
  if index[i]:
    index_first = i
    break

bits = []
for code_point in index:
  bits.append(1 if code_point > 0xFFFF else 0)

bits_cap = len(bits)

bits_first = 0
for i in xrange(len(bits)):
  if bits[i]:
    bits_first = i
    break

# pad length to multiple of 32
for j in xrange(32 - ((len(bits) - bits_first) % 32)):
  bits.append(0)

data_file.write('''static ASTRALNESS: [u32; %d] = [
''' % ((len(bits) - bits_first) / 32))

i = bits_first
while i < len(bits):
  accu = 0
  for j in xrange(32):
    accu |= bits[i + j] << j
  data_file.write('0x%08X,\n' % accu)
  i += 32

data_file.write('''];

static LOW_BITS: [u16; %d] = [
''' % (len(index) - index_first))

for i in xrange(index_first, len(index)):
  data_file.write('0x%04X,\n' % (index[i] & 0xFFFF))

data_file.write('''];

#[inline(always)]
pub fn big5_is_astral(pointer: usize) -> bool {
    let i = pointer.wrapping_sub(%d);
    if i < %d {
        (ASTRALNESS[i >> 5] & (1 << (i & 0x1F))) != 0
    } else {
        false
    }
}

#[inline(always)]
pub fn big5_low_bits(pointer: usize) -> u16 {
    let i = pointer.wrapping_sub(%d);
    if i < %d {
        LOW_BITS[i]
    } else {
        0
    }
}
''' % (bits_first, bits_cap - bits_first, index_first, len(index) - index_first))

data_file.write('''
#[inline(always)]
pub fn big5_find_pointer(low_bits: u16, is_astral: bool) -> usize {
    if !is_astral {
        match low_bits {
''')

hkscs_bound = (0xA1 - 0x81) * 157

hkscs_start_index = hkscs_bound -  index_first

prefer_last = [
  0x2550,
  0x255E,
  0x2561,
  0x256A,
  0x5341,
  0x5345,
]

for code_point in prefer_last:
  # Python lists don't have .rindex() :-(
  for i in xrange(len(index) - 1, -1, -1):
    candidate = index[i]
    if candidate == code_point:
       data_file.write('''0x%04X => {
   return %d;
},
''' % (code_point, i))
       break

data_file.write('''_ => {},
        }
    }
    let mut it = LOW_BITS[%d..].iter().enumerate();
    loop {
        match it.next() {
            Some((i, bits)) => {
                if *bits != low_bits {
                    continue;
                }
                let pointer = i + %d;
                if is_astral == big5_is_astral(pointer) {
                    return pointer;
                }
            },
            None => {
                    return 0;
            }
        }
    }
}
''' % (hkscs_start_index, hkscs_bound))
data_file.close()

# Variant

variant_file = open("src/variant.rs", "w")
variant_file.write('''// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// THIS IS A GENERATED FILE. PLEASE DO NOT EDIT.
// Instead, please regenerate using generate-encoding-data.py

//! This module provides enums that wrap the various decoders and encoders.
//! The purpose is to make `Decoder` and `Encoder` `Sized` by writing the
//! dispatch explicitly for a finite set of specialized decoders and encoders.
//! Unfortunately, this means the compiler doesn't generate the dispatch code
//! and it has to be written here instead.
//!
//! The purpose of making `Decoder` and `Encoder` `Sized` is to allow stack
//! allocation in Rust code, including the convenience methods on `Encoding`.

''')

encoding_variants = [u"single-byte",]
for encoding in multi_byte:
  if encoding["name"] in [u"utf-16le", u"utf-16be"]:
    continue
  else:
    encoding_variants.append(encoding["name"])
encoding_variants.append(u"utf-16")

decoder_variants = []
for variant in encoding_variants:
  if variant == u"gbk":
    continue
  decoder_variants.append(variant)

encoder_variants = []
for variant in encoding_variants:
  if variant in [u"replacement", u"gbk"]:
    continue
  encoder_variants.append(variant)

for variant in decoder_variants:
  variant_file.write("use %s::*;\n" % to_snake_name(variant))

variant_file.write('''use super::*;

pub enum VariantDecoder {
''')

for variant in decoder_variants:
  variant_file.write("   %s(%sDecoder),\n" % (to_camel_name(variant), to_camel_name(variant)))

variant_file.write('''}

impl VariantDecoder {
''')

def write_variant_method(name, mut, arg_list, ret, variants, excludes, kind):
  variant_file.write('''pub fn %s(&''' % name)
  if mut:
    variant_file.write('''mut ''')
  variant_file.write('''self''')
  for arg in arg_list:
    variant_file.write(''', %s: %s''' % (arg[0], arg[1]))
  variant_file.write(''')''')
  if ret:
    variant_file.write(''' -> %s''' % ret)  
  variant_file.write(''' {\nmatch self {\n''')
  for variant in variants:
    variant_file.write('''&''')
    if mut:
      variant_file.write('''mut ''')
    variant_file.write('''Variant%s::%s(ref ''' % (kind, to_camel_name(variant)))
    if mut:
      variant_file.write('''mut ''')
    if variant in excludes:
      variant_file.write('''v) => (),''')
      continue
    variant_file.write('''v) => v.%s(''' % name)
    first = True
    for arg in arg_list:
      if not first:
        variant_file.write(''', ''')
      first = False
      variant_file.write(arg[0])
    variant_file.write('''),\n''')
  variant_file.write('''}\n}\n\n''')

write_variant_method("reset", True, [], None, decoder_variants, [u"single-byte"], "Decoder")

write_variant_method("max_utf16_buffer_length", False, [("u16_length", "usize")], "usize", decoder_variants, [], "Decoder")

write_variant_method("max_utf8_buffer_length", False, [("byte_length", "usize")], "usize", decoder_variants, [], "Decoder")

write_variant_method("max_utf8_buffer_length_with_replacement", False, [("byte_length", "usize")], "usize", decoder_variants, [], "Decoder")

write_variant_method("decode_to_utf16", True, [("src", "&[u8]"),
                           ("dst", "&mut [u16]"),
                           ("last", "bool")], "(DecoderResult, usize, usize)", decoder_variants, [], "Decoder")

write_variant_method("decode_to_utf8", True, [("src", "&[u8]"),
                           ("dst", "&mut [u8]"),
                           ("last", "bool")], "(DecoderResult, usize, usize)", decoder_variants, [], "Decoder")

variant_file.write('''
}

pub enum VariantEncoder {
''')

for variant in encoder_variants:
  variant_file.write("   %s(%sEncoder),\n" % (to_camel_name(variant), to_camel_name(variant)))

variant_file.write('''}

impl VariantEncoder {
    pub fn reset(&mut self) {}

''')

write_variant_method("max_buffer_length_from_utf16", False, [("u16_length", "usize")], "usize", encoder_variants, [], "Encoder")

write_variant_method("max_buffer_length_from_utf8", False, [("byte_length", "usize")], "usize", encoder_variants, [], "Encoder")

write_variant_method("encode_from_utf16", True, [("src", "&[u16]"),
                           ("dst", "&mut [u8]"),
                           ("last", "bool")], "(EncoderResult, usize, usize)", encoder_variants, [], "Encoder")

write_variant_method("encode_from_utf8", True, [("src", "&str"),
                           ("dst", "&mut [u8]"),
                           ("last", "bool")], "(EncoderResult, usize, usize)", encoder_variants, [], "Encoder")


variant_file.write('''}

pub enum VariantEncoding {
    SingleByte(&'static [u16; 128]),''')

for encoding in multi_byte:
  variant_file.write("%s,\n" % to_camel_name(encoding["name"]))

variant_file.write('''}

impl VariantEncoding {
    pub fn new_decoder(&self, encoding: &'static Encoding) -> Decoder {
        match self {
            &VariantEncoding::SingleByte(table) => SingleByteDecoder::new(encoding, table),
            &VariantEncoding::Utf8 => Utf8Decoder::new(encoding),
            &VariantEncoding::Gbk | &VariantEncoding::Gb18030 => Gb18030Decoder::new(encoding),
            &VariantEncoding::Big5 => Big5Decoder::new(encoding),
            &VariantEncoding::EucJp => EucJpDecoder::new(encoding),
            &VariantEncoding::Iso2022Jp => Iso2022JpDecoder::new(encoding),
            &VariantEncoding::ShiftJis => ShiftJisDecoder::new(encoding),
            &VariantEncoding::EucKr => EucKrDecoder::new(encoding),
            &VariantEncoding::Replacement => ReplacementDecoder::new(encoding),
            &VariantEncoding::UserDefined => UserDefinedDecoder::new(encoding),
            &VariantEncoding::Utf16Be => Utf16Decoder::new(encoding, true),
            &VariantEncoding::Utf16Le => Utf16Decoder::new(encoding, false),
        }
    }

    pub fn new_encoder(&self, encoding: &'static Encoding) -> Encoder {
        match self {
            &VariantEncoding::SingleByte(table) => SingleByteEncoder::new(encoding, table),
            &VariantEncoding::Utf8 => Utf8Encoder::new(encoding),
            &VariantEncoding::Gbk => Gb18030Encoder::new(encoding, false),
            &VariantEncoding::Gb18030 => Gb18030Encoder::new(encoding, true),
            &VariantEncoding::Big5 => Big5Encoder::new(encoding),
            &VariantEncoding::EucJp => EucJpEncoder::new(encoding),
            &VariantEncoding::Iso2022Jp => Iso2022JpEncoder::new(encoding),
            &VariantEncoding::ShiftJis => ShiftJisEncoder::new(encoding),
            &VariantEncoding::EucKr => EucKrEncoder::new(encoding),
            &VariantEncoding::Replacement => Utf8Encoder::new(UTF_8),
            &VariantEncoding::UserDefined => UserDefinedEncoder::new(encoding),
            &VariantEncoding::Utf16Be => Utf16Encoder::new(encoding, true),
            &VariantEncoding::Utf16Le => Utf16Encoder::new(encoding, false),
        }
    }

    pub fn can_encode_everything(&self) -> bool {
        match self {
            &VariantEncoding::Utf8 => true,
            &VariantEncoding::Gb18030 => true,
            &VariantEncoding::Utf16Be => true,
            &VariantEncoding::Utf16Le => true,
            _ => false,
        }
    }
}
''')

variant_file.close()

(ffi_rs_begin, ffi_rs_end) = read_non_generated("src/ffi.rs")

ffi_file = open("src/ffi.rs", "w")

ffi_file.write(ffi_rs_begin)
ffi_file.write("""
// Instead, please regenerate using generate-encoding-data.py

""")

for name in preferred:
  ffi_file.write('''/// The %s encoding.
#[no_mangle]
pub static %s_ENCODING: *const Encoding = %s;

''' % (to_dom_name(name), to_constant_name(name), to_constant_name(name)))

ffi_file.write(ffi_rs_end)
ffi_file.close()

static_file = open("include/encoding_rs_statics.h", "w")

static_file.write("""// Copyright 2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// THIS IS A GENERATED FILE. PLEASE DO NOT EDIT.
// Instead, please regenerate using generate-encoding-data.py

// This file is not meant to be included directly. Instead, encoding_rs.h
// includes this file.

#ifndef encoding_rs_statics_h_
#define encoding_rs_statics_h_

#include <uchar.h>

#ifndef __cplusplus
typedef struct Encoding_ Encoding;
typedef struct Decoder_ Decoder;
typedef struct Encoder_ Encoder;
#endif

#define INPUT_EMPTY 0

#define OUTPUT_FULL 0xFFFFFFFF

// %s
#define ENCODING_NAME_MAX_LENGTH %d

""" % (longest_name, longest_name_length))

for name in preferred:
  static_file.write('''/// The %s encoding.
extern const Encoding* const %s_ENCODING;

''' % (to_dom_name(name), to_constant_name(name)))

static_file.write("""#endif // encoding_rs_statics_h_
""")
static_file.close()

subprocess.call(["cargo", "fmt"])
