# Vulkan Loader

This crate contains a raw FFI binding to the official Vulkan ICD Loader from the Khronos Group. The Vulkan Loader is a library, written primarily in C, that is designed to locate and load the Vulkan function pointers so that they can be used by an application. It is included as part of the Vulkan SDK, but can be used as a separate unit as well.

Because it is a mere FFI binding, this crate will expose the raw C interfaces to Vulkan. It does not and will not offer any helper functions, traits, lifetime annotations, or builder patterns that may exist in other Vulkan bindings for Rust. As a result, the Vulkan API functions will be the same as the functions will be the same as the functions in a C or C++ Vulkan applications. This may make it easier for you to follow a Vulkan tutorial written for C++ developers, but for practical applications this crate probably is not the wisest choice.

Due to the fact that I, the author of this binding, do not own a Mac and knows nothing about development on a Mac, macOS and iOS are currently not supported. If you want macOS or iOS support, feel free to open a pull request. Android isn't supported either for a similar reason. The Nintendo Switch is also not supported, but for a different reason.

Please open an issue in [this repository](https://github.com/earthtraveller1/Vulkan-Loader-sys-rs) if you have problems adding this crate to your project.

## Introduction

Vulkan is an explicit API, enabling direct control over how GPUs actually work. As such, Vulkan supports systems that have multiple GPUs, each running with a different driver, or ICD (Installable Client Driver). Vulkan also supports multiple global contexts (instances, in Vulkan terminology). The ICD loader is a library that is placed between a Vulkan application and any number of Vulkan drivers, in order to support multiple drivers and the instance-level functionality that works across these drivers. Additionally, the loader manages inserting Vulkan layer libraries, such as validation layers, between an application and the drivers.

This repository contains the FFI bindings for the Vulkan loader that is used for Linux and Windows. The repository also supports macOS and iOS, but the bindings currently do not. There is also a separate loader, maintained by Google, which is used on Android, but not covered by the bindings.

## Using this crate

The first thing you need to do is install a number of prerequesites.

- [LLVM Clang](https://clang.llvm.org/). This is required for `bindgen` to parse the header files and generate the bindings.
- A C/C++ compiler of your choice. This is required to compile the Vulkan Loader itself.
- [CMake](https://cmake.org). This is required to build the Vulkan Loader itself.
- [Python](https://python.org). This is required for downloading and configuring the C/C++ dependencies that the Vulkan Loader itself relies on.

After you have you prerequesites installed, you can simply do `cargo add vulkan_loader_sys` to add it as a dependency to your project.

## Example

Here is an example of an application that uses this crate to interact with the Vulkan API. It creates a Vulkan instance, destroys it, and immediately exits. It should give you a basic idea of how this binding is structured.

```rust
use vulkan_loader_sys::*;
use std::ptr::{null, null_mut};

fn main() {
    let application_info = VkApplicationInfo {
        sType: VK_STRUCTURE_TYPE_APPLICATION_INFO,
        pNext: null(),
        pApplicationName: b"Triangle Example\0".as_ptr() as *const i8,
        applicationVersion: 1,
        pEngineName: null(),
        engineVersion: 0,
        apiVersion: VK_MAKE_API_VERSION(0, 1, 2, 0),
    };

    let create_info = VkInstanceCreateInfo {
        sType: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        pNext: null(),
        flags: 0,
        pApplicationInfo: &application_info,
        enabledLayerCount: 0,
        ppEnabledLayerNames: null(),
        enabledExtensionCount: 0,
        ppEnabledExtensionNames: null(),
    };

    let mut instance = null_mut();
    let result = vkCreateInstance(&create_info, null(), &mut instance);
    if result != VK_SUCCESS {
        panic!("Failed to create the instance. Vulkan error {}.", result);
    }
    
    vkDestroyInstance(instance, null());
}
```

## License

The Vulkan Loader project is released as open source under a Apache-style license from Khronos including a Khronos copyright. As a result, this crate, which provides the bindings for the Vulkan Loader, is also released under the same license.

## Acknowledgements

While the Vulkan Loader project has been developed primarily by LunarG, Inc., there are many other companies and individuals making this possible: Valve Corporation, funding project development; Khronos providing oversight and hosting of the project. As for the crate providing the FFI bindings, it is currently developed by only one person: me.
