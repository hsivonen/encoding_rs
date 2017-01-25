#!/usr/bin/python

# Copyright 2013-2016 Mozilla Foundation. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import json
import subprocess
import sys

class Label:
  def __init__(self, label, preferred):
    self.label = label
    self.preferred = preferred
  def __cmp__(self, other):
    return cmp(self.label, other.label)

def static_u16_table(name, data):
  data_file.write('''pub static %s: [u16; %d] = [
  ''' % (name, len(data)))

  for i in xrange(len(data)):
    data_file.write('0x%04X,\n' % data[i])

  data_file.write('''];

  ''')

def static_u16_table_from_indexable(name, data, item):
  data_file.write('''#[cfg(not(feature = "no-static-ideograph-encoder-tables"))]
static %s: [u16; %d] = [
  ''' % (name, len(data)))

  for i in xrange(len(data)):
    data_file.write('0x%04X,\n' % data[i][item])

  data_file.write('''];

  ''')

def static_u8_pair_table_from_indexable(name, data, item):
  data_file.write('''#[cfg(not(feature = "no-static-ideograph-encoder-tables"))]
static %s: [[u8; 2]; %d] = [
  ''' % (name, len(data)))

  for i in xrange(len(data)):
    data_file.write('[0x%02X, 0x%02X],\n' % data[i][item])

  data_file.write('''];

  ''')

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
    print "Can't find generated code start marker in %s. Exiting." % path
    sys.exit(-1)
  generated_end_index = full.find(generated_end)
  if generated_end_index < 0:
    print "Can't find generated code end marker in %s. Exiting." % path
    sys.exit(-1)

  return (full[0:generated_begin_index + len(generated_begin)],
          full[generated_end_index:])

(lib_rs_begin, lib_rs_end) = read_non_generated("src/lib.rs")

label_file = open("src/lib.rs", "w")

label_file.write(lib_rs_begin)
label_file.write("""
// Instead, please regenerate using generate-encoding-data.py

const LONGEST_LABEL_LENGTH: usize = %d; // %s

""" % (longest_label_length, longest_label))

for name in preferred:
  variant = None
  if is_single_byte(name):
    variant = "SingleByte(data::%s_DATA)" % to_constant_name(u"iso-8859-8" if name == u"ISO-8859-8-I" else name)
  else:
    variant = to_camel_name(name)

  label_file.write('''/// The initializer for the %s encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s. If in doubt,
/// use the corresponding non-`_INIT` reference-typed `static`.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static %s_INIT: Encoding = Encoding {
    name: "%s",
    variant: VariantEncoding::%s,
};

/// The %s encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static %s: &'static Encoding = &%s_INIT;

''' % (to_dom_name(name), to_constant_name(name), to_dom_name(name), variant, to_dom_name(name), to_constant_name(name), to_constant_name(name)))

label_file.write("""static ENCODINGS_SORTED_BY_NAME: [&'static Encoding; %d] = [
""" % len(dom))

for dom_name in dom:
  label_file.write("&%s_INIT,\n" % to_constant_name(dom_name))

label_file.write("""];

static LABELS_SORTED: [&'static str; %d] = [
""" % len(labels))

for label in labels:
  label_file.write('''"%s",\n''' % label.label)

label_file.write("""];

static ENCODINGS_IN_LABEL_SORT: [&'static Encoding; %d] = [
""" % len(labels))

for label in labels:
  label_file.write('''&%s_INIT,\n''' % to_constant_name(label.preferred))

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
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// THIS IS A GENERATED FILE. PLEASE DO NOT EDIT.
// Instead, please regenerate using generate-encoding-data.py

''')

# Single-byte

for encoding in single_byte:
  name = encoding["name"]
  if name == u"ISO-8859-8-I":
    continue

  data_file.write('''pub const %s_DATA: &'static [u16; 128] = &[
