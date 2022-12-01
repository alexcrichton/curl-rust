#[test]
fn test_static_zlib() {
    let version = curl::Version::get();
    let feature_libz = version.feature_libz();

    #[cfg(feature = "static-curl")]
    assert_eq!(feature_libz, true);
    #[cfg(feature = "static-curl-no-zlib")]
    assert_eq!(feature_libz, false);

    #[cfg(not(feature = "static-curl-no-zlib"))]
    {
        let libz_version = version.libz_version();
        assert!(libz_version.is_some());

        let libz_version = libz_version.unwrap();

        #[cfg(feature = "zlib-ng-compat")]
        assert_eq!(libz_version.contains("zlib-ng"), true);

        #[cfg(not(feature = "zlib-ng-compat"))]
        assert_eq!(libz_version.contains("zlib-ng"), false);
    }
}
