use std::{env::var, str::FromStr};

fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    cmake::Config::new(".")
        .define("UPDATE_DEPS", "on")
        .define("CMAKE_BUILD_TYPE", profile)
        .build();

    println!("cargo:rustc-link-search={}/lib", out_dir);

    if target_os == "windows" {
        println!("cargo:rustc-link-lib=vulkan-1");
    } else if target_os == "linux" {
        println!("cargo:rustc-link-lib=vulkan");
    }

    let mut bindgen_builder = bindgen::Builder::default()
        .header("external/Vulkan-Headers/build/install/include/vulkan/vulkan.h")
        .prepend_enum_name(false)
        .layout_tests(false)
        .blocklist_type("_IMAGE_TLS_DIRECTORY64")
        .blocklist_type("IMAGE_TLS_DIRECTORY64")
        .blocklist_type("IMAGE_TLS_DIRECTORY")
        .blocklist_type("PIMAGE_TLS_DIRECTORY64")
        .blocklist_type("PIMAGE_TLS_DIRECTORY");

    if target_os == "windows" {
        bindgen_builder = bindgen_builder.clang_arg("-DVK_USE_PLATFORM_WIN32_KHR");
    }

    if var("CARGO_FEATURE_XCB_EXTENSIONS").is_ok() {
        bindgen_builder = bindgen_builder.clang_arg("-DVK_USE_PLATFORM_XCB_KHR");
    }

    if var("CARGO_FEATURE_WAYLAND_EXTENSIONS").is_ok() {
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
