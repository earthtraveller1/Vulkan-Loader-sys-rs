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

const SWAP_CHAIN_EXTENSION: *const i8 = VK_KHR_SWAPCHAIN_EXTENSION_NAME.as_ptr() as *const i8;

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

    let validation_layers = [b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8];

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

            let mut extension_count = 0;
            vkEnumerateDeviceExtensionProperties(
                **device,
                null(),
                &mut extension_count,
                null_mut(),
            );
            let mut extensions = Vec::with_capacity(extension_count as usize);
            vkEnumerateDeviceExtensionProperties(
                **device,
                null(),
                &mut extension_count,
                extensions.as_mut_ptr(),
            );
            extensions.set_len(extension_count as usize);

            let swap_chain_extension_name = CStr::from_ptr(SWAP_CHAIN_EXTENSION).to_str().unwrap();

            let swap_chain_extension = extensions.iter().find(|extension| {
                CStr::from_ptr(extension.extensionName.as_ptr())
                    .to_str()
                    .unwrap()
                    == swap_chain_extension_name
            });

            let (_, swap_chain_formats, present_modes) =
                query_swap_chain_support(**device, surface);

            graphics_family.is_some()
                && present_family.is_some()
                && swap_chain_extension.is_some()
                && !swap_chain_formats.is_empty()
                && !present_modes.is_empty()
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

unsafe fn create_logical_device(
    physical_device: VkPhysicalDevice,
    graphics_family: u32,
    present_family: u32,
) -> VkDevice {
    let mut queue_create_infos = Vec::new();

    let queue_priority = 1.0f32;

    if graphics_family == present_family {
        queue_create_infos.push(VkDeviceQueueCreateInfo {
            sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            pNext: null(),
            flags: 0,
            queueFamilyIndex: graphics_family,
            queueCount: 1,
            pQueuePriorities: &queue_priority,
        });
    } else {
        queue_create_infos.push(VkDeviceQueueCreateInfo {
            sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            pNext: null(),
            flags: 0,
            queueFamilyIndex: graphics_family,
            queueCount: 1,
            pQueuePriorities: &queue_priority,
        });

        queue_create_infos.push(VkDeviceQueueCreateInfo {
            sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            pNext: null(),
            flags: 0,
            queueFamilyIndex: present_family,
            queueCount: 1,
            pQueuePriorities: &queue_priority,
        });
    }

    let device_extensions = [SWAP_CHAIN_EXTENSION];

    let create_info = VkDeviceCreateInfo {
        sType: VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
        pNext: null(),
        flags: 0,
        queueCreateInfoCount: queue_create_infos.len() as u32,
        pQueueCreateInfos: queue_create_infos.as_ptr(),
        enabledLayerCount: 0,
        ppEnabledLayerNames: null(),
        enabledExtensionCount: device_extensions.len() as u32,
        ppEnabledExtensionNames: device_extensions.as_ptr(),
        pEnabledFeatures: null(),
    };

    let mut device = null_mut();
    let result = vkCreateDevice(physical_device, &create_info, null(), &mut device);
    if result != VK_SUCCESS {
        panic!(
            "Failed to create the logical device. Vulkan error {}.",
            result
        );
    }

    device
}

unsafe fn query_swap_chain_support(
    device: VkPhysicalDevice,
    surface: VkSurfaceKHR,
) -> (
    VkSurfaceCapabilitiesKHR,
    Vec<VkSurfaceFormatKHR>,
    Vec<VkPresentModeKHR>,
) {
    let mut capabilities = MaybeUninit::uninit();
    vkGetPhysicalDeviceSurfaceCapabilitiesKHR(device, surface, capabilities.as_mut_ptr());

    let mut format_count = 0;
    vkGetPhysicalDeviceSurfaceFormatsKHR(device, surface, &mut format_count, null_mut());
    let mut formats = Vec::with_capacity(format_count as usize);
    vkGetPhysicalDeviceSurfaceFormatsKHR(device, surface, &mut format_count, formats.as_mut_ptr());
    formats.set_len(format_count as usize);

    let mut present_mode_count = 0;
    vkGetPhysicalDeviceSurfacePresentModesKHR(device, surface, &mut present_mode_count, null_mut());
    let mut present_modes = Vec::with_capacity(present_mode_count as usize);
    vkGetPhysicalDeviceSurfacePresentModesKHR(
        device,
        surface,
        &mut present_mode_count,
        present_modes.as_mut_ptr(),
    );
    present_modes.set_len(present_mode_count as usize);

    (capabilities.assume_init(), formats, present_modes)
}

