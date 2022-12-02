#[cfg(feature = "static-curl")]
#[test]
fn test_static_zlib() {
    let version = curl::Version::get();
    let feature_libz = version.feature_libz();

    if !cfg!(feature = "zlib") {
        assert_eq!(feature_libz, false);
    } else {
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
