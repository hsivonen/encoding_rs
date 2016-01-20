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

singleByte = []

multiByte = []

def toStructName(name):
  if name == u"iso-8859-8-i":
    return u"Iso8I"
  if name.startswith(u"iso-8859-"):
    return name.replace(u"iso-8859-", u"Iso")
  return name.title().replace(u"X-", u"").replace(u"-", u"").replace(u"_", u"")

def toConstantName(name):
  return name.replace(u"-", u"_").upper()

# 

for group in data:
  if group["heading"] == "Legacy single-byte encodings":
    singleByte = group["encodings"]
  else:
    multiByte.extend(group["encodings"])
  for encoding in group["encodings"]:
    preferred.append(encoding["name"])
    for label in encoding["labels"]:
      labels.append(Label(label, encoding["name"]))

preferred.sort()
labels.sort()

# Big5

def nullToZero(codePoint):
  if not codePoint:
    codePoint = 0
  return codePoint

index = []

for codePoint in indexes["big5"]:
  index.append(nullToZero(codePoint))  

indexFirst = 0

for i in xrange(len(index)):
  if index[i]:
    indexFirst = i
    break

dataFile = open("src/data.rs", "w")

bits = []
for codePoint in index:
  bits.append(1 if codePoint > 0xFFFF else 0)

bitsCap = len(bits)

bitsFirst = 0
for i in xrange(len(bits)):
  if bits[i]:
    bitsFirst = i
    break

# pad length to multiple of 32
for j in xrange(32 - ((len(bits) - bitsFirst) % 32)):
  bits.append(0)

dataFile.write('''// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
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
''' % ((len(bits) - bitsFirst) / 32))

i = bitsFirst
while i < len(bits):
  accu = 0
  for j in xrange(32):
    accu |= bits[i + j] << j
  dataFile.write('0x%08X,\n' % accu)
  i += 32

dataFile.write('''];

static LOW_BITS: [u16; %d] = [
''' % (len(index) - indexFirst))

for i in xrange(indexFirst, len(index)):
  dataFile.write('0x%04X,\n' % (index[i] & 0xFFFF))

dataFile.write('''];

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
''' % (bitsFirst, bitsCap - bitsFirst, indexFirst, len(index) - indexFirst))

dataFile.write('''
#[inline(always)]
pub fn big5_find_pointer(low_bits: u16, is_astral: bool) -> usize {
    if !is_astral {
        match low_bits {
''')

hkscsBound = (0xA1 - 0x81) * 157

hkscsStartIndex = hkscsBound -  indexFirst

preferLast = [
  0x2550,
  0x255E,
  0x2561,
  0x256A,
  0x5341,
  0x5345,
]

for codePoint in preferLast:
  # Python lists don't have .rindex() :-(
  for i in xrange(len(index) - 1, -1, -1):
    candidate = index[i]
    if candidate == codePoint:
       dataFile.write('''0x%04X => {
   return %d;
},
''' % (codePoint, i))
       break

dataFile.write('''_ => {},
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
''' % (hkscsStartIndex, hkscsBound))
dataFile.close()

subprocess.call(["cargo", "fmt"])
