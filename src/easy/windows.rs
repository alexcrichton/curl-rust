#![allow(non_camel_case_types, non_snake_case)]

    use libc:: c_void;

#[cfg(target_env = "msvc")]
mod win {

    use kernel32;
    use libc::{c_int, c_long, c_uchar, c_void};
    use std::ffi::CString;
    use std::mem;
    use std::ptr;
    use schannel::cert_context::ValidUses;
    use schannel::cert_store::CertStore;
    use winapi;

    fn lookup(module: Option<&str>, symbol: &str) -> Option<*const ::std::os::raw::c_void> {
        let symbol = CString::new(symbol).unwrap();
        unsafe {
            let mut mod_buf: Vec<u16>;
            let mod_ptr: *mut u16 = if let Some(module) = module {
                mod_buf = module.encode_utf16().collect();
                mod_buf.push(0);
                mod_buf.as_mut_ptr()
            } else {
                ptr::null_mut()
            };

            let handle = kernel32::GetModuleHandleW(mod_ptr);
            let n = kernel32::GetProcAddress(handle, symbol.as_ptr());
            if n == ptr::null() {
                None
            } else {
                Some(n)
            }
        }
    }

    pub enum X509_STORE {}
    pub enum X509 {}
    pub enum SSL_CTX {}

    type d2i_X509_fn = unsafe extern "C" fn(
        a: *mut *mut X509,
        pp: *mut *const c_uchar,
        length: c_long,
    ) -> *mut X509;
    type X509_free_fn = unsafe extern "C" fn(x: *mut X509);
    type X509_STORE_add_cert_fn = unsafe extern "C" fn(store: *mut X509_STORE, x: *mut X509)
        -> c_int;
    type SSL_CTX_get_cert_store_fn = unsafe extern "C" fn(ctx: *const SSL_CTX)
        -> *mut X509_STORE;

    struct OpenSSL {
        d2i_X509: d2i_X509_fn,
        X509_free: X509_free_fn,
        X509_STORE_add_cert: X509_STORE_add_cert_fn,
        SSL_CTX_get_cert_store: SSL_CTX_get_cert_store_fn,
    }

    fn lookup_functions(crypto_module: Option<&str>, ssl_module: Option<&str>) -> Option<OpenSSL> {
        let d2i_X509 = lookup(crypto_module, "d2i_X509");
        let X509_free = lookup(crypto_module, "X509_free");
        let X509_STORE_add_cert = lookup(crypto_module, "X509_STORE_add_cert");
        let SSL_CTX_get_cert_store = lookup(ssl_module, "SSL_CTX_get_cert_store");

        if d2i_X509.is_some() && X509_free.is_some() && X509_STORE_add_cert.is_some() &&
            SSL_CTX_get_cert_store.is_some()
        {
            unsafe {
                Some(OpenSSL {
                    d2i_X509: mem::transmute(d2i_X509.unwrap()),
                    X509_free: mem::transmute(X509_free.unwrap()),
                    X509_STORE_add_cert: mem::transmute(X509_STORE_add_cert.unwrap()),
                    SSL_CTX_get_cert_store: mem::transmute(SSL_CTX_get_cert_store.unwrap()),
                })
            }
        } else {
            None
        }
    }

    pub fn add_certs_to_context(ssl_ctx: *mut c_void) {
        unsafe {
            let openssl = if let Some(o) = lookup_functions(None, None) {
                o
            } else if let Some(o) = lookup_functions(Some("libcrypto"), Some("libssl")) {
                o
            } else if let Some(o) = lookup_functions(Some("libeay32"), Some("ssleay32")) {
                o
            } else {
                return;
            };

            let openssl_store = (openssl.SSL_CTX_get_cert_store)(ssl_ctx as *const SSL_CTX);

            let mut store = if let Ok(s) = CertStore::open_current_user("ROOT") {
                s
            } else {
                return;
            };

            for cert in store.certs() {
                let valid_uses = if let Ok(v) = cert.valid_uses() {
                    v
                } else {
                    return;
                };

                // check the extended key usage for the "Server Authentication" OID
                let is_server_auth = match valid_uses {
                    ValidUses::All => true,
                    ValidUses::OIDs(ref oids) => {
                        oids.contains(&winapi::wincrypt::szOID_PKIX_KP_SERVER_AUTH.to_owned())
                    }
                };

                if !is_server_auth {
                    continue;
                }

                let der = cert.to_der();
                let x509 =
                    (openssl.d2i_X509)(ptr::null_mut(), &mut der.as_ptr(), der.len() as c_long);
                if !x509.is_null() {
                    (openssl.X509_STORE_add_cert)(openssl_store, x509);
                    (openssl.X509_free)(x509);
                }
            }
        }
    }
}

#[cfg(target_env = "msvc")]
pub fn add_certs_to_context(ssl_ctx: *mut c_void) {
    win::add_certs_to_context(ssl_ctx)
}

#[cfg(not(target_env = "msvc"))]
pub fn add_certs_to_context(_: *mut c_void) {}