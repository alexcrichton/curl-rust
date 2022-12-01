#[test]
fn test_static_zlib() {
    let version = curl::Version::get();
    let feature_libz = version.feature_libz();

    if cfg!(feature = "static-curl-no-zlib") {
        assert_eq!(feature_libz, false);
    }

    if cfg!(feature = "static-curl") || cfg!(feature = "zlib-ng-compat") {
        assert_eq!(feature_libz, true);

        let libz_version = version.libz_version();
        assert!(libz_version.is_some());

        let libz_version = libz_version.unwrap();

        if cfg!(feature = "zlib-ng-compat") {
            assert_eq!(libz_version.contains("zlib-ng"), true);
        } else {
            assert_eq!(libz_version.contains("zlib-ng"), false);
        }
    }
}
