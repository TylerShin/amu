use std::{env, path::PathBuf, process::Command};

fn main() {
    let host = env::var("HOST").unwrap();
    let target = env::var("TARGET").unwrap();
    assert!(
        host == target || (host.ends_with("windows-msvc") && target.ends_with("windows-msvc")),
        "Build on the target OS; Windows builds select the MSVC architecture before invoking Cargo"
    );
    let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../third_party/libhangul");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-env-changed=CC");
    if target_os == "windows" {
        build_windows(&source, &output);
        return;
    }
    let library = output.join(match target_os.as_str() {
        "macos" => "libhangul.dylib",
        "linux" => "libhangul.so",
        _ => {
            panic!("libhangul build supports macOS, Linux and Windows MSVC")
        }
    });

    let mut compiler = Command::new(env::var_os("CC").unwrap_or_else(|| "clang".into()));
    compiler.args(["-std=c11", "-O2", "-fPIC", "-DENABLE_EXTERNAL_KEYBOARDS=0"]);
    if target_os == "macos" {
        compiler.args(["-dynamiclib", "-mmacosx-version-min=13.0", "-install_name"]);
        compiler.arg(&library);
    } else {
        // Strict C11 hides POSIX declarations such as strdup on glibc.
        compiler.args(["-shared", "-D_POSIX_C_SOURCE=200809L"]);
        compiler.arg(format!("-Wl,-soname,{}", library.display()));
    }
    for file in ["hangulctype.c", "hangulinputcontext.c", "hangulkeyboard.c"] {
        compiler.arg(source.join(file));
    }
    let status = compiler
        .arg("-o")
        .arg(&library)
        .status()
        .expect("clang is required to build libhangul");
    assert!(
        status.success(),
        "libhangul shared-library compilation failed"
    );
    println!("cargo:rustc-link-search=native={}", output.display());
    println!("cargo:rustc-link-lib=dylib=hangul");
}

fn build_windows(source: &std::path::Path, output: &std::path::Path) {
    let exports = output.join("hangul.def");
    std::fs::write(&exports, "LIBRARY hangul\nEXPORTS\nhangul_ic_new\nhangul_ic_delete\nhangul_ic_process\nhangul_ic_backspace\nhangul_ic_reset\nhangul_ic_get_commit_string\nhangul_ic_get_preedit_string\n").unwrap();
    let mut compiler = Command::new("cl.exe");
    compiler.current_dir(output).args([
        "/nologo",
        "/std:c11",
        "/O2",
        "/LD",
        "/MD",
        "/utf-8",
        "/DENABLE_EXTERNAL_KEYBOARDS=0",
        "/D_CRT_SECURE_NO_WARNINGS",
    ]);
    for file in ["hangulctype.c", "hangulinputcontext.c", "hangulkeyboard.c"] {
        compiler.arg(source.join(file));
    }
    let status = compiler
        .arg("/Fe:hangul.dll")
        .arg("/link")
        .arg(format!("/DEF:{}", exports.display()))
        .arg("/IMPLIB:hangul.lib")
        .status()
        .expect("MSVC is required; run the Windows mise tasks to select the compiler environment");
    assert!(status.success(), "libhangul DLL compilation failed");
    println!("cargo:rustc-link-search=native={}", output.display());
    println!("cargo:rustc-link-lib=dylib=hangul");
}
