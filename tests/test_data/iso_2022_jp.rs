use encoding_rs::*;

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_iso_2022_jp_decode_all() {
    let input = include_bytes!("iso_2022_jp_in.txt");
    let expectation = include_str!("iso_2022_jp_in_ref.txt");
    let (cow, had_errors) = ISO_2022_JP.decode_without_bom_handling(input);
    assert!(had_errors, "Should have had errors.");
    assert_eq!(&cow[..], expectation);
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_iso_2022_jp_encode_all() {
    let input = include_str!("iso_2022_jp_out.txt");
    let expectation = include_bytes!("iso_2022_jp_out_ref.txt");
    let (cow, encoding, had_errors) = ISO_2022_JP.encode(input);
    assert!(!had_errors, "Should not have had errors.");
    assert_eq!(encoding, ISO_2022_JP);
    assert_eq!(&cow[..], &expectation[..]);
}