''' % to_constant_name(name))

  for code_point in indexes[name.lower()]:
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

pointer_for_prefer_last = []

for code_point in prefer_last:
  # Python lists don't have .rindex() :-(
  for i in xrange(len(index) - 1, -1, -1):
    candidate = index[i]
    if candidate == code_point:
       data_file.write('''0x%04X => {
   return %d;
},
''' % (code_point, i))
       pointer_for_prefer_last.append(i)
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

# JIS0208

index = indexes["jis0208"]

# JIS 0208 Level 1 Kanji
static_u16_table("JIS0208_LEVEL1_KANJI", index[1410:4375])

# JIS 0208 Level 2 Kanji and Additional Kanji
static_u16_table("JIS0208_LEVEL2_AND_ADDITIONAL_KANJI", index[4418:7808])

# IBM Kanji
static_u16_table("IBM_KANJI", index[8272:8632])

# Check that the other instance is the same
if index[8272:8632] != index[10744:11104]:
  raise Error()

# JIS 0208 symbols (all non-Kanji, non-range items)
symbol_index = []
symbol_triples = []
pointers_to_scan = [
  (0, 188),
  (658, 691),
  (1159, 1221),
]
in_run = False
run_start_pointer = 0
run_start_array_index = 0
for (start, end) in pointers_to_scan:
  for i in range(start, end):
    code_point = index[i]
    if in_run:
      if code_point:
        symbol_index.append(code_point)
      else:
        symbol_triples.append(run_start_pointer)
        symbol_triples.append(i - run_start_pointer)
        symbol_triples.append(run_start_array_index)
        in_run = False
    else:
      if code_point:
        in_run = True
        run_start_pointer = i
        run_start_array_index = len(symbol_index)
        symbol_index.append(code_point)
  if in_run:
    symbol_triples.append(run_start_pointer)
    symbol_triples.append(end - run_start_pointer)
    symbol_triples.append(run_start_array_index)
    in_run = False
if in_run:
  raise Error()

# Now add manually the two overlapping slices of
# index from the NEC/IBM extensions.
run_start_array_index = len(symbol_index)
symbol_index.extend(index[10736:10744])
# Later
symbol_triples.append(10736)
symbol_triples.append(8)
symbol_triples.append(run_start_array_index)
# Earlier
symbol_triples.append(8644)
symbol_triples.append(4)
symbol_triples.append(run_start_array_index)

static_u16_table("JIS0208_SYMBOLS", symbol_index)
static_u16_table("JIS0208_SYMBOL_TRIPLES", symbol_triples)

# Write down the magic numbers needed when preferring the earlier case
data_file.write('''const IBM_SYMBOL_START: usize = %d;''' % (run_start_array_index + 1))
data_file.write('''const IBM_SYMBOL_END: usize = %d;''' % (run_start_array_index + 4))
data_file.write('''const IBM_SYMBOL_POINTER_START: usize = %d;''' % 8645)

# JIS 0208 ranges (excluding kana)
range_triples = []
pointers_to_scan = [
  (188, 281),
  (470, 657),
  (1128, 1159),
  (8634, 8644),
  (10716, 10736),
]
in_run = False
run_start_pointer = 0
run_start_code_point = 0
previous_code_point = 0
for (start, end) in pointers_to_scan:
  for i in range(start, end):
    code_point = index[i]
    if in_run:
      if code_point:
        if previous_code_point + 1 != code_point:
          range_triples.append(run_start_pointer)
          range_triples.append(i - run_start_pointer)
          range_triples.append(run_start_code_point)
          run_start_pointer = i
          run_start_code_point = code_point
        previous_code_point = code_point
      else:
          range_triples.append(run_start_pointer)
          range_triples.append(i - run_start_pointer)
          range_triples.append(run_start_code_point)
          run_start_pointer = 0
          run_start_code_point = 0
          previous_code_point = 0
          in_run = False
    else:
      if code_point:
        in_run = True
        run_start_pointer = i
        run_start_code_point = code_point
        previous_code_point = code_point
  if in_run:
    range_triples.append(run_start_pointer)
    range_triples.append(end - run_start_pointer)
    range_triples.append(run_start_code_point)
    run_start_pointer = 0
    run_start_code_point = 0
    previous_code_point = 0
    in_run = False
if in_run:
  raise Error()

static_u16_table("JIS0208_RANGE_TRIPLES", range_triples)

# Encoder table for Level 1 Kanji
# Note: If we were OK with 30 KB more footprint, we
# could use a directly-indexable table instead...
level1_kanji_index = index[1410:4375]
level1_kanji_pairs = []
for i in xrange(len(level1_kanji_index)):
  pointer = 1410 + i
  (lead, trail) = divmod(pointer, 188)
  lead += 0x81 if lead < 0x1F else 0xC1
  trail += 0x40 if trail < 0x3F else 0x41
  level1_kanji_pairs.append((level1_kanji_index[i], (lead, trail)))
level1_kanji_pairs.sort(key=lambda x: x[0])

static_u16_table_from_indexable("JIS0208_LEVEL1_KANJI_CODE_POINTS", level1_kanji_pairs, 0)
static_u8_pair_table_from_indexable("JIS0208_LEVEL1_KANJI_SHIFT_JIS_BYTES", level1_kanji_pairs, 1)

# EUC-KR

index = indexes["euc-kr"]

# Unicode 1.1 Hangul above the old KS X 1001 block
# Compressed form takes 35% of uncompressed form
pointers = []
offsets = []
previous_code_point = 0
for row in xrange(0x20):
  for column in xrange(190):
    i = column + (row * 190)
    # Skip the gaps
    if (column >= 0x1A and column < 0x20) or (column >= 0x3A and column < 0x40):
      continue
    code_point = index[i]
    if previous_code_point > code_point:
      raise Error()
    if code_point - previous_code_point != 1:
      adjustment = 0
      if column >= 0x40:
        adjustment = 12
      elif column >= 0x20:
        adjustment = 6
      pointers.append(column - adjustment + (row * (190 - 12)))
      offsets.append(code_point)
    previous_code_point = code_point

static_u16_table("CP949_TOP_HANGUL_POINTERS", pointers)
static_u16_table("CP949_TOP_HANGUL_OFFSETS", offsets)

# Unicode 1.1 Hangul to the left of the old KS X 1001 block
pointers = []
offsets = []
previous_code_point = 0
for row in xrange(0x46 - 0x20):
  for column in xrange(190 - 94):
    i = 6080 + column + (row * 190)
    # Skip the gaps
    if (column >= 0x1A and column < 0x20) or (column >= 0x3A and column < 0x40):
      continue
    if i > 13127:
      # Exclude unassigned on partial last row
      break
    code_point = index[i]
    if previous_code_point > code_point:
      raise Error()
    if code_point - previous_code_point != 1:
      adjustment = 0
      if column >= 0x40:
        adjustment = 12
      elif column >= 0x20:
        adjustment = 6
      pointers.append(column - adjustment + (row * (190 - 94 - 12)))
      offsets.append(code_point)
    previous_code_point = code_point

static_u16_table("CP949_LEFT_HANGUL_POINTERS", pointers)
static_u16_table("CP949_LEFT_HANGUL_OFFSETS", offsets)

# KS X 1001 Hangul
hangul_index = []
previous_code_point = 0
for row in xrange(0x48 - 0x2F):
  for column in xrange(94):
    code_point = index[9026 + column + (row * 190)]
    if previous_code_point >= code_point:
      raise Error()
    hangul_index.append(code_point)
    previous_code_point = code_point

static_u16_table("KSX1001_HANGUL", hangul_index)

# KS X 1001 Hanja
hanja_index = []
for row in xrange(0x7D - 0x49):
  for column in xrange(94):
    hanja_index.append(index[13966 + column + (row * 190)])

static_u16_table("KSX1001_HANJA", hanja_index)

# KS X 1001 symbols
symbol_index = []
for i in range(6176, 6270):
  symbol_index.append(index[i])
for i in range(6366, 6437):
  symbol_index.append(index[i])

static_u16_table("KSX1001_SYMBOLS", symbol_index)

# KS X 1001 Uppercase Latin
subindex = []
for i in range(7506, 7521):
  subindex.append(null_to_zero(index[i]))

static_u16_table("KSX1001_UPPERCASE", subindex)

# KS X 1001 Lowercase Latin
subindex = []
for i in range(7696, 7712):
  subindex.append(index[i])

static_u16_table("KSX1001_LOWERCASE", subindex)

# KS X 1001 Box drawing
subindex = []
for i in range(7126, 7194):
  subindex.append(index[i])

static_u16_table("KSX1001_BOX", subindex)

# KS X 1001 other
pointers = []
offsets = []
previous_code_point = 0
for row in xrange(10):
  for column in xrange(94):
    i = 6556 + column + (row * 190)
    code_point = index[i]
    # Exclude ranges that were processed as lookup tables
    # or that contain unmapped cells by filling them with
    # ASCII. Upon encode, ASCII code points will
    # never appear as the search key.
    if (i >= 6946 and i <= 6950):
      code_point = i - 6946
    elif (i >= 6961 and i <= 6967):
      code_point = i - 6961
    elif (i >= 6992 and i <= 6999):
      code_point = i - 6992
    elif (i >= 7024 and i <= 7029):
      code_point = i - 7024
    elif (i >= 7126 and i <= 7219):
      code_point = i - 7126
    elif (i >= 7395 and i <= 7409):
      code_point = i - 7395
    elif (i >= 7506 and i <= 7521):
      code_point = i - 7506
    elif (i >= 7696 and i <= 7711):
      code_point = i - 7696
    elif (i >= 7969 and i <= 7979):
      code_point = i - 7969
    elif (i >= 8162 and i <= 8169):
      code_point = i - 8162
    elif (i >= 8299 and i <= 8313):
      code_point = i - 8299
    elif (i >= 8347 and i <= 8359):
      code_point = i - 8347
    if code_point - previous_code_point != 1:
      pointers.append(column + (row * 94))
      offsets.append(code_point)
    previous_code_point = code_point

static_u16_table("KSX1001_OTHER_POINTERS", pointers)
# Omit the last offset, because the end of the last line
# is unmapped, so we don't want to look at it.
static_u16_table("KSX1001_OTHER_UNSORTED_OFFSETS", offsets[:-1])

# JIS 0212

index = indexes["jis0212"]

# JIS 0212 Kanji
static_u16_table("JIS0212_KANJI", index[1410:7211])

# JIS 0212 accented (all non-Kanji, non-range items)
symbol_index = []
symbol_triples = []
pointers_to_scan = [
  (0, 596),
  (608, 644),
  (656, 1409),
]
in_run = False
run_start_pointer = 0
run_start_array_index = 0
for (start, end) in pointers_to_scan:
  for i in range(start, end):
    code_point = index[i]
    if in_run:
      if code_point:
        symbol_index.append(code_point)
      elif index[i + 1]:
        symbol_index.append(0)
      else:
        symbol_triples.append(run_start_pointer)
        symbol_triples.append(i - run_start_pointer)
        symbol_triples.append(run_start_array_index)
        in_run = False
    else:
      if code_point:
        in_run = True
        run_start_pointer = i
        run_start_array_index = len(symbol_index)
        symbol_index.append(code_point)
  if in_run:
    symbol_triples.append(run_start_pointer)
    symbol_triples.append(end - run_start_pointer)
    symbol_triples.append(run_start_array_index)
    in_run = False
if in_run:
  raise Error()

static_u16_table("JIS0212_ACCENTED", symbol_index)
static_u16_table("JIS0212_ACCENTED_TRIPLES", symbol_triples)

# gb18030

index = indexes["gb18030"]

# Unicode 1.1 ideographs above the old GB2312 block
# Compressed form takes 63% of uncompressed form
pointers = []
offsets = []
previous_code_point = 0
for i in xrange(6080):
  code_point = index[i]
  if previous_code_point > code_point:
    raise Error()
  if code_point - previous_code_point != 1:
    pointers.append(i)
    offsets.append(code_point)
  previous_code_point = code_point

static_u16_table("GBK_TOP_IDEOGRAPH_POINTERS", pointers)
static_u16_table("GBK_TOP_IDEOGRAPH_OFFSETS", offsets)

# Unicode 1.1 ideographs to the left of the old GB2312 block
# Compressed form takes 40% of uncompressed form
pointers = []
offsets = []
previous_code_point = 0
for row in xrange(0x7D - 0x29):
  for column in xrange(190 - 94):
    i = 7790 + column + (row * 190)
    if i > 23650:
      # Exclude compatibility ideographs at the end
      break
    code_point = index[i]
    if previous_code_point > code_point:
      raise Error()
    if code_point - previous_code_point != 1:
      pointers.append(column + (row * (190 - 94)))
      offsets.append(code_point)
    previous_code_point = code_point

static_u16_table("GBK_LEFT_IDEOGRAPH_POINTERS", pointers)
static_u16_table("GBK_LEFT_IDEOGRAPH_OFFSETS", offsets)

# GBK other (excl. Ext A, Compat & PUA at the bottom)
pointers = []
offsets = []
previous_code_point = 0
for row in xrange(0x29 - 0x20):
  for column in xrange(190 - 94):
    i = 6080 + column + (row * 190)
    code_point = index[i]
    if code_point - previous_code_point != 1:
      pointers.append(column + (row * (190 - 94)))
      offsets.append(code_point)
    previous_code_point = code_point

pointers.append((190 - 94) * (0x29 - 0x20))
static_u16_table("GBK_OTHER_POINTERS", pointers)
static_u16_table("GBK_OTHER_UNSORTED_OFFSETS", offsets)

# GBK bottom: Compatibility ideagraphs, Ext A and PUA
bottom_index = []
# 5 compat following Unified Ideographs
for i in range(23651, 23656):
  bottom_index.append(index[i])
# Last row
for i in range(23750, 23846):
  bottom_index.append(index[i])

static_u16_table("GBK_BOTTOM", bottom_index)

# GB2312 Hanzi
# (and the 5 PUA code points in between Level 1 and Level 2)
hanzi_index = []
for row in xrange(0x77 - 0x2F):
  for column in xrange(94):
    hanzi_index.append(index[9026 + column + (row * 190)])

static_u16_table("GB2312_HANZI", hanzi_index)

# GB2312 symbols
symbol_index = []
for i in xrange(94):
  symbol_index.append(index[6176 + i])

static_u16_table("GB2312_SYMBOLS", symbol_index)

# GB2312 symbols on Greek row (incl. PUA)
symbol_index = []
for i in xrange(22):
  symbol_index.append(index[7189 + i])

static_u16_table("GB2312_SYMBOLS_AFTER_GREEK", symbol_index)

# GB2312 Pinyin
pinyin_index = []
for i in xrange(32):
  pinyin_index.append(index[7506 + i])

static_u16_table("GB2312_PINYIN", pinyin_index)

# GB2312 other (excl. bottom PUA)
pointers = []
offsets = []
previous_code_point = 0
for row in xrange(14):
  for column in xrange(94):
    i = 6366 + column + (row * 190)
    code_point = index[i]
    # Exclude the two ranges that were processed as
    # lookup tables above by filling them with
    # ASCII. Upon encode, ASCII code points will
    # never appear as the search key.
    if (i >= 7189 and i < 7189 + 22):
      code_point = i - 7189
    elif (i >= 7506 and i < 7506 + 32):
      code_point = i - 7506
    if code_point - previous_code_point != 1:
      pointers.append(column + (row * 94))
      offsets.append(code_point)
    previous_code_point = code_point

pointers.append(14 * 94)
static_u16_table("GB2312_OTHER_POINTERS", pointers)
static_u16_table("GB2312_OTHER_UNSORTED_OFFSETS", offsets)

# Non-gbk code points
pointers = []
offsets = []
for pair in indexes["gb18030-ranges"]:
  if pair[1] == 0x10000:
    break # the last entry doesn't fit in u16
  pointers.append(pair[0])
  offsets.append(pair[1])

static_u16_table("GB18030_RANGE_POINTERS", pointers)
static_u16_table("GB18030_RANGE_OFFSETS", offsets)

# Encoder table for Level 1 Hanzi
# The units here really fit into 12 bits, but since we're
# looking for speed here, let's use 16 bits per unit.
# Once we use 16 bits per unit, we might as well precompute
# the output bytes.
level1_hanzi_index = hanzi_index[:(94 * (0xD8 - 0xB0) - 5)]
level1_hanzi_pairs = []
for i in xrange(len(level1_hanzi_index)):
  hanzi_lead = (i / 94) + 0xB0;
  hanzi_trail = (i % 94) + 0xA1;
  level1_hanzi_pairs.append((level1_hanzi_index[i], (hanzi_lead, hanzi_trail)))
level1_hanzi_pairs.sort(key=lambda x: x[0])

static_u16_table_from_indexable("GB2312_LEVEL1_HANZI_CODE_POINTS", level1_hanzi_pairs, 0)
static_u8_pair_table_from_indexable("GB2312_LEVEL1_HANZI_BYTES", level1_hanzi_pairs, 1)

data_file.write('''#[inline(always)]
fn map_with_ranges(haystack: &[u16], other: &[u16], needle: u16) -> u16 {
    debug_assert_eq!(haystack.len(), other.len());
    match haystack.binary_search(&needle) {
        Ok(i) => other[i],
        Err(i) => other[i - 1] + (needle - haystack[i - 1]),
    }
}

