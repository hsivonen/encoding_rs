use encoding_rs::*;

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_euc_kr_decode_all() {
    let input = include_bytes!("euc_kr_in.txt");
    let expectation = include_str!("euc_kr_in_ref.txt");
    let (cow, had_errors) = EUC_KR.decode_without_bom_handling(input);
    assert!(had_errors, "Should have had errors.");
    assert_eq!(&cow[..], expectation);
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_euc_kr_encode_all() {
    let input = include_str!("euc_kr_out.txt");
    let expectation = include_bytes!("euc_kr_out_ref.txt");
    let (cow, encoding, had_errors) = EUC_KR.encode(input);
    assert!(!had_errors, "Should not have had errors.");
    assert_eq!(encoding, EUC_KR);
    assert_eq!(&cow[..], &expectation[..]);
}
