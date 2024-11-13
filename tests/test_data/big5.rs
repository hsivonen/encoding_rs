use encoding_rs::*;

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_big5_decode_all() {
    let input = include_bytes!("big5_in.txt");
    let expectation = include_str!("big5_in_ref.txt");
    let (cow, had_errors) = BIG5.decode_without_bom_handling(input);
    assert!(had_errors, "Should have had errors.");
    assert_eq!(&cow[..], expectation);
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_big5_encode_all() {
    let input = include_str!("big5_out.txt");
    let expectation = include_bytes!("big5_out_ref.txt");
    let (cow, encoding, had_errors) = BIG5.encode(input);
    assert!(!had_errors, "Should not have had errors.");
    assert_eq!(encoding, BIG5);
    assert_eq!(&cow[..], &expectation[..]);
}
