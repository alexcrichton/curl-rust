use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;

use curl::easy::{Easy2, Handler, WriteError};
use curl::multi::{Easy2Handle, Multi};

const URLS: &[&str] = &[
    "https://www.microsoft.com",
    "https://www.google.com",
    "https://www.amazon.com",
    "https://www.apple.com",
];

struct Collector(Vec<u8>);
impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

fn download(multi: &mut Multi, token: usize, url: &str) -> Result<Easy2Handle<Collector>> {
    let version = curl::Version::get();
    let mut request = Easy2::new(Collector(Vec::new()));
    request.url(&url)?;
    request.useragent(&format!("curl/{}", version.version()))?;

    let mut handle = multi.add2(request)?;
    handle.set_token(token)?;
    Ok(handle)
}

fn main() -> Result<()> {
    let mut multi = Multi::new();
    let mut handles = URLS
        .iter()
        .enumerate()
        .map(|(token, url)| Ok((token, download(&mut multi, token, url)?)))
        .collect::<Result<HashMap<_, _>>>()?;

    let mut still_alive = true;
    while still_alive {
        // We still need to process the last messages when
        // `Multi::perform` returns "0".
        if multi.perform()? == 0 {
            still_alive = false;
        }

        multi.messages(|message| {
            let token = message.token().expect("failed to get the token");
            let handle = handles
                .get_mut(&token)
                .expect("the download value should exist in the HashMap");

            match message
                .result_for2(&handle)
                .expect("token mismatch with the `EasyHandle`")
            {
                Ok(()) => {
                    let http_status = handle
                        .response_code()
                        .expect("HTTP request finished without status code");

                    println!(
                        "R: Transfer succeeded (Status: {}) {} (Download length: {})",
                        http_status,
                        URLS[token],
                        handle.get_ref().0.len()
                    );
                }
                Err(error) => {
                    println!("E: {} - <{}>", error, URLS[token]);
                }
            }
        });

        if still_alive {
            // The sleeping time could be reduced to allow other processing.
            // For instance, a thread could check a condition signalling the
            // thread shutdown.
            multi.wait(&mut [], Duration::from_secs(60))?;
        }
    }

    Ok(())
}