unsafe fn create_swap_chain(
    physical_device: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    window: &glfw::Window,
    graphics_family: u32,
    present_family: u32,
    device: VkDevice,
) -> (VkSwapchainKHR, VkFormat, VkExtent2D, Vec<VkImage>) {
    // We start by querying the swap chain support details for the device.
    let (surface_capabilities, surface_formats, present_modes) =
        query_swap_chain_support(physical_device, surface);

    // Choose the settings for the swap chain.
    let surface_format = surface_formats
        .iter()
        .find(|format| {
            format.format == VK_FORMAT_B8G8R8A8_SRGB
                && format.colorSpace == VK_COLOR_SPACE_SRGB_NONLINEAR_KHR
        })
        .unwrap_or(&surface_formats[0])
        .clone();

    let present_mode = present_modes
        .iter()
        .find(|mode| **mode == VK_PRESENT_MODE_MAILBOX_KHR)
        .unwrap_or(&VK_PRESENT_MODE_FIFO_KHR)
        .clone();

    let swap_chain_extent = if surface_capabilities.currentExtent.width != u32::MAX {
        surface_capabilities.currentExtent
    } else {
        let (width, height) = window.get_framebuffer_size();
        VkExtent2D {
            width: (width as u32).clamp(
                surface_capabilities.minImageExtent.width,
                surface_capabilities.maxImageExtent.width,
            ),
            height: (height as u32).clamp(
                surface_capabilities.minImageExtent.height,
                surface_capabilities.maxImageExtent.height,
            ),
        }
    };

    let queue_family_indices = [graphics_family, present_family];

    let create_info = VkSwapchainCreateInfoKHR {
        sType: VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
        pNext: null(),
        flags: 0,
        surface: surface,
        minImageCount: if surface_capabilities.maxImageCount > 0
            && surface_capabilities.minImageCount == surface_capabilities.minImageCount
        {
            surface_capabilities.maxImageCount
        } else {
            surface_capabilities.minImageCount + 1
        },
        imageFormat: surface_format.format,
        imageColorSpace: surface_format.colorSpace,
        imageExtent: swap_chain_extent,
        imageArrayLayers: 1,
        imageUsage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
        imageSharingMode: if graphics_family == present_family {
            VK_SHARING_MODE_EXCLUSIVE
        } else {
            VK_SHARING_MODE_CONCURRENT
        },
        queueFamilyIndexCount: if graphics_family == present_family {
            0
        } else {
            2
        },
        pQueueFamilyIndices: if graphics_family == present_family {
            null()
        } else {
            queue_family_indices.as_ptr()
        },
        preTransform: surface_capabilities.currentTransform,
        compositeAlpha: VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
        presentMode: present_mode,
        clipped: VK_TRUE,
        oldSwapchain: null_mut(),
    };

    let mut swap_chain = null_mut();
    let result = vkCreateSwapchainKHR(device, &create_info, null(), &mut swap_chain);
    if result != VK_SUCCESS {
        panic!(
            "Failed to create the Vulkan swap chain. Vulkan error {}.",
            result
        );
    }

    let mut image_count = 0;
    vkGetSwapchainImagesKHR(device, swap_chain, &mut image_count, null_mut());
    let mut images = Vec::with_capacity(image_count as usize);
    vkGetSwapchainImagesKHR(device, swap_chain, &mut image_count, images.as_mut_ptr());
    images.set_len(image_count as usize);

    (swap_chain, surface_format.format, swap_chain_extent, images)
}

unsafe fn create_image_views(
    device: VkDevice,
    images: Vec<VkImage>,
    format: VkFormat,
) -> Vec<VkImageView> {
    images
        .iter()
        .map(|image| {
            let create_info = VkImageViewCreateInfo {
                sType: VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
                pNext: null(),
                flags: 0,
                image: *image,
                viewType: VK_IMAGE_VIEW_TYPE_2D,
                format: format,
                components: VkComponentMapping {
                    r: VK_COMPONENT_SWIZZLE_IDENTITY,
                    g: VK_COMPONENT_SWIZZLE_IDENTITY,
                    b: VK_COMPONENT_SWIZZLE_IDENTITY,
                    a: VK_COMPONENT_SWIZZLE_IDENTITY,
                },
                subresourceRange: VkImageSubresourceRange {
                    aspectMask: VK_IMAGE_ASPECT_COLOR_BIT,
                    baseMipLevel: 0,
                    levelCount: 1,
                    baseArrayLayer: 0,
                    layerCount: 1,
                },
            };

            let mut image_view = null_mut();
            let result = vkCreateImageView(device, &create_info, null(), &mut image_view);
            if result != VK_SUCCESS {
                panic!("Failed to create an image view!");
            }

            image_view
        })
        .collect()
}

fn main() {
    unsafe {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("Failed to initialize GLFW.");
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        glfw.window_hint(glfw::WindowHint::Resizable(false)); // Handling resizing is a bit complicated in Vulkan, so I'll disable it for now.

        let instance = create_instance(&glfw);
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
            choose_physical_device(instance, surface);

        let device =
            create_logical_device(physical_device, graphics_queue_family, present_queue_family);

        let (swap_chain, swap_chain_format, swap_chain_extent, swap_chain_images) =
            create_swap_chain(
                physical_device,
                surface,
                &window,
                graphics_queue_family,
                present_queue_family,
                device,
            );

        let swap_chain_image_views =
            create_image_views(device, swap_chain_images, swap_chain_format);

        while !window.should_close() {
            glfw.poll_events();
        }

        swap_chain_image_views
            .iter()
            .for_each(|image_view| vkDestroyImageView(device, *image_view, null()));
        vkDestroySwapchainKHR(device, swap_chain, null());
        vkDestroyDevice(device, null());
        vkDestroySurfaceKHR(instance, surface, null());
        vkDestroyInstance(instance, null());
    }
}
