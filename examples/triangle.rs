use std::{
    ffi::{CStr, CString},
    fs::File,
    io::Read,
    mem::{size_of, MaybeUninit},
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

#[repr(C)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
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

fn main() {
    unsafe {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("Failed to initialize GLFW.");
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        glfw.window_hint(glfw::WindowHint::Resizable(false)); // Handling resizing is a bit complicated in Vulkan, so I'll disable it for now.

        let instance = {
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
        };

        let (window, _) = glfw
            .create_window(
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                "Triangle Example",
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create the GLFW window.");

        let surface = {
            let mut surface = null_mut::<VkSurfaceKHR_T>();
            window.create_window_surface(
                instance as usize,
                null(),
                &mut surface as *mut VkSurfaceKHR as *mut u64,
            );
            surface
        };

        let (physical_device, graphics_queue_family, present_queue_family) = {
            let mut physical_device_count = 0;
            vkEnumeratePhysicalDevices(instance, &mut physical_device_count, null_mut());

            let mut physical_devices =
                Vec::with_capacity(physical_device_count.try_into().unwrap());
            vkEnumeratePhysicalDevices(
                instance,
                &mut physical_device_count,
                physical_devices.as_mut_ptr(),
            );
            physical_devices.set_len(physical_device_count.try_into().unwrap());

            let find_queue_families = |device: VkPhysicalDevice| -> (Option<u32>, Option<u32>) {
                let mut queue_family_count = 0;
                vkGetPhysicalDeviceQueueFamilyProperties(
                    device,
                    &mut queue_family_count,
                    null_mut(),
                );
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

                    let swap_chain_extension_name =
                        CStr::from_ptr(SWAP_CHAIN_EXTENSION).to_str().unwrap();

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
        };

        let device = {
            let graphics_family = &graphics_queue_family;
            let present_family = &present_queue_family;

            let mut queue_create_infos = Vec::new();

            let queue_priority = 1.0f32;

            if graphics_family == present_family {
                queue_create_infos.push(VkDeviceQueueCreateInfo {
                    sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    queueFamilyIndex: *graphics_family,
                    queueCount: 1,
                    pQueuePriorities: &queue_priority,
                });
            } else {
                queue_create_infos.push(VkDeviceQueueCreateInfo {
                    sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    queueFamilyIndex: *graphics_family,
                    queueCount: 1,
                    pQueuePriorities: &queue_priority,
                });

                queue_create_infos.push(VkDeviceQueueCreateInfo {
                    sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    queueFamilyIndex: *present_family,
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
        };

        let (swap_chain, swap_chain_format, swap_chain_extent, swap_chain_images) = {
            let graphics_family = &graphics_queue_family;
            let present_family = &present_queue_family;

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
                imageUsage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT as u32,
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
                    *queue_family_indices.as_ptr()
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
        };

        let swap_chain_image_views = swap_chain_images
            .iter()
            .map(|image| {
                let create_info = VkImageViewCreateInfo {
                    sType: VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    image: *image,
                    viewType: VK_IMAGE_VIEW_TYPE_2D,
                    format: swap_chain_format,
                    components: VkComponentMapping {
                        r: VK_COMPONENT_SWIZZLE_IDENTITY,
                        g: VK_COMPONENT_SWIZZLE_IDENTITY,
                        b: VK_COMPONENT_SWIZZLE_IDENTITY,
                        a: VK_COMPONENT_SWIZZLE_IDENTITY,
                    },
                    subresourceRange: VkImageSubresourceRange {
                        aspectMask: VK_IMAGE_ASPECT_COLOR_BIT as u32,
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
            .collect::<Vec<VkImageView>>();

        let render_pass = {
            let color_attachment = VkAttachmentDescription {
                flags: 0,
                format: swap_chain_format,
                samples: VK_SAMPLE_COUNT_1_BIT,
                loadOp: VK_ATTACHMENT_LOAD_OP_CLEAR,
                storeOp: VK_ATTACHMENT_STORE_OP_STORE,
                stencilLoadOp: VK_ATTACHMENT_LOAD_OP_DONT_CARE,
                stencilStoreOp: VK_ATTACHMENT_STORE_OP_DONT_CARE,
                initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
                finalLayout: VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
            };

            let color_attachment_ref = VkAttachmentReference {
                attachment: 0,
                layout: VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
            };

            let subpass = VkSubpassDescription {
                flags: 0,
                pipelineBindPoint: VK_PIPELINE_BIND_POINT_GRAPHICS,
                inputAttachmentCount: 0,
                pInputAttachments: null(),
                colorAttachmentCount: 1,
                pColorAttachments: &color_attachment_ref,
                pResolveAttachments: null(),
                pDepthStencilAttachment: null(),
                preserveAttachmentCount: 0,
                pPreserveAttachments: null(),
            };

            let create_info = VkRenderPassCreateInfo {
                sType: VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
                pNext: null(),
                flags: 0,
                attachmentCount: 1,
                pAttachments: &color_attachment,
                subpassCount: 1,
                pSubpasses: &subpass,
                dependencyCount: 0,
                pDependencies: null(),
            };

            let mut render_pass = null_mut();
            let result = vkCreateRenderPass(device, &create_info, null(), &mut render_pass);
            if result != VK_SUCCESS {
                panic!(
                    "Failed to create the Vulkan render pass. Vulkan error {}",
                    result
                );
            }

            render_pass
        };

        let pipeline_layout = {
            let create_info = VkPipelineLayoutCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
                pNext: null(),
                flags: 0,
                setLayoutCount: 0,
                pSetLayouts: null(),
                pushConstantRangeCount: 0,
                pPushConstantRanges: null(),
            };

            let mut pipeline_layout = null_mut();
            let result = vkCreatePipelineLayout(device, &create_info, null(), &mut pipeline_layout);
            if result != VK_SUCCESS {
                panic!("Failed to create the pipeline layout.");
            }

            pipeline_layout
        };

        let graphics_pipeline = {
            let (vertex_shader_module, fragment_shader_module) = {
                let mut vertex_file = File::open("examples/triangle/shaders/vertex.vert.spv").expect("Failed to open the vertex shader file. Make sure you are running from the project's root directory.");
                let mut fragment_file = File::open("examples/triangle/shaders/fragment.frag.spv").expect("Failed to open the fragment shader file. Make sure you are running from the project's root directory.");

                let mut vertex_code = Vec::new();
                vertex_file
                    .read_to_end(&mut vertex_code)
                    .expect("Failed to read from the vertex shader file.");

                let mut fragment_code = Vec::new();
                fragment_file
                    .read_to_end(&mut fragment_code)
                    .expect("Failed to read from the fragment shader file.");

                let vertex_module_create_info = VkShaderModuleCreateInfo {
                    sType: VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    codeSize: fragment_code.len(),
                    pCode: fragment_code.as_ptr() as *const u32,
                };

                let fragment_module_create_info = VkShaderModuleCreateInfo {
                    sType: VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    codeSize: fragment_code.len(),
                    pCode: fragment_code.as_ptr() as *const u32,
                };

                let mut vertex_module = null_mut();
                let vertex_result = vkCreateShaderModule(
                    device,
                    &vertex_module_create_info,
                    null(),
                    &mut vertex_module,
                );
                if vertex_result != VK_SUCCESS {
                    panic!(
                        "Failed to create the vertex shader module. Vulkan error {}.",
                        vertex_result
                    );
                }

                let mut fragment_module = null_mut();
                let fragment_result = vkCreateShaderModule(
                    device,
                    &fragment_module_create_info,
                    null(),
                    &mut fragment_module,
                );
                if fragment_result != VK_SUCCESS {
                    panic!(
                        "Failed to create the fragment shader module. Vulkan error {}.",
                        fragment_result
                    );
                }

                (vertex_module, fragment_module)
            };

            let vertex_shader_stage = VkPipelineShaderStageCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                stage: VK_SHADER_STAGE_VERTEX_BIT,
                module: vertex_shader_module,
                pName: b"main".as_ptr() as *const i8,
                pSpecializationInfo: null(),
            };

            let fragment_shader_stage = VkPipelineShaderStageCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                stage: VK_SHADER_STAGE_FRAGMENT_BIT,
                module: fragment_shader_module,
                pName: b"main".as_ptr() as *const i8,
                pSpecializationInfo: null(),
            };

            let shader_stages = [vertex_shader_stage, fragment_shader_stage];

            let binding_description = VkVertexInputBindingDescription {
                binding: 0,
                stride: size_of::<Vertex>() as u32,
                inputRate: VK_VERTEX_INPUT_RATE_VERTEX,
            };

            // We will only have one input attribute, and that is the vertex position.
            let attribute_description = VkVertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: VK_FORMAT_R32G32_SFLOAT,
                offset: 0,
            };

            let vertex_input = VkPipelineVertexInputStateCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                vertexBindingDescriptionCount: 1,
                pVertexBindingDescriptions: &binding_description,
                vertexAttributeDescriptionCount: 1,
                pVertexAttributeDescriptions: &attribute_description,
            };

            let input_assembly = VkPipelineInputAssemblyStateCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                topology: VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
                primitiveRestartEnable: VK_FALSE,
            };

            let viewport = VkViewport {
                x: 0.0,
                y: 0.0,
                width: swap_chain_extent.width as f32,
                height: swap_chain_extent.height as f32,
                minDepth: 0.0,
                maxDepth: 0.0,
            };

            let scissor = VkRect2D {
                offset: VkOffset2D { x: 0, y: 0 },
                extent: swap_chain_extent,
            };

            let viewport_state = VkPipelineViewportStateCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                viewportCount: 1,
                pViewports: &viewport,
                scissorCount: 1,
                pScissors: &scissor,
            };

            let rasterization = VkPipelineRasterizationStateCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                depthClampEnable: VK_FALSE,
                rasterizerDiscardEnable: VK_FALSE,
                polygonMode: VK_POLYGON_MODE_FILL,
                cullMode: VK_CULL_MODE_BACK_BIT as u32,
                frontFace: VK_FRONT_FACE_CLOCKWISE,
                depthBiasEnable: VK_FALSE,
                depthBiasConstantFactor: 0.0,
                depthBiasClamp: 0.0,
                depthBiasSlopeFactor: 0.0,
                lineWidth: 1.0,
            };

            let multisampling = VkPipelineMultisampleStateCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                rasterizationSamples: VK_SAMPLE_COUNT_1_BIT,
                sampleShadingEnable: VK_FALSE,
                minSampleShading: 0.0,
                pSampleMask: null(),
                alphaToCoverageEnable: VK_FALSE,
                alphaToOneEnable: VK_FALSE,
            };

            let color_blend_attachment = VkPipelineColorBlendAttachmentState {
                colorWriteMask: (VK_COLOR_COMPONENT_R_BIT
                    | VK_COLOR_COMPONENT_G_BIT
                    | VK_COLOR_COMPONENT_G_BIT
                    | VK_COLOR_COMPONENT_A_BIT) as u32,
                blendEnable: VK_FALSE,
                srcColorBlendFactor: VK_BLEND_FACTOR_ONE,
                dstColorBlendFactor: VK_BLEND_FACTOR_ZERO,
                colorBlendOp: VK_BLEND_OP_ADD,
                srcAlphaBlendFactor: VK_BLEND_FACTOR_ONE,
                dstAlphaBlendFactor: VK_BLEND_FACTOR_ZERO,
                alphaBlendOp: VK_BLEND_OP_ADD,
            };

            let color_blending = VkPipelineColorBlendStateCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                logicOpEnable: VK_FALSE,
                logicOp: VK_LOGIC_OP_COPY,
                attachmentCount: 1,
                pAttachments: &color_blend_attachment,
                blendConstants: [0.0; 4],
            };

            let create_info = VkGraphicsPipelineCreateInfo {
                sType: VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                stageCount: shader_stages.len() as u32,
                pStages: shader_stages.as_ptr(),
                pVertexInputState: &vertex_input,
                pInputAssemblyState: &input_assembly,
                pTessellationState: null(),
                pViewportState: &viewport_state,
                pRasterizationState: &rasterization,
                pMultisampleState: &multisampling,
                pDepthStencilState: null(),
                pColorBlendState: &color_blending,
                pDynamicState: null(),
                layout: pipeline_layout,
                renderPass: render_pass,
                subpass: 0,
                basePipelineHandle: null_mut(),
                basePipelineIndex: 0,
            };

            let mut pipeline = null_mut();
            let result = vkCreateGraphicsPipelines(
                device,
                null_mut(),
                1,
                &create_info,
                null(),
                &mut pipeline,
            );
            if result != VK_SUCCESS {
                panic!(
                    "Failed to create the graphics pipeline. Vulkan error {}.",
                    result
                );
            }

            vkDestroyShaderModule(device, vertex_shader_module, null());
            vkDestroyShaderModule(device, fragment_shader_module, null());

            pipeline
        };

        while !window.should_close() {
            glfw.poll_events();
        }

        vkDestroyPipeline(device, graphics_pipeline, null());
        vkDestroyRenderPass(device, render_pass, null());
        vkDestroyPipelineLayout(device, pipeline_layout, null());
        swap_chain_image_views
            .iter()
            .for_each(|image_view| vkDestroyImageView(device, *image_view, null()));
        vkDestroySwapchainKHR(device, swap_chain, null());
        vkDestroyDevice(device, null());
        vkDestroySurfaceKHR(instance, surface, null());
        vkDestroyInstance(instance, null());
    }
}
