use std::{env, str::FromStr, process::Command, fs};

fn run_python(file: &str, cwd: &str) -> bool {
    Command::new("python3")
        .arg(fs::canonicalize(file).unwrap())
        .current_dir(cwd)
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    fs::create_dir_all(format!("{}/deps", out_dir)).unwrap();
    run_python("./scripts/update_deps.py", format!("{}/deps", out_dir).as_str());

    cmake::Config::new(".")
        .configure_arg(format!("-C{}/deps/helper.cmake", out_dir))
        .build();

    println!("cargo:rustc-link-search={}/lib", out_dir);

    if target_os == "windows" {
        println!("cargo:rustc-link-lib=vulkan-1");
    } else if target_os == "linux" {
        println!("cargo:rustc-link-lib=vulkan");
    }

    let mut bindgen_builder = bindgen::Builder::default()
        .header(format!("{}/deps/Vulkan-Headers/build/install/include/vulkan/vulkan.h", out_dir.as_str()))
        .prepend_enum_name(false)
        .layout_tests(false)
        .allowlist_type("Vk.*")
        .allowlist_function("vk.*")
        .allowlist_var("VK_.*");

    if target_os == "windows" {
        bindgen_builder = bindgen_builder.clang_arg("-DVK_USE_PLATFORM_WIN32_KHR");
    }

    if env::var("CARGO_FEATURE_XCB_EXTENSIONS").is_ok() {
        bindgen_builder = bindgen_builder.clang_arg("-DVK_USE_PLATFORM_XCB_KHR");
    }

    if env::var("CARGO_FEATURE_WAYLAND_EXTENSIONS").is_ok() {
        bindgen_builder = bindgen_builder.clang_arg("-DVK_USE_PLATFORM_WAYLAND_KHR");
    }

    let mut bindgen_out_file = std::path::PathBuf::from_str(out_dir.as_str()).unwrap();
    bindgen_out_file.push("vulkan.rs");
    bindgen_builder
        .generate()
        .expect("Failed to generate bindings for vulkan/vulkan.h!")
        .write_to_file(bindgen_out_file.to_str().unwrap())
        .expect("Failed to write bindings to a disk.");
}
