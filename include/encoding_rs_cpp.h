// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#pragma once

#ifndef encoding_rs_cpp_h_
#define encoding_rs_cpp_h_

#include <string>
#include <tuple>
#include <memory>
#include "gsl.h"

#include "encoding_rs.h"

class Encoding final
{
public:
  inline static const Encoding* for_label(cstring_span label)
  {
    return encoding_for_label(reinterpret_cast<const uint8_t*>(label.data()),
                              label.length());
  }

  inline static const Encoding* for_label_no_replacement(cstring_span label)
  {
    return encoding_for_label_no_replacement(
      reinterpret_cast<const uint8_t*>(label.data()), label.length());
  }

  inline static const Encoding* for_name(cstring_span name)
  {
    return encoding_for_name(reinterpret_cast<const uint8_t*>(name.data()),
                             name.length());
  }

  inline std::string name() const
  {
    std::string name(ENCODING_NAME_MAX_LENGTH, '\0');
    // http://herbsutter.com/2008/04/07/cringe-not-vectors-are-guaranteed-to-be-contiguous/#comment-483
    size_t length = encoding_name(this, reinterpret_cast<uint8_t*>(&name[0]));
    name.resize(length);
    return name;
  }

  inline bool can_encode_everything() const
  {
    return encoding_can_encode_everything(this);
  }

  inline std::unique_pointer<Decoder> new_decoder() const
  {
    std::unique_pointer<Decoder> decoder(encoding_new_decoder(this));
    return decoder;
  }

  inline std::unique_pointer<Encoder> new_encoder() const
  {
    std::unique_pointer<Encoder> encoder(encoding_new_encoder(this));
    return encoder;
  }

private:
  Encoding() = delete;
  ~Encoding() = delete;
};

class Decoder final
{
public:
  ~Decoder() {}
  operator delete(void* decoder) { decoder_free(decoder); }

  inline const Encoding* encoding() const { return decoder_encoding(this); }

  inline void reset() { return decoder_reset(this); }

  inline size_t max_utf16_length(size_t u16_length) const
  {
    return decoder_max_utf16_length(this, u16_length);
  }

  inline size_t max_utf8_length(size_t byte_length) const
  {
    return decoder_max_utf8_length(this, byte_length);
  }

  inline size_t max_utf8_length_with_replacement(size_t byte_length) const
  {
    return decoder_max_utf8_length_with_replacement(this, byte_length);
  }

  inline std::tuple<uint32_t, size_t, size_t> decode_to_utf16(
    span<const uint8_t> src, span<char16_t> dst, bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    uint32_t result = decoder_decode_to_utf16(this, src.data(), &src_read,
                                              dst.data(), &dst_written, last);
    return std::make_tuple(result, src_read, dst_written);
  }

  inline std::tuple<uint32_t, size_t, size_t> decode_to_utf8(
    span<const uint8_t> src, span<uint8_t> dst, bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    uint32_t result = decoder_decode_to_utf8(this, src.data(), &src_read,
                                             dst.data(), &dst_written, last);
    return std::make_tuple(result, src_read, dst_written);
  }

  inline std::tuple<uint32_t, size_t, size_t, bool>
  decode_to_utf16_with_replacement(span<const uint8_t> src, span<char16_t> dst,
                                   bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    bool had_replacements;
    uint32_t result = decoder_decode_to_utf16_with_replacement(
      this, src.data(), &src_read, dst.data(), &dst_written, last,
      &had_replacements);
    return std::make_tuple(result, src_read, dst_written, had_replacements);
  }

  inline std::tuple<uint32_t, size_t, size_t, bool>
  decode_to_utf8_with_replacement(span<const uint8_t> src, span<uint8_t> dst,
                                  bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    bool had_replacements;
    uint32_t result = decoder_decode_to_utf8_with_replacement(
      this, src.data(), &src_read, dst.data(), &dst_written, last,
      &had_replacements);
    return std::make_tuple(result, src_read, dst_written, had_replacements);
  }

private:
  Decoder() = delete;
};

class Encoder final
{
  ~Decoder() {}
  operator delete(void* encoder) { encoder_free(encoder); }

  inline const Encoding* encoding() const { return encoder_encoding(this); }

  inline void reset() { return encoder_reset(this); }

  inline size_t max_buffer_length_from_utf16(size_t u16_length) const
  {
    return encoder_max_buffer_length_from_utf16(this, u16_length);
  }

  inline size_t max_buffer_length_from_utf8(size_t byte_length) const
  {
    return encoder_max_buffer_length_from_utf8(this, byte_length);
  }

  inline size_t max_buffer_length_from_utf16_with_replacement_if_no_unmappables(
    size_t u16_length) const
  {
    return encoder_max_buffer_length_from_utf16_with_replacement_if_no_unmappables(
      this, u16_length);
  }

  inline size_t max_buffer_length_from_utf8_with_replacement_if_no_unmappables(
    size_t byte_length) const
  {
    return encoder_max_buffer_length_from_utf8_with_replacement_if_no_unmappables(
      this, byte_length);
  }

  inline std::tuple<uint32_t, size_t, size_t> encode_from_utf16(
    span<const char16_t> src, span<uint8_t> dst, bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    uint32_t result = encoder_encode_from_utf16(this, src.data(), &src_read,
                                                dst.data(), &dst_written, last);
    return std::make_tuple(result, src_read, dst_written);
  }

  inline std::tuple<uint32_t, size_t, size_t> encode_from_utf8(
    span<const uint8_t> src, span<uint8_t> dst, bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    uint32_t result = encoder_encode_from_utf8(this, src.data(), &src_read,
                                               dst.data(), &dst_written, last);
    return std::make_tuple(result, src_read, dst_written);
  }

  inline std::tuple<uint32_t, size_t, size_t, bool>
  encode_from_utf16_with_replacement(span<const char16_t> src,
                                     span<uint8_t> dst, bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    bool had_replacements;
    uint32_t result = encoder_encode_from_utf16_with_replacement(
      this, src.data(), &src_read, dst.data(), &dst_written, last,
      &had_replacements);
    return std::make_tuple(result, src_read, dst_written, had_replacements);
  }

  inline std::tuple<uint32_t, size_t, size_t, bool>
  encode_from_utf8_with_replacement(span<const uint8_t> src, span<uint8_t> dst,
                                    bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    bool had_replacements;
    uint32_t result = encoder_encode_from_utf8_with_replacement(
      this, src.data(), &src_read, dst.data(), &dst_written, last,
      &had_replacements);
    return std::make_tuple(result, src_read, dst_written, had_replacements);
  }

private:
  Encoder() = delete;
};

#endif // encoding_rs_cpp_h_
