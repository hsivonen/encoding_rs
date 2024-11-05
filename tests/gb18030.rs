use encoding_rs::*;

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_gb18030_decode_all() {
    let input = include_bytes!("test_data/gb18030_in.txt");
    let expectation = include_str!("test_data/gb18030_in_ref.txt");
    let (cow, had_errors) = GB18030.decode_without_bom_handling(input);
    assert!(!had_errors, "Should not have had errors.");
    assert_eq!(&cow[..], expectation);
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_gb18030_encode_all() {
    let input = include_str!("test_data/gb18030_out.txt");
    let expectation = include_bytes!("test_data/gb18030_out_ref.txt");
    let (cow, encoding, had_errors) = GB18030.encode(input);
    assert!(!had_errors, "Should not have had errors.");
    assert_eq!(encoding, GB18030);
    assert_eq!(&cow[..], &expectation[..]);
}
