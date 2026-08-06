use encoding_rs::*;

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_shift_jis_decode_all() {
    let input = include_bytes!("shift_jis_in.txt");
    let expectation = include_str!("shift_jis_in_ref.txt");
    let (cow, had_errors) = SHIFT_JIS.decode_without_bom_handling(input);
    assert!(had_errors, "Should have had errors.");
    assert_eq!(&cow[..], expectation);
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_shift_jis_encode_all() {
    let input = include_str!("shift_jis_out.txt");
    let expectation = include_bytes!("shift_jis_out_ref.txt");
    let (cow, encoding, had_errors) = SHIFT_JIS.encode(input);
    assert!(!had_errors, "Should not have had errors.");
    assert_eq!(encoding, SHIFT_JIS);
    assert_eq!(&cow[..], &expectation[..]);
}
