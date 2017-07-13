fn main() {
    #[cfg(target_os = "macos")]
    #[cfg(feature = "private_api")]
    println!("cargo:rustc-link-search=framework={}", "/System/Library/PrivateFrameworks");
}
