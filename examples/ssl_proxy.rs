use anyhow::Result;

fn main() -> Result<()> {
    let mut handle = curl::easy::Easy::new();

    let proxy_url = "https://fwdproxy";
    let proxy_port = 8082;
    let cainfo = "/var/credentials/root/ca.pem";
    let sslcert = "/var/credentials/user/x509.pem";
    let sslkey = "/var/credentials/user/x509.pem";

    handle.connect_timeout(std::time::Duration::from_secs(5))?;
    handle.connect_only(true)?;
    handle.verbose(true)?;
    handle.url("https://www.google.com")?;

    handle.proxy(proxy_url)?;
    handle.proxy_port(proxy_port)?;
    handle.proxy_cainfo(cainfo)?;
    handle.proxy_sslcert(sslcert)?;
    handle.proxy_sslkey(sslkey)?;
    println!("ssl proxy setup done");

    handle.perform()?;
    println!("connected done");
    Ok(())
}
