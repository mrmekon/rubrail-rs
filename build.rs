#[allow(dead_code)]
fn add_xcode_private_framework_path() {
    // PrivateFramework dir:
    // `xcode-select -p`/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/System/Library/PrivateFrameworks/
    let xcode_dir = std::process::Command::new("xcode-select")
        .arg("-p")
        .output()
        .expect("Failed to run xcode-select");
    let mut xcode_dir = String::from_utf8(xcode_dir.stdout).unwrap();
    xcode_dir.pop(); // remove trailing newline
    let framework_dir = format!("{}/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/System/Library/PrivateFrameworks/",
                                xcode_dir);
    println!("XCode PrivateFramework dir: {}", framework_dir);

    println!("cargo:rustc-link-search=framework={}", framework_dir);
}

fn main() {
    #[cfg(target_os = "macos")]
    #[cfg(feature = "private_api")]
    add_xcode_private_framework_path();
}
