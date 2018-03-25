extern crate vulkano;
extern crate vulkano_win;
extern crate winit;

use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;

use args::Args;
use camera;
use cs;
use drawer::Drawer;
use fps_counter::FPSCounter;
use vulkan_ctx::VulkanCtx;

use std::path::Path;
use std::sync::Arc;

pub struct RealTimeRender<'a> {
    pub vulkan_ctx: VulkanCtx<'a>,
    window: vulkano_win::Window,
    drawer: Drawer<'a>,
}

impl<'a> RealTimeRender<'a> {
    pub fn new(
        args: &Args,
        events_loop: &winit::EventsLoop,
        instance: &'a Arc<vulkano::instance::Instance>,
    ) -> RealTimeRender<'a> {
        let window = winit::WindowBuilder::new()
            .with_min_dimensions(args.resolution[0], args.resolution[1])
            .with_max_dimensions(args.resolution[0], args.resolution[1])
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();
        window.window().set_cursor(winit::MouseCursor::NoneCursor);

        let (vulkan_ctx, _) = VulkanCtx::new(&instance, Path::new(&args.model), |&q| {
            q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false)
        });

        let (drawer, _) = Drawer::new(
            vulkan_ctx.device.clone(),
            &window,
            vulkan_ctx.physical.clone(),
            vulkan_ctx.queue.clone(),
        );

        RealTimeRender {
            vulkan_ctx,
            window,
            drawer,
        }
    }

    pub fn render(
        &mut self,
        camera: &mut camera::Camera,
        fps_counter: &FPSCounter,
        recreate_swapchain: bool,
        mut previous_frame_end: Box<vulkano::sync::GpuFuture>,
    ) -> Box<vulkano::sync::GpuFuture> {
        previous_frame_end.cleanup_finished();

        if self.drawer.recreate_swapchain(&self.window) {
            return previous_frame_end;
        }

        self.drawer.recreate_framebuffers();

        let (image_num, aquire_future) = match self.drawer.acquire_next_image() {
            Ok(r) => r,
            Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                return previous_frame_end;
            }
            Err(err) => panic!("{:?}", err),
        };

        let (grid, grid_future) = self.vulkan_ctx
            .grid_builder
            .build(Box::new(vulkano::sync::now(self.vulkan_ctx.device.clone())));

        // FIXME: it is not used here, but is required for tracer.render()
        let statistics_buffer =
            vulkano::buffer::CpuAccessibleBuffer::<cs::ty::Statistics>::from_data(
                self.vulkan_ctx.device.clone(),
                vulkano::buffer::BufferUsage::all(),
                cs::ty::Statistics {
                    triangle_intersections: 0,
                    triangle_tests: 0,
                    cell_intersections: 0,
                },
            ).unwrap();

        let cb = {
            let mut cbb =
                vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(
                    self.vulkan_ctx.device.clone(),
                    self.vulkan_ctx.queue.family(),
                ).unwrap();
            cbb = self.vulkan_ctx.tracer.render(
                cbb,
                self.drawer.texture.clone(),
                statistics_buffer.clone(),
                &camera,
                &grid,
            );
            cbb = self.drawer.draw(cbb, image_num);
            cbb.build().unwrap()
        };

        self.drawer.recreate_swapchain = recreate_swapchain;

        let future = previous_frame_end
            .join(aquire_future)
            .join(grid_future)
            .then_execute(self.vulkan_ctx.queue.clone(), cb)
            .unwrap()
            .then_swapchain_present(
                self.vulkan_ctx.queue.clone(),
                self.drawer.swapchain.clone(),
                image_num,
            )
            .then_signal_fence_and_flush()
            .unwrap();

        self.drawer.queue_text(
            10.0,
            20.0,
            20.0,
            &format!(
                "Using device: {}\nRender time: {} ms ({} FPS)\nCamera: {}",
                self.vulkan_ctx.physical.name(),
                fps_counter.average_render_time(),
                fps_counter.current_fps(),
                camera
            ),
        );

        Box::new(future)
    }
}
