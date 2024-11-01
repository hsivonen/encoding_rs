use encoding_rs::*;

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_jis0208_decode_all() {
    let input = include_bytes!("test_data/jis0208_in.txt");
    let expectation = include_str!("test_data/jis0208_in_ref.txt");
    let (cow, had_errors) = EUC_JP.decode_without_bom_handling(input);
    assert!(had_errors, "Should have had errors.");
    assert_eq!(&cow[..], expectation);
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_jis0208_encode_all() {
    let input = include_str!("test_data/jis0208_out.txt");
    let expectation = include_bytes!("test_data/jis0208_out_ref.txt");
    let (cow, encoding, had_errors) = EUC_JP.encode(input);
    assert!(!had_errors, "Should not have had errors.");
    assert_eq!(encoding, EUC_JP);
    assert_eq!(&cow[..], &expectation[..]);
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_jis0212_decode_all() {
    let input = include_bytes!("test_data/jis0212_in.txt");
    let expectation = include_str!("test_data/jis0212_in_ref.txt");
    let (cow, had_errors) = EUC_JP.decode_without_bom_handling(input);
    assert!(had_errors, "Should have had errors.");
    assert_eq!(&cow[..], expectation);
}
