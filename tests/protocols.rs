#[cfg(all(feature = "static-curl", not(feature = "protocol-ftp")))]
#[test]
fn static_with_ftp_disabled() {
    assert!(curl::Version::get()
        .protocols()
        .filter(|&p| p == "ftp")
        .next()
        .is_none());
}

#[cfg(all(feature = "static-curl", feature = "protocol-ftp"))]
#[test]
fn static_with_ftp_enabled() {
    assert!(curl::Version::get()
        .protocols()
        .filter(|&p| p == "ftp")
        .next()
        .is_some());
}
