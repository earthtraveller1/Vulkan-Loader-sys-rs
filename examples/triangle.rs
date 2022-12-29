use std::ffi::CString;
use std::ptr::{null, null_mut};
/// Designed to be a basic Vulkan application.
/// Would probably display a triangle some time in the future, but for now it
/// is just Vulkan boilerplate code.
use vulkan_loader_sys::*;

const ENABLE_VALIDATION: bool = true;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

unsafe fn create_instance(glfw: &glfw::Glfw) -> VkInstance {
    let application_info = VkApplicationInfo {
        sType: VK_STRUCTURE_TYPE_APPLICATION_INFO,
        pNext: null(),
        pApplicationName: b"Triangle Example\0".as_ptr() as *const i8,
        applicationVersion: 1,
        pEngineName: null(), // We aren't using an engine in this case.
        engineVersion: 0,
        apiVersion: VK_MAKE_API_VERSION(0, 1, 2, 0),
    };

    let required_extensions = glfw
        .get_required_instance_extensions()
        .expect("Cannot obtain the list of required instance extensions.")
        .iter()
        .map(|s| CString::new(s.as_str()).unwrap())
        .collect::<Vec<CString>>();
    let required_extensions = required_extensions
        .iter()
        .map(|s| s.as_ptr())
        .collect::<Vec<*const i8>>();

    let validation_layers = [b"VK_LAYER_KHRONOS_validation".as_ptr() as *const i8];

    let create_info = VkInstanceCreateInfo {
        sType: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        pNext: null(),
        flags: 0,
        pApplicationInfo: &application_info,
        enabledLayerCount: if ENABLE_VALIDATION {
            validation_layers.len().try_into().unwrap()
        } else {
            0
        },
        ppEnabledLayerNames: if ENABLE_VALIDATION {
            validation_layers.as_ptr()
        } else {
            null()
        },
        enabledExtensionCount: required_extensions.len().try_into().unwrap(),
        ppEnabledExtensionNames: required_extensions.as_ptr(),
    };

    let mut instance = null_mut();
    let result = vkCreateInstance(&create_info, null(), &mut instance);
    if result != VK_SUCCESS {
        panic!("Failed to create the instance. Vulkan error {}.", result);
    }

    instance
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("Failed to initialize GLFW.");
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    glfw.window_hint(glfw::WindowHint::Resizable(false)); // Handling resizing is a bit complicated in Vulkan, so I'll disable it for now.
    
    let instance = unsafe { create_instance(&glfw) };
    let (window, _) = glfw.create_window(WINDOW_WIDTH, WINDOW_HEIGHT, "Triangle Example", glfw::WindowMode::Windowed).expect("Failed to create the GLFW window.");
    
    while !window.should_close() {
        glfw.poll_events();
    }
    
    unsafe {
        vkDestroyInstance(instance, null());
    }
}
