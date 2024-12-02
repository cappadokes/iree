fn main() {
    cxx_build::bridge("src/lib.rs")
        .compile("toy-cxx");

    println!("cargo:rerun-if-changed=src/lib.rs");
}