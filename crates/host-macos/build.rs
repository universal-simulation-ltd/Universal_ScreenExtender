fn main() {
    #[cfg(target_os = "macos")]
    {
        compile_virtual_display_shim();
        configure_swift_runtime();
    }
}

#[cfg(target_os = "macos")]
fn compile_virtual_display_shim() {
    println!("cargo:rerun-if-changed=shim/virtual_display.m");
    cc::Build::new()
        .file("shim/virtual_display.m")
        .flag("-fobjc-arc")
        .compile("extender_vdisplay_shim");
    println!("cargo:rustc-link-lib=framework=CoreGraphics");
    println!("cargo:rustc-link-lib=framework=Foundation");
}

#[cfg(target_os = "macos")]
fn configure_swift_runtime() {
    use std::path::Path;
    use std::process::Command;

    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");

    let Ok(output) = Command::new("xcode-select").arg("-p").output() else {
        return;
    };
    let dev_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if dev_dir.is_empty() {
        return;
    }

    let candidates = [
        format!("{dev_dir}/usr/lib/swift/macosx"),
        format!("{dev_dir}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx"),
    ];
    for path in candidates {
        if Path::new(&path).is_dir() {
            println!("cargo:rustc-link-search=native={path}");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{path}");
        }
    }
}
