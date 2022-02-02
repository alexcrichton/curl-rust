use curl::easy::Easy;
use std::io::{stdout, Write};

fn main() -> Result<(), curl::Error> {
    let mut curl = Easy::new();

    curl.url("https://example.com")?;
    curl.doh_url(Some("https://cloudflare-dns.com/dns-query"))?;
    curl.write_function(|data| {
        stdout().write_all(data).unwrap();
        Ok(data.len())
    })?;

    curl.perform()?;

    Ok(())
}
