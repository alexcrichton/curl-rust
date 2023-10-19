/// This is an example how to use Easy2 to download a file.
/// Can able to resume a download and control download speed.
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use curl::easy::{Easy2, Handler, WriteError};

#[derive(Clone)]
enum Collector {
    File(PathBuf),
    Ram(Vec<u8>),
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
            Collector::Ram(container) => {
                container.extend_from_slice(data);
                Ok(data.len())
            }
        }
    }
}

impl Collector {
    fn get_response_body(&self) -> Option<Vec<u8>> {
        match self {
            Collector::File(_) => None,
            Collector::Ram(container) => Some(container.clone()),
        }
    }
}

fn example_file_download() {
    // File Download
    let target_path = PathBuf::from("<SAVE DOWNLOADED FILE HERE>");
    let collector = Collector::File(target_path.clone());
    let mut easy2 = Easy2::new(collector);

    easy2
        .url("<INPUT YOUR TARGET DOWNLOAD LOCATION HERE>")
        .unwrap();
    easy2.get(true).unwrap();
    // Download the actual file from actual location
    easy2.follow_location(true).unwrap();
    // Setting of download speed control in bytes per second
    easy2.max_recv_speed(2000).unwrap();
    // Can resume download by giving a byte offset
    easy2.resume_from(0).unwrap();
    easy2.perform().unwrap();

    let status_code = easy2.response_code().unwrap();
    let response_body = easy2.get_ref().get_response_body().take();
    let content_type = easy2.content_type();

    eprintln!("Status Code: {}", status_code);
    eprintln!("content-type: {:?}", content_type);
    eprintln!("Response Body: {:?}", response_body);

    let file = fs::metadata(target_path.clone()).unwrap();
    assert!(file.len() != 0);
}

fn example_response_as_body() {
    // Get Response as Body
    let collector = Collector::Ram(Vec::new());
    let mut easy2 = Easy2::new(collector);

    easy2
        .url("<INPUT YOUR TARGET DOWNLOAD LOCATION HERE>")
        .unwrap();
    easy2.get(true).unwrap();
    easy2.perform().unwrap();

    let status_code = easy2.response_code().unwrap();
    let response_body = easy2.get_ref().get_response_body().take();
    let content_type = easy2.content_type();

    eprintln!("Status Code: {}", status_code);
    eprintln!("content-type: {:?}", content_type);
    eprintln!("Response Body: {:?}", response_body);
}

fn main() {
    example_file_download();
    example_response_as_body();
}
