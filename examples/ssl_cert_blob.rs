use std::env;
use std::fs::File;
use std::io::{stdout, Read, Write};
use std::path::Path;

use anyhow::{bail, Result};
use curl::easy::Easy;

fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let mut f = File::open(path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(buf)
}

fn main() -> Result<()> {
    let argv = env::args().collect::<Vec<_>>();
    if argv.len() < 4 {
        bail!("usage: ssl_cert_blob URL CERT KEY");
    }
    let url = &argv[1];
    let cert_path = &argv[2];
    let key_path = &argv[3];

    let mut handle = Easy::new();

    handle.url(url)?;
    handle.verbose(true)?;
    handle.write_function(|data| {
        stdout().write_all(data).unwrap();
        Ok(data.len())
    })?;

    let cert_blob = read_file(cert_path)?;
    let key_blob = read_file(key_path)?;

    handle.ssl_cert_blob(&cert_blob)?;
    handle.ssl_key_blob(&key_blob)?;

    handle.perform()?;
    Ok(())
}
