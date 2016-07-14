// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
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
  static const Encoding* for_label(gsl::cstring_span<> label)
  {
    return encoding_for_label(reinterpret_cast<const uint8_t*>(label.data()),
                              label.length());
  }

  inline const Encoding* for_label_no_replacement(gsl::cstring_span<> label)
  {
    return encoding_for_label_no_replacement(
      reinterpret_cast<const uint8_t*>(label.data()), label.length());
  }

  inline const Encoding* for_name(gsl::cstring_span<> name)
  {
    return encoding_for_name(reinterpret_cast<const uint8_t*>(name.data()),
                             name.length());
  }

  inline const Encoding* for_bom(gsl::span<const uint8_t> buffer)
  {
    return encoding_for_bom(buffer.data(), buffer.size());
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

  inline bool is_ascii_compatible() const
  {
    return encoding_is_ascii_compatible(this);
  }

  inline const Encoding* output_encoding() const
  {
    return encoding_output_encoding(this);
  }

  inline std::unique_ptr<Decoder> new_decoder() const
  {
    std::unique_ptr<Decoder> decoder(encoding_new_decoder(this));
    return decoder;
  }

  inline void new_decoder_into(Decoder& decoder) const
  {
    encoding_new_decoder_into(this, &decoder);
  }

  inline std::unique_ptr<Decoder> new_decoder_with_bom_removal() const
  {
    std::unique_ptr<Decoder> decoder(encoding_new_decoder_with_bom_removal(this));
    return decoder;
  }

  inline void new_decoder_with_bom_removal_into(Decoder& decoder) const
  {
    encoding_new_decoder_with_bom_removal_into(this, &decoder);
  }

  inline std::unique_ptr<Decoder> new_decoder_without_bom_handling() const
  {
    std::unique_ptr<Decoder> decoder(encoding_new_decoder_without_bom_handling(this));
    return decoder;
  }

  inline void new_decoder_without_bom_handling_into(Decoder& decoder) const
  {
    encoding_new_decoder_without_bom_handling_into(this, &decoder);
  }

  inline std::unique_ptr<Encoder> new_encoder() const
  {
    std::unique_ptr<Encoder> encoder(encoding_new_encoder(this));
    return encoder;
  }

  inline void new_encoder_into(Encoder* encoder) const
  {
    encoding_new_encoder_into(this, encoder);
  }

private:
  Encoding() = delete;
  ~Encoding() = delete;
};

class Decoder final
{
public:
  ~Decoder() {}
  static void operator delete(void* decoder) { decoder_free(reinterpret_cast<Decoder*>(decoder)); }

  inline const Encoding* encoding() const { return decoder_encoding(this); }

  inline size_t max_utf16_buffer_length(size_t u16_length) const
  {
    return decoder_max_utf16_buffer_length(this, u16_length);
  }

  inline size_t max_utf8_buffer_length_without_replacement(size_t byte_length) const
  {
    return decoder_max_utf8_buffer_length_without_replacement(this, byte_length);
  }

  inline size_t max_utf8_buffer_length(size_t byte_length) const
  {
    return decoder_max_utf8_buffer_length(this, byte_length);
  }

  inline std::tuple<uint32_t, size_t, size_t> decode_to_utf16_without_replacement(
    gsl::span<const uint8_t> src, gsl::span<char16_t> dst, bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    uint32_t result = decoder_decode_to_utf16_without_replacement(this, src.data(), &src_read,
                                              dst.data(), &dst_written, last);
    return std::make_tuple(result, src_read, dst_written);
  }

  inline std::tuple<uint32_t, size_t, size_t> decode_to_utf8_without_replacement(
    gsl::span<const uint8_t> src, gsl::span<uint8_t> dst, bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    uint32_t result = decoder_decode_to_utf8_without_replacement(this, src.data(), &src_read,
                                             dst.data(), &dst_written, last);
    return std::make_tuple(result, src_read, dst_written);
  }

  inline std::tuple<uint32_t, size_t, size_t, bool>
  decode_to_utf16(gsl::span<const uint8_t> src, gsl::span<char16_t> dst,
                                   bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    bool had_replacements;
    uint32_t result = decoder_decode_to_utf16(
      this, src.data(), &src_read, dst.data(), &dst_written, last,
      &had_replacements);
    return std::make_tuple(result, src_read, dst_written, had_replacements);
  }

  inline std::tuple<uint32_t, size_t, size_t, bool>
  decode_to_utf8(gsl::span<const uint8_t> src, gsl::span<uint8_t> dst,
                                  bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    bool had_replacements;
    uint32_t result = decoder_decode_to_utf8(
      this, src.data(), &src_read, dst.data(), &dst_written, last,
      &had_replacements);
    return std::make_tuple(result, src_read, dst_written, had_replacements);
  }

private:
  Decoder() = delete;
};

class Encoder final
{
public:
  ~Encoder() {}
  static void operator delete(void* encoder) { encoder_free(reinterpret_cast<Encoder*>(encoder)); }

  inline const Encoding* encoding() const { return encoder_encoding(this); }

  inline size_t max_buffer_length_from_utf16_without_replacement(size_t u16_length) const
  {
    return encoder_max_buffer_length_from_utf16_without_replacement(this, u16_length);
  }

  inline size_t max_buffer_length_from_utf8_without_replacement(size_t byte_length) const
  {
    return encoder_max_buffer_length_from_utf8_without_replacement(this, byte_length);
  }

  inline size_t max_buffer_length_from_utf16_if_no_unmappables(
    size_t u16_length) const
  {
    return encoder_max_buffer_length_from_utf16_if_no_unmappables(
      this, u16_length);
  }

  inline size_t max_buffer_length_from_utf8_if_no_unmappables(
    size_t byte_length) const
  {
    return encoder_max_buffer_length_from_utf8_if_no_unmappables(
      this, byte_length);
  }

  inline std::tuple<uint32_t, size_t, size_t> encode_from_utf16_without_replacement(
    gsl::span<const char16_t> src, gsl::span<uint8_t> dst, bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    uint32_t result = encoder_encode_from_utf16_without_replacement(this, src.data(), &src_read,
                                                dst.data(), &dst_written, last);
    return std::make_tuple(result, src_read, dst_written);
  }

  inline std::tuple<uint32_t, size_t, size_t> encode_from_utf8_without_replacement(
    gsl::span<const uint8_t> src, gsl::span<uint8_t> dst, bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    uint32_t result = encoder_encode_from_utf8_without_replacement(this, src.data(), &src_read,
                                               dst.data(), &dst_written, last);
    return std::make_tuple(result, src_read, dst_written);
  }

  inline std::tuple<uint32_t, size_t, size_t, bool>
  encode_from_utf16(gsl::span<const char16_t> src,
                                     gsl::span<uint8_t> dst, bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    bool had_replacements;
    uint32_t result = encoder_encode_from_utf16(
      this, src.data(), &src_read, dst.data(), &dst_written, last,
      &had_replacements);
    return std::make_tuple(result, src_read, dst_written, had_replacements);
  }

  inline std::tuple<uint32_t, size_t, size_t, bool>
  encode_from_utf8(gsl::span<const uint8_t> src, gsl::span<uint8_t> dst,
                                    bool last)
  {
    size_t src_read = src.size();
    size_t dst_written = dst.size();
    bool had_replacements;
    uint32_t result = encoder_encode_from_utf8(
      this, src.data(), &src_read, dst.data(), &dst_written, last,
      &had_replacements);
    return std::make_tuple(result, src_read, dst_written, had_replacements);
  }

private:
  Encoder() = delete;
};

#endif // encoding_rs_cpp_h_
