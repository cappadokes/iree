#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn say_hello();
    }
}

fn say_hello() {
    println!("Hello from Rust!");
}