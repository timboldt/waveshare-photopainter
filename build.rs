use std::process::Command;

fn main() {
    // Get the current date and time using cross-platform approach
    let build_date = if cfg!(unix) {
        // Unix-like systems (Linux, macOS)
        Command::new("date")
            .args(["+%Y-%m-%d %H:%M:%S"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    } else if cfg!(windows) {
        // Windows - use PowerShell Get-Date
        Command::new("powershell")
            .args(["-Command", "Get-Date -Format 'yyyy-MM-dd HH:mm:ss'"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "unknown".to_string()
    };

    // Set as environment variable for compile-time inclusion
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // Rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}
