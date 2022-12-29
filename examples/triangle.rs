use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    ptr::{null, null_mut},
};
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

fn create_surface(instance: VkInstance, window: &glfw::Window) -> VkSurfaceKHR {
    let mut surface = null_mut::<VkSurfaceKHR_T>();
    window.create_window_surface(
        instance as usize,
        null(),
        &mut surface as *mut VkSurfaceKHR as *mut u64,
    );
    surface
}

unsafe fn choose_physical_device(
    instance: VkInstance,
    surface: VkSurfaceKHR,
) -> (VkPhysicalDevice, u32, u32) {
    let mut physical_device_count = 0;
    vkEnumeratePhysicalDevices(instance, &mut physical_device_count, null_mut());

    let mut physical_devices = Vec::with_capacity(physical_device_count.try_into().unwrap());
    vkEnumeratePhysicalDevices(
        instance,
        &mut physical_device_count,
        physical_devices.as_mut_ptr(),
    );
    physical_devices.set_len(physical_device_count.try_into().unwrap());

    let find_queue_families = |device: VkPhysicalDevice| -> (Option<u32>, Option<u32>) {
        let mut queue_family_count = 0;
        vkGetPhysicalDeviceQueueFamilyProperties(device, &mut queue_family_count, null_mut());
        let mut queue_family_properties =
            Vec::with_capacity(queue_family_count.try_into().unwrap());
        vkGetPhysicalDeviceQueueFamilyProperties(
            device,
            &mut queue_family_count,
            queue_family_properties.as_mut_ptr(),
        );
        queue_family_properties.set_len(queue_family_count.try_into().unwrap());

        let mut graphics_family = None;
        let mut present_family = None;

        for i in 0..queue_family_properties.len() {
            if (queue_family_properties[i].queueFlags & VK_QUEUE_GRAPHICS_BIT as u32) != 0 {
                graphics_family = Some(i.try_into().unwrap());
            }

            let mut present_support = VK_FALSE;
            vkGetPhysicalDeviceSurfaceSupportKHR(
                device,
                i.try_into().unwrap(),
                surface,
                &mut present_support,
            );

            if present_support == VK_TRUE {
                present_family = Some(i.try_into().unwrap());
            }
        }

        (graphics_family, present_family)
    };

    let physical_device = *physical_devices
        .iter()
        .find(|device| {
            let (graphics_family, present_family) = find_queue_families(**device);

            graphics_family.is_some() && present_family.is_some()
        })
        .expect("Could not find an adequate physical device.");

    let (graphics_family, present_family) = find_queue_families(physical_device);
    let graphics_family = graphics_family.unwrap();
    let present_family = present_family.unwrap();

    let mut device_properties = MaybeUninit::uninit();
    vkGetPhysicalDeviceProperties(physical_device, device_properties.as_mut_ptr());
    let device_properties = device_properties.assume_init();

    println!(
        "[INFO]: Using the {} graphics card.",
        CStr::from_ptr(&device_properties.deviceName[0])
            .to_str()
            .unwrap()
    );

    (physical_device, graphics_family, present_family)
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("Failed to initialize GLFW.");
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    glfw.window_hint(glfw::WindowHint::Resizable(false)); // Handling resizing is a bit complicated in Vulkan, so I'll disable it for now.

    let instance = unsafe { create_instance(&glfw) };
    let (window, _) = glfw
        .create_window(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            "Triangle Example",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create the GLFW window.");

    let surface = create_surface(instance, &window);

    let (physical_device, graphics_queue_family, present_queue_family) =
        unsafe { choose_physical_device(instance, surface) };

    while !window.should_close() {
        glfw.poll_events();
    }

    unsafe {
        vkDestroySurfaceKHR(instance, surface, null());
        vkDestroyInstance(instance, null());
    }
}
