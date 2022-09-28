use std::env;
use std::io::{stdout, Write};

use anyhow::{bail, Result};
use curl::easy::Easy;

fn main() -> Result<()> {
    let argv = env::args().collect::<Vec<_>>();
    if argv.len() < 4 {
        bail!("usage: ssl_client_auth URL CERT KEY CAINFO? PASSWORD?");
    }
    let url = &argv[1];
    let cert_path = &argv[2];
    let key_path = &argv[3];
    let cainfo = if argv.len() >= 5 {
        Some(&argv[4])
    } else {
        None
    };
    let password = if argv.len() >= 6 {
        Some(&argv[5])
    } else {
        None
    };

    let mut handle = Easy::new();

    handle.url(url)?;
    handle.verbose(true)?;
    handle.write_function(|data| {
        stdout().write_all(data).unwrap();
        Ok(data.len())
    })?;

    handle.ssl_cert(&cert_path)?;
    handle.ssl_key(&key_path)?;
    if let Some(password) = password {
        handle.key_password(password)?;
    }
    if let Some(cainfo) = cainfo {
        handle.cainfo(cainfo)?;
    }

    handle.perform()?;
    Ok(())
}
