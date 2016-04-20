extern crate curl;

fn main() {
    // let data = 2;
    let mut easy = curl::easy::Easy::new();
    // easy.debug_function(&mut |_, _| {});
    let _ = easy.perform();
}
