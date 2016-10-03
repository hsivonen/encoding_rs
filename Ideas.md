This document contains notes about various ideas that for one reason or another
are not being actively pursued.

==Two-level tables for single-byte encodings==

It might be possible to make the data for single-byte encodings smaller at the
expense of complexity by changing the lookups to have two levels. The
first-level of the lookup table would have 8 (or maybe 4) entries of 16-bit
values representing (depending on the most significant bit used as a tag a bit)
offset to add to the byte value or index to a shared second-level lookup table.

If the first-level lookup table had eight entries, each entry would represent a
stride of 16 code points. On the other hand, if there were only four entries,
each stride would represent 32 code points.

For example, to represent windows-1252, with 4 strides, the first stride would
point to lookup table and the other three strides would have the offset 0.
Meanwhile, e.g. windows-1255 could not use the offset mode with 4 strides,
since none of the 4 strides are representable as a mere offset. However, with 8
strides, the 5th and 7th strides could be represented as offsets.

The second-level lookup-table should be shared among the encodings, so that
shared strides don't need to be duplicated.

==Alternating lead/trail bytes and branch prediction==

Currently, the decoders don't make use of the fact that legacy multi-byte
encodings alternate between lead byte being set and unset. Potentially the
loop could be unrolled and the check for lead being set be predicted in opposite
ways in the two copies of the loop.

==Next byte is non-ASCII after ASCII optimization==

The current plan for a SIMD-accelerated inner loop for handling ASCII bytes
makes no use of the bit of information that if the buffers didn't end but the
ASCII loop exited, the next byte will not be an ASCII byte.

==The structure of handles.rs and bound checks==

handles.rs is designed to make it possible to avoid bound checks when writing
to the slices. While it would be possible to omit the bound checks manually,
it probably makes more sense to carry out an investigation to make sure that
the compiler performs the omission. If not, it makes more sense to file a bug
on the compiler than to omit the checks manually.


