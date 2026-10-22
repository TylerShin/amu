use std::{env, path::PathBuf, process::Command};

fn main() {
    assert_eq!(
        env::var("HOST").unwrap(),
        env::var("TARGET").unwrap(),
        "Cross compilation is not configured yet; build on the target host"
    );
    let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../third_party/libhangul");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let library = output.join(match target_os.as_str() {
        "macos" => "libhangul.dylib",
        "linux" => "libhangul.so",
        _ => {
            panic!("libhangul build currently supports macOS and Linux; Windows adapter is pending")
        }
    });

    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-env-changed=CC");
    let mut compiler = Command::new(env::var_os("CC").unwrap_or_else(|| "clang".into()));
    compiler.args(["-std=c11", "-O2", "-fPIC", "-DENABLE_EXTERNAL_KEYBOARDS=0"]);
    if target_os == "macos" {
        compiler.args(["-dynamiclib", "-mmacosx-version-min=13.0", "-install_name"]);
        compiler.arg(&library);
    } else {
        compiler.arg("-shared");
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
