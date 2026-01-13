//! libcurl URL parsing, generation, and manipulation.

mod error;
mod get_flags;
mod handle;
mod set_flags;

pub use error::Error;
pub use get_flags::GetFlags;
pub use handle::Url;
pub use set_flags::SetFlags;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_new() {
        let mut url = Url::new().unwrap();
        url.set_url("https://www.rust-lang.org/", SetFlags::new())
            .unwrap();
        let full_url = url.get_url(GetFlags::new()).unwrap();
        assert_eq!(full_url, "https://www.rust-lang.org/");
        url.clear_url().unwrap();
    }

    fn test_component_set_get_clear(
        val: &str,
        set: fn(&mut Url, &str, SetFlags) -> Result<(), Error>,
        get: fn(&Url, GetFlags) -> Result<Option<String>, Error>,
        clear: fn(&mut Url) -> Result<(), Error>,
    ) {
        let mut url = Url::new().unwrap();
        set(&mut url, val, SetFlags::new()).unwrap();
        let retrieved = get(&url, GetFlags::new()).unwrap();
        assert_eq!(retrieved, Some(val.into()));
        clear(&mut url).unwrap();
        let retrieved = get(&url, GetFlags::new()).unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_url_parts() {
        test_component_set_get_clear("http", Url::set_scheme, Url::get_scheme, Url::clear_scheme);
        test_component_set_get_clear("alex", Url::set_user, Url::get_user, Url::clear_user);
        test_component_set_get_clear(
            "secret",
            Url::set_password,
            Url::get_password,
            Url::clear_password,
        );
        test_component_set_get_clear(
            "opt",
            Url::set_options,
            Url::get_options,
            Url::clear_options,
        );
        test_component_set_get_clear(
            "www.rust-lang.org",
            Url::set_host,
            Url::get_host,
            Url::clear_host,
        );
        test_component_set_get_clear(
            "eth0",
            Url::set_zone_id,
            Url::get_zone_id,
            Url::clear_zone_id,
        );

        let mut url = Url::new().unwrap();
        url.set_port(10086, SetFlags::new()).unwrap();
        let port = url.get_port(GetFlags::new()).unwrap();
        assert_eq!(port, Some(10086));
        url.clear_port().unwrap();
        let port = url.get_port(GetFlags::new()).unwrap();
        assert_eq!(port, None);

        url.set_path("/community", SetFlags::new()).unwrap();
        let path = url.get_path(GetFlags::new()).unwrap();
        assert_eq!(path, "/community".to_string());
        url.set_path("", SetFlags::new()).unwrap();
        let path = url.get_path(GetFlags::new()).unwrap();
        assert_eq!(path, "/".to_string());

        test_component_set_get_clear("a=1&b=2", Url::set_query, Url::get_query, Url::clear_query);
        test_component_set_get_clear(
            "fragment",
            Url::set_fragment,
            Url::get_fragment,
            Url::clear_fragment,
        );
    }

    #[test]
    fn test_url_parser() {
        let mut url = Url::new().unwrap();
        url.set_url(
            "https://user:password@[::1%eth0]:8080/path?query#fragment",
            SetFlags::new(),
        )
        .unwrap();

        let empty_flags = GetFlags::new();
        assert_eq!(url.get_scheme(empty_flags).unwrap(), Some("https".into()));
        assert_eq!(url.get_user(empty_flags).unwrap(), Some("user".into()));
        assert_eq!(
            url.get_password(empty_flags).unwrap(),
            Some("password".into())
        );
        assert_eq!(url.get_host(empty_flags).unwrap(), Some("[::1]".into()));
        assert_eq!(url.get_port(empty_flags).unwrap(), Some(8080));
        assert_eq!(url.get_path(empty_flags).unwrap(), "/path".to_string());
        assert_eq!(url.get_query(empty_flags).unwrap(), Some("query".into()));
        assert_eq!(
            url.get_fragment(empty_flags).unwrap(),
            Some("fragment".into())
        );
        assert_eq!(url.get_zone_id(empty_flags).unwrap(), Some("eth0".into()));
        assert_eq!(
            url.get_url(empty_flags).unwrap(),
            "https://user:password@[::1%25eth0]:8080/path?query#fragment"
        );
    }

    #[test]
    fn test_url_get_flags() {
        let mut url = Url::new().unwrap();
        url.set_url("https://www.rust-lang.org/", SetFlags::new())
            .unwrap();
        assert_eq!(url.get_port(GetFlags::new()).unwrap(), None);
        assert_eq!(
            url.get_port(GetFlags::new().default_port()).unwrap(),
            Some(443)
        );

        url.clear_scheme().unwrap();
        url.set_host("www.rust-lang.org", SetFlags::new()).unwrap();
        assert_eq!(
            url.get_url(GetFlags::new()).unwrap_err().code(),
            curl_sys::CURLUE_NO_SCHEME
        );
        assert_eq!(
            url.get_url(GetFlags::new().default_scheme()).unwrap(),
            "https://www.rust-lang.org/"
        );

        url.set_url("https://www.rust-lang.org:443/", SetFlags::new())
            .unwrap();
        assert_eq!(
            url.get_url(GetFlags::new()).unwrap(),
            "https://www.rust-lang.org:443/"
        );
        assert_eq!(
            url.get_url(GetFlags::new().no_default_port()).unwrap(),
            "https://www.rust-lang.org/"
        );

        url.set_url("https://www.rust-lang.org/a%20b", SetFlags::new())
            .unwrap();
        assert_eq!(url.get_path(GetFlags::new()).unwrap(), "/a%20b");
        assert_eq!(url.get_path(GetFlags::new().urldecode()).unwrap(), "/a b");

        url.set_host("茹斯特", SetFlags::new()).unwrap();
        assert_eq!(
            url.get_host(GetFlags::new().urlencode()).unwrap(),
            Some("%e8%8c%b9%e6%96%af%e7%89%b9".into())
        );

        // FIXME: IDN support may not be available in all builds of libcurl.
        // assert_eq!(
        //     url.get_host(GetFlags::new().punycode()).unwrap(),
        //     Some("xn--dfvq8zr5m".into())
        // );

        // url.set_host("xn--dfvq8zr5m", SetFlags::new()).unwrap();
        // assert_eq!(
        //     url.get_host(GetFlags::new().punycode2idn()).unwrap(),
        //     Some("茹斯特".into())
        // );

        url.set_query("", SetFlags::new()).unwrap();
        assert_eq!(url.get_query(GetFlags::new()).unwrap(), None);
        assert_eq!(
            url.get_query(GetFlags::new().get_empty()).unwrap(),
            Some("".into())
        );
    }

    #[test]
    fn test_url_guess_scheme() {
        let mut url = Url::new().unwrap();
        url.set_url("www.rust-lang.org", SetFlags::new().guess_scheme())
            .unwrap();

        assert_eq!(
            url.get_scheme(GetFlags::new().no_guess_scheme()).unwrap(),
            None
        );
        assert_eq!(
            url.get_scheme(GetFlags::new()).unwrap(),
            Some("http".into())
        );
    }

    #[test]
    fn test_url_set_flags() {
        let mut url = Url::new().unwrap();
        url.set_query("a=1", SetFlags::new()).unwrap();

        url.set_query("b=2", SetFlags::new().append_query())
            .unwrap();
        assert_eq!(
            url.get_query(GetFlags::new()).unwrap(),
            Some("a=1&b=2".into())
        );

        assert_eq!(
            url.set_scheme("rust", SetFlags::new()).unwrap_err().code(),
            curl_sys::CURLUE_UNSUPPORTED_SCHEME
        );
        assert!(url
            .set_scheme("rust", SetFlags::new().non_support_scheme())
            .is_ok());

        url.set_path("/a b", SetFlags::new()).unwrap();
        assert_eq!(url.get_path(GetFlags::new()).unwrap(), "/a b");
        url.set_path("/a b", SetFlags::new().urlencode()).unwrap();
        assert_eq!(url.get_path(GetFlags::new()).unwrap(), "/a%20b");

        assert_eq!(
            url.set_url("www.rust-lang.org", SetFlags::new())
                .unwrap_err()
                .code(),
            curl_sys::CURLUE_BAD_SCHEME
        );
        assert!(url
            .set_url("www.rust-lang.org", SetFlags::new().default_scheme())
            .is_ok());
        assert_eq!(
            url.get_url(GetFlags::new()).unwrap(),
            "https://www.rust-lang.org/"
        );

        url.clear_url().unwrap();
        assert!(url
            .set_url("www.rust-lang.org", SetFlags::new().guess_scheme())
            .is_ok());
        assert_eq!(
            url.get_url(GetFlags::new()).unwrap(),
            "http://www.rust-lang.org/"
        );

        assert_eq!(
            url.set_url("http:///", SetFlags::new()).unwrap_err().code(),
            curl_sys::CURLUE_NO_HOST
        );
        assert!(url
            .set_url("http:///", SetFlags::new().no_authority())
            .is_ok());

        url.set_url("https://www.rust-lang.org/././foo", SetFlags::new())
            .unwrap();
        assert_eq!(url.get_path(GetFlags::new()).unwrap(), "/foo");
        url.set_url(
            "https://www.rust-lang.org/././foo",
            SetFlags::new().path_as_is(),
        )
        .unwrap();
        assert_eq!(url.get_path(GetFlags::new()).unwrap(), "/././foo");

        assert!(url
            .set_url("https://www.rust-lang.org/ a", SetFlags::new())
            .is_err());
        assert!(url
            .set_url(
                "https://www.rust-lang.org/ a",
                SetFlags::new().allow_space()
            )
            .is_ok());
    }
}
