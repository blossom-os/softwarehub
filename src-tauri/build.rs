fn main() {
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-env=GTK_CSD=0");
        println!("cargo:rustc-env=GTK_USE_PORTAL=0");
        println!("cargo:rustc-env=SWT_GTK3=1");
    }

    tauri_build::build()
}
