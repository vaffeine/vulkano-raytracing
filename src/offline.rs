extern crate time;
extern crate vulkano;

use vulkano::sync::GpuFuture;

use args::Args;
use camera::Camera;
use cs;
use vulkan_ctx::VulkanCtx;
use grid::Grid;

use std::mem;
use std::path::Path;
use std::sync::Arc;
use std::fmt;

pub struct OfflineRender<'a> {
    vulkan_ctx: VulkanCtx<'a>,
    statistics_buffer: Arc<vulkano::buffer::CpuAccessibleBuffer<cs::ty::Statistics>>,
    texture: Arc<vulkano::image::StorageImage<vulkano::format::R8G8B8A8Unorm>>,
    dimensions: [u32; 2],
}

impl<'a> OfflineRender<'a> {
    pub fn new(
        args: &Args,
        instance: &'a Arc<vulkano::instance::Instance>,
        dimensions: [u32; 2],
    ) -> OfflineRender<'a> {
        let (vulkan_ctx, _) =
            VulkanCtx::new(&instance, Path::new(&args.model), |&q| q.supports_compute());
        let statistics_buffer =
            vulkano::buffer::CpuAccessibleBuffer::<cs::ty::Statistics>::from_data(
                vulkan_ctx.device.clone(),
                vulkano::buffer::BufferUsage::all(),
                cs::ty::Statistics {
                    triangle_intersections: 0,
                    triangle_tests: 0,
                    cell_intersections: 0,
                },
            ).unwrap();

        let texture = vulkano::image::StorageImage::new(
            vulkan_ctx.device.clone(),
            vulkano::image::Dimensions::Dim2d {
                width: dimensions[0],
                height: dimensions[1],
            },
            vulkano::format::R8G8B8A8Unorm,
            Some(vulkan_ctx.queue.family()),
        ).unwrap();

        OfflineRender {
            vulkan_ctx,
            statistics_buffer,
            texture,
            dimensions,
        }
    }

    pub fn render(&mut self, camera: &Camera) -> Statistics {
        let grid_start = time::PreciseTime::now();
        let (grid, future) = self.vulkan_ctx
            .grid_builder
            .build(Box::new(vulkano::sync::now(self.vulkan_ctx.device.clone())));
        mem::drop(future);
        let grid_build_time = grid_start.to(time::PreciseTime::now()).num_milliseconds();

        let cb = {
            let mut cbb =
                vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(
                    self.vulkan_ctx.device.clone(),
                    self.vulkan_ctx.queue.family(),
                ).unwrap();

            cbb = self.vulkan_ctx.tracer.render(
                cbb,
                self.texture.clone(),
                self.statistics_buffer.clone(),
                &camera,
                &grid,
            );

            cbb.build().unwrap()
        };

        let render_start = time::PreciseTime::now();
        let future = vulkano::sync::now(self.vulkan_ctx.device.clone())
            .then_execute(self.vulkan_ctx.queue.clone(), cb)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();
        let render_time = render_start.to(time::PreciseTime::now()).num_milliseconds();

        let render_statistics = *self.statistics_buffer
            .read()
            .expect("failed to lock buffer for reading");

        let primary_rays = self.dimensions[0] * self.dimensions[1];
        Statistics {
            grid_build_time,
            render_time,
            triangle_count: self.vulkan_ctx.scene_buffers.triangle_count,
            primary_rays,
            render_statistics,
            grid,
        }
    }
}

pub struct Statistics {
    grid_build_time: i64,
    render_time: i64,
    triangle_count: usize,
    primary_rays: u32,
    render_statistics: cs::ty::Statistics,
    grid: Grid,
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let render_time = self.grid_build_time + self.render_time;
        writeln!(f, "\n>>> General")?;
        writeln!(
            f,
            "\ttotal time: {} ms ({} FPS)",
            render_time,
            1000 / render_time
        )?;
        writeln!(
            f,
            "\ttriangles: {}",
            self.triangle_count
        )?;
        writeln!(f, "\tprimary rays: {}", self.primary_rays)?;
        writeln!(f, "\n>>> Triangle")?;
        writeln!(f, "\ttests: {}", self.render_statistics.triangle_tests)?;
        writeln!(f, "\tintersections: {}", self.render_statistics.triangle_intersections)?;
        writeln!(
            f,
            "\ttests per ray: {}",
            self.render_statistics.triangle_tests as f32 / self.primary_rays as f32
        )?;
        writeln!(
            f,
            "\ttests per triangle: {}",
            self.render_statistics.triangle_tests as f32 / self.triangle_count as f32
        )?;
        writeln!(f, "\n>>> Grid")?;
        writeln!(f, "\tbuild time: {} ms", self.grid_build_time)?;
        let grid_size = [
            self.grid.bbox.max.position[0] - self.grid.bbox.min.position[0],
            self.grid.bbox.max.position[1] - self.grid.bbox.min.position[1],
            self.grid.bbox.max.position[2] - self.grid.bbox.min.position[2],
        ];
        writeln!(f, "\tsize: {:?}", grid_size)?;
        writeln!(f, "\tresolution: {:?}", self.grid.resolution)?;
        let cell_count = self.grid.resolution[0] * self.grid.resolution[1] * self.grid.resolution[2];
        writeln!(f, "\tcell count: {}", cell_count)?;
        writeln!(f, "\tcell size: {:?}", self.grid.cell_size)?;
        writeln!(f, "\tcell intersections: {}", self.render_statistics.cell_intersections)?;
        writeln!(
            f,
            "\tintersections per ray: {}",
            self.render_statistics.cell_intersections as f32 / self.primary_rays as f32
        )?;
        writeln!(
            f,
            "\tintersections per cell: {}",
            self.render_statistics.cell_intersections as f32 / cell_count as f32
        )
    }
}
