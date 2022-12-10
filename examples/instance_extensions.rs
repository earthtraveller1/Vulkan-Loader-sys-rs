// A simple Vulkan application that does nothing but retrieves the available i-
// nstance extensions and exits.

use std::{ptr::{null, null_mut},ffi::CStr};
use vulkan_loader_sys::*;

fn main() {
    unsafe {
        let mut extension_count = 0;
        vkEnumerateInstanceExtensionProperties(null(), &mut extension_count, null_mut());

        let mut extensions = Vec::with_capacity(extension_count.try_into().unwrap());
        vkEnumerateInstanceExtensionProperties(null(), &mut extension_count, extensions.as_mut_ptr());
        extensions.set_len(extension_count.try_into().unwrap());
        
        extensions.iter().for_each(|extension| {
            println!(
                "[INFO]: Found instance extension {}.",
                CStr::from_ptr(&extension.extensionName[0]).to_str().unwrap()
            );
        });
    }
}
