extern crate bindgen;

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use git2::Repository;

static IREE_SAMPLES_REPO: &str = "https://github.com/iree-org/iree-samples";
static IREE_REPO: &str = "https://github.com/iree-org/iree";

fn shallow_clone(path: &Path, repo: &str) -> Repository {
    let mut child = Command::new("git")
        .args(&[
            "clone",
            "--depth",
            "1",
            "--recurse-submodules",
            "--shallow-submodules",
            "-j10",
            repo,
            path.to_str().unwrap(),
        ])
        .spawn()
        .expect("failed to execute process");
    child.wait().unwrap();

    git2::Repository::open(path).unwrap()
}

/// use cached repo if it exists, otherwise clone it
fn get_repo(path: &Path, repo: &str) -> git2::Repository {
    println!("Checking for cached repo at: {}", path.to_str().unwrap());
    if path.exists() {
        git2::Repository::open(path).unwrap()
    } else {
        // shallow clone
        shallow_clone(path, repo)
    }
}

/// Clones the IREE repository and builds it.
fn clone_and_build_iree(out_dir: &Path) -> PathBuf {
    // clone IREE repo
    let iree_dir = out_dir.join("iree");
    let iree = get_repo(iree_dir.as_path(), IREE_REPO);

    // clone IREE samples repo
    let iree_samples = get_repo(&out_dir.join("iree-samples"), IREE_SAMPLES_REPO);

    // make build directory
    let mut iree_samples_build_path = out_dir.join("iree-samples-build");
    if iree_samples_build_path.exists() {
        // already built!
        return iree_samples_build_path;
    }
    std::fs::create_dir_all(iree_samples_build_path.clone()).unwrap();

    // build iree-samples
    cmake::Config::new(out_dir.join("iree-samples/runtime-library"))
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("CMAKE_C_COMPILER", "clang")
        .define("CMAKE_CXX_COMPILER", "clang++")
        .define(
            "IREE_ROOT_DIR",
            out_dir
                .join("iree")
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .out_dir(iree_samples_build_path.clone())
        .build();

    // add library path to linker

    iree_samples_build_path
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let iree_build_dir = clone_and_build_iree(out_path.as_path());
    println!(
        "cargo:rustc-link-search={}",
        iree_build_dir.join("build/lib").to_str().unwrap()
    );

    // add built third party libraries to linker
    // cpuinfo
    println!(
        "cargo:rustc-link-search={}",
        iree_build_dir
            .join("build/iree_core/third_party/cpuinfo/")
            .to_str()
            .unwrap()
    );

    // flatcc
    println!(
        "cargo:rustc-link-search={}",
        iree_build_dir
            .join("build/iree_core/build_tools/third_party/flatcc/")
            .to_str()
            .unwrap()
    );

    // clog
    println!(
        "cargo:rustc-link-search={}",
        iree_build_dir
            .join("build/iree_core/third_party/cpuinfo/deps/clog/")
            .to_str()
            .unwrap()
    );
    let iree_include_dir = iree_build_dir.as_path().join("build/include");

    println!("cargo:rustc-link-lib=iree");

    // third party libraries
    println!("cargo:rustc-link-lib=cpuinfo");
    println!("cargo:rustc-link-lib=flatcc_parsing");
    println!("cargo:rustc-link-lib=clog");
    println!("cargo:rustc-link-lib=stdc++");

    // gather all api headers we want
    let iree_api_headers = ["iree/runtime/api.h"];

    for &header in iree_api_headers.iter() {
        let header_out = Path::new(header)
            .to_str()
            .and_then(|s| s.strip_suffix(".h"))
            .and_then(|s| Some(format!("{}.rs", s)))
            .unwrap();

        if out_path.join(header_out.clone()).exists() {
            // already generated
            continue;
        }
        let header_buf = iree_include_dir.join(header);
        let header_path = header_buf.as_path();

        let dir = out_path.join(Path::new(header).parent().unwrap());

        if !dir.exists() {
            std::fs::create_dir_all(&dir).expect("Unable to create directory");
        }

        let bindings = bindgen::Builder::default()
            .header(header_path.to_str().unwrap())
            .clang_arg(format!("-I{}", iree_include_dir.to_str().unwrap()))
            .default_enum_style(bindgen::EnumVariation::NewType {
                is_bitfield: true,
                is_global: true,
            })
            .generate_inline_functions(false)
            .derive_default(true)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(out_path.join(header_out))
            .expect("Couldn't write bindings!");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
