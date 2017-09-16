extern crate vulkano;
use vulkano::descriptor::descriptor_set;

extern crate vulkano_win;

use std::sync::Arc;
use std::boxed::Box;
use std::marker::{Sync, Send};
use std::mem;

use gl_types::Vec2;

pub struct GraphicsPart {
    pub pipeline: Arc<vulkano::pipeline::GraphicsPipeline<
        vulkano::pipeline::vertex::SingleBufferDefinition<Vec2>,
        Box<vulkano::descriptor::PipelineLayoutAbstract + Sync + Send>,
        Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>>>,
    pub dimensions: [u32; 2],
    pub swapchain: Arc<vulkano::swapchain::Swapchain>,
    pub images: Vec<Arc<vulkano::image::swapchain::SwapchainImage>>,
    pub set: Arc<descriptor_set::DescriptorSet + Send +  Sync>,
    pub renderpass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
    pub vertex_buffer: Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<[Vec2]>>,
}

impl GraphicsPart {
    pub fn new<I: 'static + vulkano::image::ImageViewAccess + Send + Sync>(device: &Arc<vulkano::device::Device>,
           window: &vulkano_win::Window,
           physical: vulkano::instance::PhysicalDevice,
           queue: Arc<vulkano::device::Queue>,
           texture: Arc<I>)
           -> GraphicsPart {

        let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

        let sampler =
            vulkano::sampler::Sampler::new(device.clone(),
                                           vulkano::sampler::Filter::Nearest,
                                           vulkano::sampler::Filter::Nearest,
                                           vulkano::sampler::MipmapMode::Nearest,
                                           vulkano::sampler::SamplerAddressMode::ClampToEdge,
                                           vulkano::sampler::SamplerAddressMode::ClampToEdge,
                                           vulkano::sampler::SamplerAddressMode::ClampToEdge,
                                           0.0,
                                           1.0,
                                           0.0,
                                           0.0)
                .unwrap();

        let dimensions = {
            let (width, height) = window.window().get_inner_size_pixels().unwrap();
            [width, height]
        };

        let (swapchain, images) = {
            let caps = window.surface()
                .capabilities(physical)
                .expect("failed to get surface capabilities");

            let usage = caps.supported_usage_flags;
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;

            vulkano::swapchain::Swapchain::new(device.clone(),
                                               window.surface().clone(),
                                               caps.min_image_count,
                                               format,
                                               dimensions,
                                               1,
                                               usage,
                                               &queue,
                                               vulkano::swapchain::SurfaceTransform::Identity,
                                               alpha,
                                               vulkano::swapchain::PresentMode::Fifo,
                                               true,
                                               None)
                .expect("failed to create swapchain")
        };

        let renderpass = Arc::new(single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: DontCare,
                    store: DontCare,
                    format: swapchain.format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
            .unwrap());

        let pipeline = Arc::new(vulkano::pipeline::GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vec2>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_strip()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .blend_alpha_blending()
            .render_pass(vulkano::framebuffer::Subpass::from(renderpass.clone()
                                                             as Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>, 0).unwrap())
            .build(device.clone())
            .unwrap());

        let set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
            .add_sampled_image(texture, sampler.clone())
            .unwrap()
            .build()
            .unwrap());

        let vertex_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
                               ::from_iter(device.clone(), vulkano::buffer::BufferUsage::all(),
                                       [
                                           Vec2 { position: [-1.0, -1.0 ] },
                                           Vec2 { position: [-1.0,  1.0 ] },
                                           Vec2 { position: [ 1.0, -1.0 ] },
                                           Vec2 { position: [ 1.0,  1.0 ] },
                                       ].iter().cloned()).expect("failed to create buffer");

        GraphicsPart {
            pipeline: pipeline,
            dimensions: dimensions,
            swapchain: swapchain,
            images: images,
            set: set,
            renderpass: renderpass,
            vertex_buffer: vertex_buffer,
        }
    }

    pub fn recreate_swapchain(&mut self, window: &vulkano_win::Window) -> bool {
        self.dimensions = {
            let (new_width, new_height) = window.window().get_inner_size_pixels().unwrap();
            [new_width, new_height]
        };

        let (new_swapchain, new_images) = match self.swapchain
            .recreate_with_dimension(self.dimensions) {
            Ok(r) => r,
            Err(vulkano::swapchain::SwapchainCreationError::UnsupportedDimensions) => return false,
            Err(err) => panic!("{:?}", err),
        };

        mem::replace(&mut self.swapchain, new_swapchain);
        mem::replace(&mut self.images, new_images);

        true
    }
}

mod vs {
    #[allow(dead_code)]
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[path = "shaders/quad.vert"]
    struct Dummy;
}

mod fs {
    #[allow(dead_code)]
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[path = "shaders/quad.frag"]
    struct Dummy;
}
