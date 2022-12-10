/// Designed to be a basic Vulkan application.
/// Would probably display a triangle some time in the future, but for now it 
/// is just Vulkan boilerplate code.

use vulkan_loader_sys::*;
use std::{ptr::{null, null_mut}};

fn main() {
    unsafe {
        let application_info = VkApplicationInfo {
            sType: VK_STRUCTURE_TYPE_APPLICATION_INFO,
            pNext: null(),
            pApplicationName: b"Basic Vulkan Example\0".as_ptr() as *const i8,
            applicationVersion: 1,
            pEngineName: null(),
            engineVersion: 0,
            apiVersion: VK_MAKE_API_VERSION(0, 1, 2, 0)
        };
        
        let instance_create_info = VkInstanceCreateInfo {
            sType: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            pNext: null(),
            flags: 0,
            pApplicationInfo: &application_info,
            enabledLayerCount: 0,
            ppEnabledLayerNames: null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: null()
        };
        
        let mut instance = null_mut();
        let result = vkCreateInstance(&instance_create_info, null(), &mut instance);
        if result != VK_SUCCESS {
            panic!("Failed to create the instance.");
        }
        
        vkDestroyInstance(instance, null());
    }
}