fn main() {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        if Command::new("pkg-config")
            .args(&["--exists", "x11"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            println!("cargo:rustc-env=GDK_BACKEND=x11");
        }
    }

    tauri_build::build()
}
