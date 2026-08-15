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

#[cfg(feature = "http3")]
#[test]
fn with_http3_enabled() {
    let version = curl::Version::get();
    assert!(version.feature_http3());
    assert!(version.quic_version().is_some());
}
