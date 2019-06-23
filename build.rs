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

    let private_framework_dirs = vec![
        format!("{}/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/System/Library/PrivateFrameworks/",
                xcode_dir),
        "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/System/Library/PrivateFrameworks/DFRFoundation.framework/".into(),
    ];
    let framework_dir = private_framework_dirs.iter().filter(|dir| {
        std::path::Path::new(dir).is_dir()
    }).next().expect("XCode PrivateFramework directory not found.");

    println!("XCode PrivateFramework dir: {}", framework_dir);
    println!("cargo:rustc-link-search=framework={}", framework_dir);
}

fn main() {
    #[cfg(target_os = "macos")]
    #[cfg(feature = "private_api")]
    add_xcode_private_framework_path();
}
