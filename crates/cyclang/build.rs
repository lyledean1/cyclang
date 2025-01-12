use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let which_output = Command::new("which")
        .arg("llvm-config")
        .output()
        .expect("Failed to execute `which`. Make sure it's installed and available in PATH.");

    if !which_output.status.success() {
        panic!("Could not find `codegen-config`. Make sure codegen is installed.");
    }

    let which_str = String::from_utf8_lossy(&which_output.stdout);
    let llvm_config_path = which_str.trim();

    let llvm_config_output = Command::new(llvm_config_path)
        .arg("--version")
        .output()
        .expect("Failed to execute codegen-config.");

    if !llvm_config_output.status.success() {
        panic!("codegen-config execution failed");
    }

    let version_str = String::from_utf8_lossy(&llvm_config_output.stdout);
    let version_str = version_str.trim();

    println!("Found LLVM version: {}", version_str);

    if !version_str.starts_with("19") {
        panic!(
            "Unsupported LLVM version: {}. LLVM 19.x.x is required.",
            version_str
        );
    }

    let llvm_dir = Path::new(llvm_config_path)
        .parent()
        .expect("Failed to get parent directory of codegen-config")
        .parent()
        .expect("Failed to get LLVM directory");

    let llvm_dir_str = llvm_dir.to_str().expect("Failed to convert path to string");

    env::set_var("LLVM_SYS_190_PREFIX", llvm_dir_str);
    println!("cargo:rerun-if-changed=build.rs");
}
