// Any copyright to the test code below this comment is dedicated to the
// Public Domain. http://creativecommons.org/publicdomain/zero/1.0/
use encoding_rs::*;

fn decode_chunks_to_utf8(encoding: &'static Encoding, chunks: &[&[u8]]) -> (String, bool) {
    let mut decoder = encoding.new_decoder_without_bom_handling();
    let mut output = Vec::new();
    let mut had_errors = false;
    for (index, chunk) in chunks.iter().enumerate() {
        let last = index + 1 == chunks.len();
        let mut buffer = vec![0; decoder.max_utf8_buffer_length(chunk.len()).unwrap()];
        let (result, read, written, errors) = decoder.decode_to_utf8(chunk, &mut buffer, last);
        assert_eq!(result, CoderResult::InputEmpty);
        assert_eq!(read, chunk.len());
        had_errors |= errors;
        output.extend_from_slice(&buffer[..written]);
    }
    (String::from_utf8(output).unwrap(), had_errors)
}

fn decode_chunks_to_utf16(encoding: &'static Encoding, chunks: &[&[u8]]) -> (Vec<u16>, bool) {
    let mut decoder = encoding.new_decoder_without_bom_handling();
    let mut output = Vec::new();
    let mut had_errors = false;
    for (index, chunk) in chunks.iter().enumerate() {
        let last = index + 1 == chunks.len();
        let mut buffer = vec![0; decoder.max_utf16_buffer_length(chunk.len()).unwrap()];
        let (result, read, written, errors) = decoder.decode_to_utf16(chunk, &mut buffer, last);
        assert_eq!(result, CoderResult::InputEmpty);
        assert_eq!(read, chunk.len());
        had_errors |= errors;
        output.extend_from_slice(&buffer[..written]);
    }
    (output, had_errors)
}

fn assert_streaming_decode_matches(
    encoding: &'static Encoding,
    chunks: &[&[u8]],
    expected: &str,
    expected_had_errors: bool,
) {
    let (utf8, utf8_had_errors) = decode_chunks_to_utf8(encoding, chunks);
    assert_eq!(utf8, expected);
    assert_eq!(utf8_had_errors, expected_had_errors);

    let (utf16, utf16_had_errors) = decode_chunks_to_utf16(encoding, chunks);
    let expected_utf16: Vec<u16> = expected.encode_utf16().collect();
    assert_eq!(utf16, expected_utf16);
    assert_eq!(utf16_had_errors, expected_had_errors);
}

#[test]
fn test_big5_streaming_decode_ignores_empty_chunk_between_lead_and_trail() {
    assert_streaming_decode_matches(BIG5, &[b"\x87", b"", b"\x40"], "\u{43F0}", false);
}

#[test]
fn test_shift_jis_streaming_decode_ignores_empty_chunk_between_lead_and_trail() {
    assert_streaming_decode_matches(SHIFT_JIS, &[b"\x81", b"", b"\x40"], "\u{3000}", false);
}

#[test]
fn test_euc_kr_streaming_decode_ignores_empty_chunk_between_lead_and_trail() {
    assert_streaming_decode_matches(EUC_KR, &[b"\x81", b"", b"\x41"], "\u{AC02}", false);
}

#[test]
fn test_big5_streaming_decode_reports_pending_lead_at_eof_after_empty_chunk() {
    assert_streaming_decode_matches(BIG5, &[b"\x87", b"", b""], "\u{FFFD}", true);
}

#[test]
fn test_shift_jis_streaming_decode_reports_pending_lead_at_eof_after_empty_chunk() {
    assert_streaming_decode_matches(SHIFT_JIS, &[b"\x81", b"", b""], "\u{FFFD}", true);
}

#[test]
fn test_euc_kr_streaming_decode_reports_pending_lead_at_eof_after_empty_chunk() {
    assert_streaming_decode_matches(EUC_KR, &[b"\x81", b"", b""], "\u{FFFD}", true);
}

#[test]
fn test_big5_pending_lead_survives_utf8_output_full_retry() {
    let mut decoder = BIG5.new_decoder_without_bom_handling();

    let mut first_buffer = [0u8; 8];
    let (result, read, written, had_errors) =
        decoder.decode_to_utf8(b"\x87", &mut first_buffer, false);
    assert_eq!(result, CoderResult::InputEmpty);
    assert_eq!(read, 1);
    assert_eq!(written, 0);
    assert!(!had_errors);

    let mut too_small = [0u8; 3];
    let (result, read, written, had_errors) =
        decoder.decode_to_utf8(b"\x40", &mut too_small, false);
    assert_eq!(result, CoderResult::OutputFull);
    assert_eq!(read, 0);
    assert_eq!(written, 0);
    assert!(!had_errors);

    let mut retry_buffer = [0u8; 8];
    let (result, read, written, had_errors) =
        decoder.decode_to_utf8(b"\x40", &mut retry_buffer, true);
    assert_eq!(result, CoderResult::InputEmpty);
    assert_eq!(read, 1);
    assert_eq!(written, "\u{43F0}".len());
    assert!(!had_errors);
    assert_eq!(&retry_buffer[..written], "\u{43F0}".as_bytes());
}

#[test]
fn test_big5_pending_lead_survives_utf16_output_full_retry() {
    let mut decoder = BIG5.new_decoder_without_bom_handling();

    let mut first_buffer = [0u16; 2];
    let (result, read, written, had_errors) =
        decoder.decode_to_utf16(b"\x87", &mut first_buffer, false);
    assert_eq!(result, CoderResult::InputEmpty);
    assert_eq!(read, 1);
    assert_eq!(written, 0);
    assert!(!had_errors);

    let mut too_small = [0u16; 1];
    let (result, read, written, had_errors) =
        decoder.decode_to_utf16(b"\x40", &mut too_small, false);
    assert_eq!(result, CoderResult::OutputFull);
    assert_eq!(read, 0);
    assert_eq!(written, 0);
    assert!(!had_errors);

    let mut retry_buffer = [0u16; 2];
    let (result, read, written, had_errors) =
        decoder.decode_to_utf16(b"\x40", &mut retry_buffer, true);
    assert_eq!(result, CoderResult::InputEmpty);
    assert_eq!(read, 1);
    assert_eq!(written, 1);
    assert!(!had_errors);
    assert_eq!(&retry_buffer[..written], &['\u{43F0}' as u16]);
}
