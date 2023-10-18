// build.rs

fn main() {
    println!("cargo:include=./lib/vosk");
    println!("cargo:rustc-link-search=./lib/vosk");

    println!("cargo:rustc-env=LD_LIBRARY_PATH=./lib/vosk");
}
