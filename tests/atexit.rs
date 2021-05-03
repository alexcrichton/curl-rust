use curl::easy::Easy;

pub extern "C" fn hook() {
    let mut easy = Easy::new();
    easy.url("google.com").unwrap();
    easy.write_function(|data| Ok(data.len())).unwrap();
    easy.perform().unwrap();
}

fn main() {
    curl::init();
    hook();
    unsafe {
        libc::atexit(hook);
    }
    println!("Finishing...")
}
