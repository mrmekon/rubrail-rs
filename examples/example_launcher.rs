
fn main() {
    let exe = std::env::current_exe().unwrap();
    let mut exe_dir = exe.clone();
    exe_dir.pop();
    let mut resource_dir = exe_dir.clone();
    resource_dir.pop();
    resource_dir.pop();
    resource_dir.pop();
    resource_dir.push("examples");
    let mut example_app = exe_dir.clone();
    example_app.push("RubrailExample.app");
    let mut icon_src = resource_dir.clone();
    icon_src.push("icon.png");
    let mut script_src = resource_dir.clone();
    script_src.push("example.sh");
    let mut icon_dst = exe_dir.clone();
    icon_dst.push("icon.png");
    let mut script_dst = exe_dir.clone();
    script_dst.push("example.sh");
    let script_exe = script_dst.clone();
    std::fs::copy(icon_src, icon_dst).unwrap();
    std::fs::copy(script_src, script_dst).unwrap();
    let _ = std::process::Command::new(script_exe)
        .output()
        .expect("Failed to run bundling script");
    let _ = std::process::Command::new("open")
        .arg(example_app)
        .output()
        .expect("Failed to launch app bundle");
}
