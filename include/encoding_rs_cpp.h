// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#ifndef encoding_rs_cpp_h_
#define encoding_rs_cpp_h_

class Encoding;
class Decoder;
class Encoder;
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

  inline size_t max_utf16_length(const Decoder* decoder, size_t u16_length)
  {
    return decoder_max_utf16_length(const Decoder* decoder, size_t u16_length);
  }

  inline size_t max_utf8_length(const Decoder* decoder, size_t byte_length)
  {
    return decoder_max_utf8_length(const Decoder* decoder, size_t byte_length);
  }

  inline size_t max_utf8_length_with_replacement(const Decoder* decoder,
                                                 size_t byte_length)
  {
    return decoder_max_utf8_length_with_replacement(const Decoder* decoder,
                                                    size_t byte_length);
  }

  inline uint32_t decode_to_utf16(Decoder* decoder, const uint8_t* src,
                                  size_t* src_len, char16_t* dst,
                                  size_t* dst_len, bool last)
  {
    return decoder_decode_to_utf16(Decoder * decoder, const uint8_t* src,
                                   size_t* src_len, char16_t* dst,
                                   size_t* dst_len, bool last);
  }

  inline uint32_t decode_to_utf8(Decoder* decoder, const uint8_t* src,
                                 size_t* src_len, uint8_t* dst, size_t* dst_len,
                                 bool last)
  {
    return decoder_decode_to_utf8(Decoder * decoder, const uint8_t* src,
                                  size_t* src_len, uint8_t* dst,
                                  size_t* dst_len, bool last);
  }

  inline uint32_t decode_to_utf16_with_replacement(
    Decoder* decoder, const uint8_t* src, size_t* src_len, char16_t* dst,
    size_t* dst_len, bool last, bool* had_replacements)
  {
    return decoder_decode_to_utf16_with_replacement(
      Decoder * decoder, const uint8_t* src, size_t* src_len, char16_t* dst,
      size_t* dst_len, bool last, bool* had_replacements);
  }

  inline uint32_t decode_to_utf8_with_replacement(Decoder* decoder,
                                                  const uint8_t* src,
                                                  size_t* src_len, uint8_t* dst,
                                                  size_t* dst_len, bool last,
                                                  bool* had_replacements)
  {
    return decoder_decode_to_utf8_with_replacement(
      Decoder * decoder, const uint8_t* src, size_t* src_len, uint8_t* dst,
      size_t* dst_len, bool last, bool* had_replacements);
  }

private:
  Decoder() = delete;
};

class Encoder final
{

  /// Deallocates an `Encoder` previously allocated by `encoding_new_encoder()`.
  inline void free(Encoder* encoder) { return encoder_free(Encoder * encoder); }

  inline const Encoding* encoding(const Encoder* encoder)
  {
    return encoder_encoding(const Encoder* encoder);
  }

  inline void reset(Encoder* encoder)
  {
    return encoder_reset(Encoder * encoder);
  }

  inline size_t max_buffer_length_from_utf16(const Encoder* encoder,
                                             size_t u16_length)
  {
    return encoder_max_buffer_length_from_utf16(const Encoder* encoder,
                                                size_t u16_length);
  }

  inline size_t max_buffer_length_from_utf8(const Encoder* encoder,
                                            size_t byte_length)
  {
    return encoder_max_buffer_length_from_utf8(const Encoder* encoder,
                                               size_t byte_length);
  }

  inline size_t max_buffer_length_from_utf16_with_replacement_if_no_unmappables(
    const Encoder* encoder, size_t u16_length)
  {
    return encoder_max_buffer_length_from_utf16_with_replacement_if_no_unmappables(
      const Encoder* encoder, size_t u16_length);
  }

  inline size_t max_buffer_length_from_utf8_with_replacement_if_no_unmappables(
    const Encoder* encoder, size_t byte_length)
  {
    return encoder_max_buffer_length_from_utf8_with_replacement_if_no_unmappables(
      const Encoder* encoder, size_t byte_length);
  }

  inline uint32_t encode_from_utf16(Encoder* encoder, const char16_t* src,
                                    size_t* src_len, uint8_t* dst,
                                    size_t* dst_len, bool last)
  {
    return encoder_encode_from_utf16(Encoder * encoder, const char16_t* src,
                                     size_t* src_len, uint8_t* dst,
                                     size_t* dst_len, bool last);
  }

  inline uint32_t encode_from_utf8(Encoder* encoder, const uint8_t* src,
                                   size_t* src_len, uint8_t* dst,
                                   size_t* dst_len, bool last)
  {
    return encoder_encode_from_utf8(Encoder * encoder, const uint8_t* src,
                                    size_t* src_len, uint8_t* dst,
                                    size_t* dst_len, bool last);
  }

  inline uint32_t encode_from_utf16_with_replacement(
    Encoder* encoder, const char16_t* src, size_t* src_len, uint8_t* dst,
    size_t* dst_len, bool last, bool* had_replacements)
  {
    return encoder_encode_from_utf16_with_replacement(
      Encoder * encoder, const char16_t* src, size_t* src_len, uint8_t* dst,
      size_t* dst_len, bool last, bool* had_replacements);
  }

  inline uint32_t encode_from_utf8_with_replacement(
    Encoder* encoder, const uint8_t* src, size_t* src_len, uint8_t* dst,
    size_t* dst_len, bool last, bool* had_replacements)
  {
    return encoder_encode_from_utf8_with_replacement(
      Encoder * encoder, const uint8_t* src, size_t* src_len, uint8_t* dst,
      size_t* dst_len, bool last, bool* had_replacements);
  }
};

#endif // encoding_rs_cpp_h_
