#[test]
fn test_static_zlib() {
    let version = curl::Version::get();
    let feature_libz = version.feature_libz();

    #[cfg(all(
        feature = "static-curl",
        not(any(feature = "zlib", feature = "zlib-ng-compat")),
    ))]
    assert_eq!(feature_libz, false);
    #[cfg(all(
        feature = "static-curl",
        any(feature = "zlib", feature = "zlib-ng-compat")
    ))]
    assert_eq!(feature_libz, true);

    #[cfg(all(
        feature = "static-curl",
        any(feature = "zlib", feature = "zlib-ng-compat")
    ))]
    {
        let libz_version = version.libz_version();
        assert!(libz_version.is_some());

        let libz_version = libz_version.unwrap();

        #[cfg(all(feature = "zlib", not(feature = "zlib-ng-compat")))]
        assert_eq!(libz_version.contains("zlib-ng"), false);

        #[cfg(all(not(feature = "zlib"), feature = "zlib-ng-compat"))]
        assert_eq!(libz_version.contains("zlib-ng"), true);
    }
}