#[inline(always)]
fn map_with_unsorted_ranges(haystack: &[u16], other: &[u16], needle: u16) -> Option<u16> {
    debug_assert_eq!(haystack.len() + 1, other.len());
    for i in 0..haystack.len() {
        let start = other[i];
        let end = other[i + 1];
        let length = end - start;
        let offset = needle.wrapping_sub(haystack[i]);
        if offset < length {
            return Some(start + offset);
        }
    }
    None
}

#[inline(always)]
pub fn position(haystack: &[u16], needle: u16) -> Option<usize> {
    haystack.iter().position(|&x| x == needle)
}

#[inline(always)]
pub fn gb18030_range_decode(pointer: u16) -> u16 {
    map_with_ranges(&GB18030_RANGE_POINTERS[..],
                    &GB18030_RANGE_OFFSETS[..],
                    pointer)
}

#[inline(always)]
pub fn gb18030_range_encode(bmp: u16) -> usize {
    if bmp == 0xE7C7 {
        return 7457;
    }
    map_with_ranges(&GB18030_RANGE_OFFSETS[..], &GB18030_RANGE_POINTERS[..], bmp) as usize
}

#[inline(always)]
pub fn gbk_top_ideograph_decode(pointer: u16) -> u16 {
    map_with_ranges(&GBK_TOP_IDEOGRAPH_POINTERS[..],
                    &GBK_TOP_IDEOGRAPH_OFFSETS[..],
                    pointer)
}

