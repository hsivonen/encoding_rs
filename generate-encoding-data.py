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

preferred = []

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

preferred.sort()
labels.sort()

# Big5

def null_to_zero(code_point):
  if not code_point:
    code_point = 0
  return code_point

index = []

for code_point in indexes["big5"]:
  index.append(null_to_zero(code_point))  

index_first = 0

for i in xrange(len(index)):
  if index[i]:
    index_first = i
    break

data_file = open("src/data.rs", "w")

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

static ASTRALNESS: [u32; %d] = [
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

subprocess.call(["cargo", "fmt"])
