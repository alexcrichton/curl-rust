#![allow(non_camel_case_types, non_snake_case)]

use libc::c_void;

#[cfg(target_env = "msvc")]
mod win {
    extern crate winapi;
    use self::winapi::ctypes::*;
    use self::winapi::um::libloaderapi::*;
    use self::winapi::um::wincrypt::*;
    use schannel::cert_context::ValidUses;
    use schannel::cert_store::CertStore;
    use std::ffi::CString;
    use std::mem;
    use std::ptr;

    fn lookup(module: &str, symbol: &str) -> Option<*const c_void> {
        unsafe {
            let symbol = CString::new(symbol).unwrap();
            let mut mod_buf: Vec<u16> = module.encode_utf16().collect();
            mod_buf.push(0);
            let handle = GetModuleHandleW(mod_buf.as_mut_ptr());
            let n = GetProcAddress(handle, symbol.as_ptr());
            if n == ptr::null_mut() {
                None
            } else {
                Some(n as *const c_void)
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
    type X509_STORE_add_cert_fn =
        unsafe extern "C" fn(store: *mut X509_STORE, x: *mut X509) -> c_int;
    type SSL_CTX_get_cert_store_fn = unsafe extern "C" fn(ctx: *const SSL_CTX) -> *mut X509_STORE;

    struct OpenSSL {
        d2i_X509: d2i_X509_fn,
        X509_free: X509_free_fn,
        X509_STORE_add_cert: X509_STORE_add_cert_fn,
        SSL_CTX_get_cert_store: SSL_CTX_get_cert_store_fn,
    }

    unsafe fn lookup_functions(crypto_module: &str, ssl_module: &str) -> Option<OpenSSL> {
        macro_rules! get {
            ($(let $sym:ident in $module:expr;)*) => ($(
                let $sym = match lookup($module, stringify!($sym)) {
                    Some(p) => p,
                    None => return None,
                };
            )*)
        }
        get! {
            let d2i_X509 in crypto_module;
            let X509_free in crypto_module;
            let X509_STORE_add_cert in crypto_module;
            let SSL_CTX_get_cert_store in ssl_module;
        }
        Some(OpenSSL {
            d2i_X509: mem::transmute(d2i_X509),
            X509_free: mem::transmute(X509_free),
            X509_STORE_add_cert: mem::transmute(X509_STORE_add_cert),
            SSL_CTX_get_cert_store: mem::transmute(SSL_CTX_get_cert_store),
        })
    }

    pub unsafe fn add_certs_to_context(ssl_ctx: *mut c_void) {
        // check the runtime version of OpenSSL
        let openssl = match ::version::Version::get().ssl_version() {
            Some(ssl_ver) if ssl_ver.starts_with("OpenSSL/1.1.0") => {
                lookup_functions("libcrypto", "libssl")
            }
            Some(ssl_ver) if ssl_ver.starts_with("OpenSSL/1.0.2") => {
                lookup_functions("libeay32", "ssleay32")
            }
            _ => return,
        };
        let openssl = match openssl {
            Some(s) => s,
            None => return,
        };

        let openssl_store = (openssl.SSL_CTX_get_cert_store)(ssl_ctx as *const SSL_CTX);
        let store = match CertStore::open_current_user("ROOT") {
            Ok(s) => s,
            Err(_) => return,
        };

        for cert in store.certs() {
            let valid_uses = match cert.valid_uses() {
                Ok(v) => v,
                Err(_) => continue,
            };

            // check the extended key usage for the "Server Authentication" OID
            match valid_uses {
                ValidUses::All => {}
                ValidUses::Oids(ref oids) => {
                    let oid = szOID_PKIX_KP_SERVER_AUTH.to_owned();
                    if !oids.contains(&oid) {
                        continue;
                    }
                }
            }

            let der = cert.to_der();
            let x509 = (openssl.d2i_X509)(ptr::null_mut(), &mut der.as_ptr(), der.len() as c_long);
            if !x509.is_null() {
                (openssl.X509_STORE_add_cert)(openssl_store, x509);
                (openssl.X509_free)(x509);
            }
        }
    }
}

#[cfg(target_env = "msvc")]
pub fn add_certs_to_context(ssl_ctx: *mut c_void) {
    unsafe {
        win::add_certs_to_context(ssl_ctx as *mut _);
    }
}

#[cfg(not(target_env = "msvc"))]
pub fn add_certs_to_context(_: *mut c_void) {}
