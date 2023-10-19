/// This is an example how to use Easy2 to download a file.
/// Can able to resume a download and control download speed.
use std::{fs::OpenOptions, io::Write, path::PathBuf};

use curl::easy::{Easy2, Handler, WriteError};

enum Collector {
    File(PathBuf),
    // You can add what type of storage you want
}

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        match self {
            Collector::File(download_path) => {
                println!("File chunk size: {}", data.len());
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(download_path)
                    .map_err(|e| {
                        eprintln!("{}", e);
                        WriteError::Pause
                    })?;

                file.write_all(data).map_err(|e| {
                    eprintln!("{}", e);
                    WriteError::Pause
                })?;
                Ok(data.len())
            }
        }
    }
}
fn main() {
    let collector = Collector::File(PathBuf::from("<YOUR TARGET PATH>"));
    let mut easy2 = Easy2::new(collector);

    easy2.url("<YOUR DOWNLOAD LOCATION>").unwrap();
    easy2.get(true).unwrap();
    // Download the actual file from actual location
    easy2.follow_location(true).unwrap();
    // Setting of download speed control in bytes per second
    easy2.max_recv_speed(2000).unwrap();
    // Can resume download by giving a byte offset
    easy2.resume_from(0).unwrap();
    easy2.perform().unwrap();

    let status_code = easy2.response_code().unwrap();
    let content_type = easy2.content_type().unwrap();
    eprintln!("Content Type: {:?}", content_type);
    eprintln!("Status Code: {}", status_code);
}