#[inline(always)]
pub fn gbk_top_ideograph_encode(bmp: u16) -> u16 {
    map_with_ranges(&GBK_TOP_IDEOGRAPH_OFFSETS[..],
                    &GBK_TOP_IDEOGRAPH_POINTERS[..],
                    bmp)
}

#[inline(always)]
pub fn gbk_left_ideograph_decode(pointer: u16) -> u16 {
    map_with_ranges(&GBK_LEFT_IDEOGRAPH_POINTERS[..],
                    &GBK_LEFT_IDEOGRAPH_OFFSETS[..],
                    pointer)
}

#[inline(always)]
pub fn gbk_left_ideograph_encode(bmp: u16) -> u16 {
    map_with_ranges(&GBK_LEFT_IDEOGRAPH_OFFSETS[..],
                    &GBK_LEFT_IDEOGRAPH_POINTERS[..],
                    bmp)
}

#[inline(always)]
pub fn cp949_top_hangul_decode(pointer: u16) -> u16 {
    map_with_ranges(&CP949_TOP_HANGUL_POINTERS[..],
                    &CP949_TOP_HANGUL_OFFSETS[..],
                    pointer)
}

#[inline(always)]
pub fn cp949_top_hangul_encode(bmp: u16) -> u16 {
    map_with_ranges(&CP949_TOP_HANGUL_OFFSETS[..],
                    &CP949_TOP_HANGUL_POINTERS[..],
                    bmp)
}

#[inline(always)]
pub fn cp949_left_hangul_decode(pointer: u16) -> u16 {
    map_with_ranges(&CP949_LEFT_HANGUL_POINTERS[..],
                    &CP949_LEFT_HANGUL_OFFSETS[..],
                    pointer)
}

#[inline(always)]
pub fn cp949_left_hangul_encode(bmp: u16) -> u16 {
    map_with_ranges(&CP949_LEFT_HANGUL_OFFSETS[..],
                    &CP949_LEFT_HANGUL_POINTERS[..],
                    bmp)
}

#[inline(always)]
pub fn gbk_other_decode(pointer: u16) -> u16 {
    map_with_ranges(&GBK_OTHER_POINTERS[..GBK_OTHER_POINTERS.len() - 1],
                    &GBK_OTHER_UNSORTED_OFFSETS[..],
                    pointer)
}

#[inline(always)]
pub fn gbk_other_encode(bmp: u16) -> Option<u16> {
    map_with_unsorted_ranges(&GBK_OTHER_UNSORTED_OFFSETS[..],
                             &GBK_OTHER_POINTERS[..],
                             bmp)
}

#[inline(always)]
pub fn gb2312_other_decode(pointer: u16) -> u16 {
    map_with_ranges(&GB2312_OTHER_POINTERS[..GB2312_OTHER_POINTERS.len() - 1],
                    &GB2312_OTHER_UNSORTED_OFFSETS[..],
                    pointer)
}

#[inline(always)]
pub fn gb2312_other_encode(bmp: u16) -> Option<u16> {
    map_with_unsorted_ranges(&GB2312_OTHER_UNSORTED_OFFSETS[..],
                             &GB2312_OTHER_POINTERS[..],
                             bmp)
}

#[cfg(feature = "no-static-ideograph-encoder-tables")]
#[inline(always)]
pub fn gb2312_level1_hanzi_encode(bmp: u16) -> Option<(u8, u8)> {
    position(&GB2312_HANZI[..(94 * (0xD8 - 0xB0) - 5)], bmp).map(|hanzi_pointer| {
        let hanzi_lead = (hanzi_pointer / 94) + 0xB0;
        let hanzi_trail = (hanzi_pointer % 94) + 0xA1;
        (hanzi_lead, hanzi_trail)
    })
}

#[cfg(not(feature = "no-static-ideograph-encoder-tables"))]
#[inline(always)]
pub fn gb2312_level1_hanzi_encode(bmp: u16) -> Option<(u8, u8)> {
    match GB2312_LEVEL1_HANZI_CODE_POINTS.binary_search(&bmp) {
        Ok(i) => {
            let pair = &GB2312_LEVEL1_HANZI_BYTES[i];
            Some((pair[0], pair[1]))
        }
        Err(_) => None,
    }
}

