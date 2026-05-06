use curl::{easy::Easy, multi::Multi};
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

fn main() -> Result<(), curl::Error> {
    // Set up a multi handle using the multi_socket API.
    let mut multi = Multi::new();
    let timer = Arc::new(Mutex::new(None));

    // Set a socket callback, but the bug here is that this callback will never
    // be invoked, preventing curl from ever making any progress.
    multi
        .socket_function(|_, _, _| panic!("socket callback was actually invoked"))
        .unwrap();

    multi
        .timer_function({
            let timer = timer.clone();
            move |timeout| {
                *timer.try_lock().unwrap() = dbg!(timeout);
                true
            }
        })
        .unwrap();

    // Prepare a request to send.
    let mut easy = Easy::new();
    easy.url("https://example.com/")?;
    easy.verbose(true)?;
    // Don't need to wait long to see the bug.
    easy.timeout(Duration::from_secs(5))?;

    // Attach to multi handle. This will call our timer callback with a zero
    // timeout to start things.
    let _handle = multi.add(easy).unwrap();

    // Start things off.
    let mut running = multi.timeout().unwrap();

    // Loop while our easy handle is still running.
    while running > 0 {
        // In a real application we would poll some sockets, but it doesn't
        // matter here since curl never gives us any.
        sleep(timer.try_lock().unwrap().unwrap_or_default());

        // Let curl know that its timer expired and perform some work.
        running = multi.timeout().unwrap();

        multi.messages(|message| {
            dbg!(message);
        });
    }

    Ok(())
}
