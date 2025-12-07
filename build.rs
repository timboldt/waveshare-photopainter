use std::process::Command;

fn main() {
    // Get the current date and time
    let output = Command::new("date")
        .args(["+%Y-%m-%d %H:%M:%S"])
        .output()
        .expect("Failed to get build date");

    let build_date = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 in date output")
        .trim()
        .to_string();

    // Set as environment variable for compile-time inclusion
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // Rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}