#[inline(always)]
pub fn gb2312_level2_hanzi_encode(bmp: u16) -> Option<usize> {
    // TODO: optimize
    position(&GB2312_HANZI[(94 * (0xD8 - 0xB0))..], bmp)
}

#[inline(always)]
pub fn ksx1001_other_decode(pointer: u16) -> u16 {
    map_with_ranges(&KSX1001_OTHER_POINTERS[..KSX1001_OTHER_POINTERS.len() - 1],
                    &KSX1001_OTHER_UNSORTED_OFFSETS[..],
                    pointer)
}

#[inline(always)]
pub fn ksx1001_other_encode(bmp: u16) -> Option<u16> {
    map_with_unsorted_ranges(&KSX1001_OTHER_UNSORTED_OFFSETS[..],
                             &KSX1001_OTHER_POINTERS[..],
                             bmp)
}

#[cfg(feature = "no-static-ideograph-encoder-tables")]
#[inline(always)]
pub fn jis0208_level1_kanji_shift_jis_encode(bmp: u16) -> Option<(u8, u8)> {
    position(&JIS0208_LEVEL1_KANJI[..], bmp).map(|kanji_pointer| {
        let pointer = 1410 + kanji_pointer;
        let lead = pointer / 188;
        let lead_offset = if lead < 0x1F {
            0x81
        } else {
            0xC1
        };
        let trail = pointer % 188;
        let trail_offset = if trail < 0x3F {
            0x40
        } else {
            0x41
        };
        ((lead + lead_offset) as u8, (trail + trail_offset) as u8)
    })
}

#[cfg(not(feature = "no-static-ideograph-encoder-tables"))]
#[inline(always)]
pub fn jis0208_level1_kanji_shift_jis_encode(bmp: u16) -> Option<(u8, u8)> {
    match JIS0208_LEVEL1_KANJI_CODE_POINTS.binary_search(&bmp) {
        Ok(i) => {
            let pair = &JIS0208_LEVEL1_KANJI_SHIFT_JIS_BYTES[i];
            Some((pair[0], pair[1]))
        }
        Err(_) => None,
    }
}

#[inline(always)]
pub fn jis0208_level1_kanji_encode(bmp: u16) -> Option<usize> {
    // TODO: optimize
    position(&JIS0208_LEVEL1_KANJI[..], bmp)
}

#[inline(always)]
pub fn jis0208_level2_and_additional_kanji_encode(bmp: u16) -> Option<usize> {
    // TODO: optimize
    position(&JIS0208_LEVEL2_AND_ADDITIONAL_KANJI[..], bmp)
}

pub fn jis0208_symbol_decode(pointer: usize) -> Option<u16> {
    let mut i = 0;
    while i < JIS0208_SYMBOL_TRIPLES.len() {
        let start = JIS0208_SYMBOL_TRIPLES[i] as usize;
        let length = JIS0208_SYMBOL_TRIPLES[i + 1] as usize;
        let pointer_minus_start = pointer.wrapping_sub(start);
        if pointer_minus_start < length {
            let offset = JIS0208_SYMBOL_TRIPLES[i + 2] as usize;
            return Some(JIS0208_SYMBOLS[pointer_minus_start + offset]);
        }
        i += 3;
    }
    None
}

/// Prefers Shift_JIS pointers for the three symbols that are in both ranges.
#[inline(always)]
pub fn jis0208_symbol_encode(bmp: u16) -> Option<usize> {
    let mut i = 0;
    while i < JIS0208_SYMBOL_TRIPLES.len() {
        let pointer_start = JIS0208_SYMBOL_TRIPLES[i] as usize;
        let length = JIS0208_SYMBOL_TRIPLES[i + 1] as usize;
        let symbol_start = JIS0208_SYMBOL_TRIPLES[i + 2] as usize;
        let symbol_end = symbol_start + length;
        let mut symbol_pos = symbol_start;
        while symbol_pos < symbol_end {
            if JIS0208_SYMBOLS[symbol_pos] == bmp {
                return Some(symbol_pos - symbol_start + pointer_start);
            }
            symbol_pos += 1;
        }
        i += 3;
    }
    None
}

#[inline(always)]
pub fn ibm_symbol_encode(bmp: u16) -> Option<usize> {
    position(&JIS0208_SYMBOLS[IBM_SYMBOL_START..IBM_SYMBOL_END], bmp)
        .map(|x| x + IBM_SYMBOL_POINTER_START)
}

#[inline(always)]
pub fn jis0208_range_decode(pointer: usize) -> Option<u16> {
    let mut i = 0;
    while i < JIS0208_RANGE_TRIPLES.len() {
        let start = JIS0208_RANGE_TRIPLES[i] as usize;
        let length = JIS0208_RANGE_TRIPLES[i + 1] as usize;
        let pointer_minus_start = pointer.wrapping_sub(start);
        if pointer_minus_start < length {
            let offset = JIS0208_RANGE_TRIPLES[i + 2] as usize;
            return Some((pointer_minus_start + offset) as u16);
        }
        i += 3;
    }
    None
}

#[inline(always)]
pub fn jis0208_range_encode(bmp: u16) -> Option<usize> {
    let mut i = 0;
    while i < JIS0208_RANGE_TRIPLES.len() {
        let start = JIS0208_RANGE_TRIPLES[i + 2] as usize;
        let length = JIS0208_RANGE_TRIPLES[i + 1] as usize;
        let bmp_minus_start = (bmp as usize).wrapping_sub(start);
        if bmp_minus_start < length {
            let offset = JIS0208_RANGE_TRIPLES[i] as usize;
            return Some(bmp_minus_start + offset);
        }
        i += 3;
    }
    None
}

pub fn jis0212_accented_decode(pointer: usize) -> Option<u16> {
    let mut i = 0;
    while i < JIS0212_ACCENTED_TRIPLES.len() {
        let start = JIS0212_ACCENTED_TRIPLES[i] as usize;
        let length = JIS0212_ACCENTED_TRIPLES[i + 1] as usize;
        let pointer_minus_start = pointer.wrapping_sub(start);
        if pointer_minus_start < length {
            let offset = JIS0212_ACCENTED_TRIPLES[i + 2] as usize;
            let candidate = JIS0212_ACCENTED[pointer_minus_start + offset];
            if candidate == 0 {
                return None;
            }
            return Some(candidate);
        }
        i += 3;
    }
    None
}

#[inline(always)]
pub fn mul_94(lead: u8) -> usize {
    lead as usize * 94
}
''')

data_file.close()

# Variant

variant_file = open("src/variant.rs", "w")
variant_file.write('''// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
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
  if encoding["name"] in [u"UTF-16LE", u"UTF-16BE"]:
    continue
  else:
    encoding_variants.append(encoding["name"])
