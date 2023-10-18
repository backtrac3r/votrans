fn main() {
    println!("cargo:include=./lib/linux_vosk");
    println!("cargo:rustc-link-search=./lib/linux_vosk");
}
