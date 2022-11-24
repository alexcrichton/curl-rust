#[cfg(all(
    feature = "static-curl",
    not(feature = "zlib"),
    not(feature = "zlib-ng-compat")
))]
#[test]
fn static_with_zlib_disabled() {
    assert_eq!(curl::Version::get().feature_libz(), false);
    assert!(curl::Version::get().libz_version().is_none());
}

#[cfg(all(
    feature = "static-curl",
    feature = "zlib",
    not(feature = "zlib-ng-compat")
))]
#[test]
fn static_with_zlib_enabled() {
    assert!(curl::Version::get().feature_libz());

    // libz_version not contains "zlib-ng" string
    assert_eq!(
        curl::Version::get()
            .libz_version()
            .unwrap()
            .contains("zlib-ng"),
        false
    );
}

#[cfg(all(
    feature = "static-curl",
    not(feature = "zlib"),
    feature = "zlib-ng-compat"
))]
#[test]
fn static_with_zlib_ng_enabled() {
    assert!(curl::Version::get().feature_libz());

    // libz_version contains "zlib-ng" string
    assert_eq!(
        curl::Version::get()
            .libz_version()
            .unwrap()
            .contains("zlib-ng"),
        true
    );
}