encoding_variants.append(u"UTF-16")

decoder_variants = []
for variant in encoding_variants:
  if variant == u"GBK":
    continue
  decoder_variants.append(variant)

encoder_variants = []
for variant in encoding_variants:
  if variant in [u"replacement", u"GBK", u"UTF-16"]:
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

write_variant_method("max_utf16_buffer_length", False, [("byte_length", "usize")], "usize", decoder_variants, [], "Decoder")

write_variant_method("max_utf8_buffer_length_without_replacement", False, [("byte_length", "usize")], "usize", decoder_variants, [], "Decoder")

write_variant_method("max_utf8_buffer_length", False, [("byte_length", "usize")], "usize", decoder_variants, [], "Decoder")

write_variant_method("decode_to_utf16_raw", True, [("src", "&[u8]"),
                           ("dst", "&mut [u16]"),
                           ("last", "bool")], "(DecoderResult, usize, usize)", decoder_variants, [], "Decoder")

write_variant_method("decode_to_utf8_raw", True, [("src", "&[u8]"),
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
''')

write_variant_method("max_buffer_length_from_utf16_without_replacement", False, [("u16_length", "usize")], "usize", encoder_variants, [], "Encoder")

write_variant_method("max_buffer_length_from_utf8_without_replacement", False, [("byte_length", "usize")], "usize", encoder_variants, [], "Encoder")

write_variant_method("encode_from_utf16_raw", True, [("src", "&[u16]"),
                           ("dst", "&mut [u8]"),
                           ("last", "bool")], "(EncoderResult, usize, usize)", encoder_variants, [], "Encoder")

write_variant_method("encode_from_utf8_raw", True, [("src", "&str"),
                           ("dst", "&mut [u8]"),
                           ("last", "bool")], "(EncoderResult, usize, usize)", encoder_variants, [], "Encoder")


variant_file.write('''}

pub enum VariantEncoding {
    SingleByte(&'static [u16; 128]),''')

for encoding in multi_byte:
  variant_file.write("%s,\n" % to_camel_name(encoding["name"]))

variant_file.write('''}

impl VariantEncoding {
    pub fn new_variant_decoder(&self) -> VariantDecoder {
        match self {
            &VariantEncoding::SingleByte(table) => SingleByteDecoder::new(table),
            &VariantEncoding::Utf8 => Utf8Decoder::new(),
            &VariantEncoding::Gbk | &VariantEncoding::Gb18030 => Gb18030Decoder::new(),
            &VariantEncoding::Big5 => Big5Decoder::new(),
            &VariantEncoding::EucJp => EucJpDecoder::new(),
            &VariantEncoding::Iso2022Jp => Iso2022JpDecoder::new(),
            &VariantEncoding::ShiftJis => ShiftJisDecoder::new(),
            &VariantEncoding::EucKr => EucKrDecoder::new(),
            &VariantEncoding::Replacement => ReplacementDecoder::new(),
            &VariantEncoding::UserDefined => UserDefinedDecoder::new(),
            &VariantEncoding::Utf16Be => Utf16Decoder::new(true),
            &VariantEncoding::Utf16Le => Utf16Decoder::new(false),
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
            &VariantEncoding::UserDefined => UserDefinedEncoder::new(encoding),
            &VariantEncoding::Utf16Be | &VariantEncoding::Replacement |
            &VariantEncoding::Utf16Le => unreachable!(),
        }
    }
}
''')

variant_file.close()

(ffi_rs_begin, ffi_rs_end) = read_non_generated("../encoding_c/src/lib.rs")

ffi_file = open("../encoding_c/src/lib.rs", "w")

ffi_file.write(ffi_rs_begin)
ffi_file.write("""
// Instead, please regenerate using generate-encoding-data.py

/// The minimum length of buffers that may be passed to `encoding_name()`.
pub const ENCODING_NAME_MAX_LENGTH: usize = %d; // %s

""" % (longest_name_length, longest_name))

for name in preferred:
  ffi_file.write('''/// The %s encoding.
#[no_mangle]
pub static %s_ENCODING: ConstEncoding = ConstEncoding(&%s_INIT);

''' % (to_dom_name(name), to_constant_name(name), to_constant_name(name)))

ffi_file.write(ffi_rs_end)
ffi_file.close()

(single_byte_rs_begin, single_byte_rs_end) = read_non_generated("src/single_byte.rs")

single_byte_file = open("src/single_byte.rs", "w")

single_byte_file.write(single_byte_rs_begin)
single_byte_file.write("""
// Instead, please regenerate using generate-encoding-data.py

    #[test]
    fn test_single_byte_decode() {""")

for name in preferred:
  if name == u"ISO-8859-8-I":
    continue;
  if is_single_byte(name):
    single_byte_file.write("""
        decode_single_byte(%s, %s_DATA);""" % (to_constant_name(name), to_constant_name(name)))

single_byte_file.write("""
    }

    #[test]
    fn test_single_byte_encode() {""")

for name in preferred:
  if name == u"ISO-8859-8-I":
    continue;
  if is_single_byte(name):
    single_byte_file.write("""
        encode_single_byte(%s, %s_DATA);""" % (to_constant_name(name), to_constant_name(name)))


single_byte_file.write("""
    }
""")

single_byte_file.write(single_byte_rs_end)
single_byte_file.close()

static_file = open("../encoding_c/include/encoding_rs_statics.h", "w")

static_file.write("""// Copyright 2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// THIS IS A GENERATED FILE. PLEASE DO NOT EDIT.
// Instead, please regenerate using generate-encoding-data.py

// This file is not meant to be included directly. Instead, encoding_rs.h
// includes this file.

#ifndef encoding_rs_statics_h_
#define encoding_rs_statics_h_

#include <uchar.h>

#ifdef __cplusplus
class Encoding;
class Decoder;
class Encoder;
#else
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

(utf_8_rs_begin, utf_8_rs_end) = read_non_generated("src/utf_8.rs")

utf_8_file = open("src/utf_8.rs", "w")

utf_8_file.write(utf_8_rs_begin)
utf_8_file.write("""
// Instead, please regenerate using generate-encoding-data.py

/// Bit is 1 if the trail is invalid.
static UTF8_TRAIL_INVALID: [u8; 256] = [""")

for i in range(256):
  combined = 0
  if i < 0x80 or i > 0xBF:
    combined |= (1 << 3)
  if i < 0xA0 or i > 0xBF:
    combined |= (1 << 4)
  if i < 0x80 or i > 0x9F:
    combined |= (1 << 5)
  if i < 0x90 or i > 0xBF:
    combined |= (1 << 6)
  if i < 0x80 or i > 0x8F:
    combined |= (1 << 7)
  utf_8_file.write("%d," % combined)

utf_8_file.write("""
];
""")

utf_8_file.write(utf_8_rs_end)
utf_8_file.close()

# Unit tests

TEST_HEADER = '''Any copyright to the test code below this comment is dedicated to the
Public Domain. http://creativecommons.org/publicdomain/zero/1.0/

This is a generated file. Please do not edit.
Instead, please regenerate using generate-encoding-data.py
'''

index = indexes["jis0208"]

jis0208_in_file = open("src/test_data/jis0208_in.txt", "w")
jis0208_in_file.write(TEST_HEADER)
for pointer in range(0, 94 * 94):
  (lead, trail) = divmod(pointer, 94)
  lead += 0xA1
  trail += 0xA1
  jis0208_in_file.write("%s%s\n" % (chr(lead), chr(trail)))
jis0208_in_file.close()

jis0208_in_ref_file = open("src/test_data/jis0208_in_ref.txt", "w")
jis0208_in_ref_file.write(TEST_HEADER)
for pointer in range(0, 94 * 94):
  code_point = index[pointer]
  if code_point:
    jis0208_in_ref_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
  else:
    jis0208_in_ref_file.write(u"\uFFFD\n".encode("utf-8"))
jis0208_in_ref_file.close()

jis0208_out_file = open("src/test_data/jis0208_out.txt", "w")
jis0208_out_ref_file = open("src/test_data/jis0208_out_ref.txt", "w")
jis0208_out_file.write(TEST_HEADER)
jis0208_out_ref_file.write(TEST_HEADER)
for pointer in range(0, 94 * 94):
  code_point = index[pointer]
  if code_point:
    revised_pointer = pointer
    if revised_pointer == 8644 or (revised_pointer >= 1207 and revised_pointer < 1220):
      revised_pointer = index.index(code_point)
    (lead, trail) = divmod(revised_pointer, 94)
    lead += 0xA1
    trail += 0xA1
    jis0208_out_ref_file.write("%s%s\n" % (chr(lead), chr(trail)))
    jis0208_out_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
jis0208_out_file.close()
jis0208_out_ref_file.close()

shift_jis_in_file = open("src/test_data/shift_jis_in.txt", "w")
shift_jis_in_file.write(TEST_HEADER)
for pointer in range(0, len(index)):
  (lead, trail) = divmod(pointer, 188)
  lead += 0x81 if lead < 0x1F else 0xC1
  trail += 0x40 if trail < 0x3F else 0x41
  shift_jis_in_file.write("%s%s\n" % (chr(lead), chr(trail)))
shift_jis_in_file.close()

shift_jis_in_ref_file = open("src/test_data/shift_jis_in_ref.txt", "w")
shift_jis_in_ref_file.write(TEST_HEADER)
for pointer in range(0, len(index)):
  code_point = 0xE000 - 8836 + pointer if pointer >= 8836 and pointer <= 10715 else index[pointer]
  if code_point:
    shift_jis_in_ref_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
  else:
    trail = pointer % 188
    trail += 0x40 if trail < 0x3F else 0x41
    if trail < 0x80:
      shift_jis_in_ref_file.write((u"\uFFFD%s\n" % unichr(trail)).encode("utf-8"))
    else:
      shift_jis_in_ref_file.write(u"\uFFFD\n".encode("utf-8"))
shift_jis_in_ref_file.close()

shift_jis_out_file = open("src/test_data/shift_jis_out.txt", "w")
shift_jis_out_ref_file = open("src/test_data/shift_jis_out_ref.txt", "w")
shift_jis_out_file.write(TEST_HEADER)
shift_jis_out_ref_file.write(TEST_HEADER)
for pointer in range(0, 8272):
  code_point = index[pointer]
  if code_point:
    revised_pointer = pointer
    if revised_pointer >= 1207 and revised_pointer < 1220:
      revised_pointer = index.index(code_point)
    (lead, trail) = divmod(revised_pointer, 188)
    lead += 0x81 if lead < 0x1F else 0xC1
    trail += 0x40 if trail < 0x3F else 0x41
    shift_jis_out_ref_file.write("%s%s\n" % (chr(lead), chr(trail)))
    shift_jis_out_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
for pointer in range(8836, len(index)):
  code_point = index[pointer]
  if code_point:
    revised_pointer = index.index(code_point)
    if revised_pointer >= 8272 and revised_pointer < 8836:
      revised_pointer = pointer
    (lead, trail) = divmod(revised_pointer, 188)
    lead += 0x81 if lead < 0x1F else 0xC1
    trail += 0x40 if trail < 0x3F else 0x41
    shift_jis_out_ref_file.write("%s%s\n" % (chr(lead), chr(trail)))
    shift_jis_out_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
shift_jis_out_file.close()
shift_jis_out_ref_file.close()

iso_2022_jp_in_file = open("src/test_data/iso_2022_jp_in.txt", "w")
iso_2022_jp_in_file.write(TEST_HEADER)
for pointer in range(0, 94 * 94):
  (lead, trail) = divmod(pointer, 94)
  lead += 0x21
  trail += 0x21
  iso_2022_jp_in_file.write("\x1B$B%s%s\x1B(B\n" % (chr(lead), chr(trail)))
iso_2022_jp_in_file.close()

iso_2022_jp_in_ref_file = open("src/test_data/iso_2022_jp_in_ref.txt", "w")
iso_2022_jp_in_ref_file.write(TEST_HEADER)
for pointer in range(0, 94 * 94):
  code_point = index[pointer]
  if code_point:
    iso_2022_jp_in_ref_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
  else:
    iso_2022_jp_in_ref_file.write(u"\uFFFD\n".encode("utf-8"))
iso_2022_jp_in_ref_file.close()

iso_2022_jp_out_file = open("src/test_data/iso_2022_jp_out.txt", "w")
iso_2022_jp_out_ref_file = open("src/test_data/iso_2022_jp_out_ref.txt", "w")
iso_2022_jp_out_file.write(TEST_HEADER)
iso_2022_jp_out_ref_file.write(TEST_HEADER)
for pointer in range(0, 94 * 94):
  code_point = index[pointer]
  if code_point:
    revised_pointer = pointer
    if revised_pointer == 8644 or (revised_pointer >= 1207 and revised_pointer < 1220):
      revised_pointer = index.index(code_point)
    (lead, trail) = divmod(revised_pointer, 94)
    lead += 0x21
    trail += 0x21
    iso_2022_jp_out_ref_file.write("\x1B$B%s%s\x1B(B\n" % (chr(lead), chr(trail)))
    iso_2022_jp_out_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
iso_2022_jp_out_file.close()
iso_2022_jp_out_ref_file.close()

index = indexes["euc-kr"]

euc_kr_in_file = open("src/test_data/euc_kr_in.txt", "w")
euc_kr_in_file.write(TEST_HEADER)
for pointer in range(0, len(index)):
  (lead, trail) = divmod(pointer, 190)
  lead += 0x81
  trail += 0x41
  euc_kr_in_file.write("%s%s\n" % (chr(lead), chr(trail)))
euc_kr_in_file.close()

euc_kr_in_ref_file = open("src/test_data/euc_kr_in_ref.txt", "w")
euc_kr_in_ref_file.write(TEST_HEADER)
for pointer in range(0, len(index)):
  code_point = index[pointer]
  if code_point:
    euc_kr_in_ref_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
  else:
    trail = pointer % 190
    trail += 0x41
    if trail < 0x80:
      euc_kr_in_ref_file.write((u"\uFFFD%s\n" % unichr(trail)).encode("utf-8"))
    else:
      euc_kr_in_ref_file.write(u"\uFFFD\n".encode("utf-8"))
euc_kr_in_ref_file.close()

euc_kr_out_file = open("src/test_data/euc_kr_out.txt", "w")
euc_kr_out_ref_file = open("src/test_data/euc_kr_out_ref.txt", "w")
euc_kr_out_file.write(TEST_HEADER)
euc_kr_out_ref_file.write(TEST_HEADER)
for pointer in range(0, len(index)):
  code_point = index[pointer]
  if code_point:
    (lead, trail) = divmod(pointer, 190)
    lead += 0x81
    trail += 0x41
    euc_kr_out_ref_file.write("%s%s\n" % (chr(lead), chr(trail)))
    euc_kr_out_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
euc_kr_out_file.close()
euc_kr_out_ref_file.close()

index = indexes["gb18030"]

gb18030_in_file = open("src/test_data/gb18030_in.txt", "w")
gb18030_in_file.write(TEST_HEADER)
for pointer in range(0, len(index)):
  (lead, trail) = divmod(pointer, 190)
  lead += 0x81
  trail += 0x40 if trail < 0x3F else 0x41
  gb18030_in_file.write("%s%s\n" % (chr(lead), chr(trail)))
gb18030_in_file.close()

gb18030_in_ref_file = open("src/test_data/gb18030_in_ref.txt", "w")
gb18030_in_ref_file.write(TEST_HEADER)
for pointer in range(0, len(index)):
  code_point = index[pointer]
  if code_point:
    gb18030_in_ref_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
  else:
    trail = pointer % 190
    trail += 0x40 if trail < 0x3F else 0x41
    if trail < 0x80:
      gb18030_in_ref_file.write((u"\uFFFD%s\n" % unichr(trail)).encode("utf-8"))
    else:
      gb18030_in_ref_file.write(u"\uFFFD\n".encode("utf-8"))
gb18030_in_ref_file.close()

gb18030_out_file = open("src/test_data/gb18030_out.txt", "w")
gb18030_out_ref_file = open("src/test_data/gb18030_out_ref.txt", "w")
gb18030_out_file.write(TEST_HEADER)
gb18030_out_ref_file.write(TEST_HEADER)
for pointer in range(0, len(index)):
  if pointer == 6555:
    continue
  code_point = index[pointer]
  if code_point:
    (lead, trail) = divmod(pointer, 190)
    lead += 0x81
    trail += 0x40 if trail < 0x3F else 0x41
    gb18030_out_ref_file.write("%s%s\n" % (chr(lead), chr(trail)))
    gb18030_out_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
gb18030_out_file.close()
gb18030_out_ref_file.close()

index = indexes["big5"]

big5_in_file = open("src/test_data/big5_in.txt", "w")
big5_in_file.write(TEST_HEADER)
for pointer in range(0, len(index)):
  (lead, trail) = divmod(pointer, 157)
  lead += 0x81
  trail += 0x40 if trail < 0x3F else 0x62
  big5_in_file.write("%s%s\n" % (chr(lead), chr(trail)))
big5_in_file.close()

big5_two_characters = {
  1133: u"\u00CA\u0304",
  1135: u"\u00CA\u030C",
  1164: u"\u00EA\u0304",
  1166: u"\u00EA\u030C",
}

big5_in_ref_file = open("src/test_data/big5_in_ref.txt", "w")
big5_in_ref_file.write(TEST_HEADER)
for pointer in range(0, len(index)):
  if pointer in big5_two_characters.keys():
    big5_in_ref_file.write((u"%s\n" % big5_two_characters[pointer]).encode("utf-8"))
    continue
  code_point = index[pointer]
  if code_point:
    big5_in_ref_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
  else:
    trail = pointer % 157
    trail += 0x40 if trail < 0x3F else 0x62
    if trail < 0x80:
      big5_in_ref_file.write((u"\uFFFD%s\n" % unichr(trail)).encode("utf-8"))
    else:
      big5_in_ref_file.write(u"\uFFFD\n".encode("utf-8"))
big5_in_ref_file.close()

big5_out_file = open("src/test_data/big5_out.txt", "w")
big5_out_ref_file = open("src/test_data/big5_out_ref.txt", "w")
big5_out_file.write(TEST_HEADER)
big5_out_ref_file.write(TEST_HEADER)
for pointer in range(((0xA1 - 0x81) * 157), len(index)):
  code_point = index[pointer]
  if code_point:
    if code_point in prefer_last:
      if pointer != pointer_for_prefer_last[prefer_last.index(code_point)]:
        continue
    else:
      if pointer != index.index(code_point):
        continue
    (lead, trail) = divmod(pointer, 157)
    lead += 0x81
    trail += 0x40 if trail < 0x3F else 0x62
    big5_out_ref_file.write("%s%s\n" % (chr(lead), chr(trail)))
    big5_out_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
big5_out_file.close()
big5_out_ref_file.close()

index = indexes["jis0212"]

jis0212_in_file = open("src/test_data/jis0212_in.txt", "w")
jis0212_in_file.write(TEST_HEADER)
for pointer in range(0, len(index)):
  (lead, trail) = divmod(pointer, 94)
  lead += 0xA1
  trail += 0xA1
  jis0212_in_file.write("\x8F%s%s\n" % (chr(lead), chr(trail)))
jis0212_in_file.close()

jis0212_in_ref_file = open("src/test_data/jis0212_in_ref.txt", "w")
jis0212_in_ref_file.write(TEST_HEADER)
for pointer in range(0, len(index)):
  code_point = index[pointer]
  if code_point:
    jis0212_in_ref_file.write((u"%s\n" % unichr(code_point)).encode("utf-8"))
  else:
    jis0212_in_ref_file.write(u"\uFFFD\n".encode("utf-8"))
jis0212_in_ref_file.close()

subprocess.call(["cargo", "fmt"])
