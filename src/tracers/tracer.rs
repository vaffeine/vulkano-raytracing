extern crate vulkano;
use vulkano::descriptor::descriptor_set;

use super::raycasting;

use control::Camera;
use grid::Grid;
use scene;

use std::sync::Arc;

pub trait TracingShader {
    type Layout;
    type Uniform;
    type Shader;

    fn load_shader(self, device: Arc<vulkano::device::Device>) -> Self::Shader;
    fn new_uniform(self, camera: &Camera, grid: &Grid) -> Self::Uniform;
}

pub struct Tracer<TS: TracingShader> {
    pipeline: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,
    uniform_buffer_pool: vulkano::buffer::CpuBufferPool<TS::Uniform>,
    ds_pool: descriptor_set::FixedSizeDescriptorSetsPool<
        Arc<
            vulkano::pipeline::ComputePipeline<
                vulkano::descriptor::pipeline_layout::PipelineLayout<TS::Layout>,
            >,
        >,
    >,
    model_set: Arc<vulkano::descriptor::DescriptorSet + Send + Sync>,
}

impl<TS: TracingShader<Uniform = raycasting::ty::Uniform, Shader = raycasting::Shader, Layout = raycasting::Layout>>
    Tracer<TS>
{
    pub fn new(
        device: Arc<vulkano::device::Device>,
        scene_buffers: &scene::ModelBuffers,
        traing_shader: TS,
    ) -> Result<Tracer<TS>, descriptor_set::PersistentDescriptorSetError> {
        let shader = traing_shader.load_shader(device.clone());
        let pipeline = Arc::new(
            vulkano::pipeline::ComputePipeline::new(
                device.clone(),
                &shader.main_entry_point(),
                &(),
            ).expect("failed to create compute pipeline"),
        );
        let uniform_buffer_pool = vulkano::buffer::CpuBufferPool::uniform_buffer(device.clone());
        let ds_pool = descriptor_set::FixedSizeDescriptorSetsPool::new(pipeline.clone(), 0);
        let model_set = scene_buffers
            .build_descriptor_set(device.clone(), pipeline.clone(), 1)
            .expect("failed to build scene descriptor set");

        Ok(Tracer {
            pipeline,
            uniform_buffer_pool,
            ds_pool,
            model_set,
        })
    }

    pub fn render(
        &mut self,
        builder: vulkano::command_buffer::AutoCommandBufferBuilder,
        texture: Arc<vulkano::image::StorageImage<vulkano::format::R8G8B8A8Unorm>>,
        statistics: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
        camera: &Camera,
        grid: &Grid,
    ) -> vulkano::command_buffer::AutoCommandBufferBuilder {
        let dimensions = texture.dimensions();
        let uniform_buffer = self.uniform_buffer_pool
            .next(TS::Uniform::new(&camera, &grid))
            .expect("failed to create uniform buffer");
        let ds = self.ds_pool
            .next()
            .add_image(texture)
            .unwrap()
            .add_buffer(uniform_buffer)
            .unwrap()
            .add_buffer(statistics)
            .unwrap()
            .add_buffer(grid.cells_buffer.clone())
            .unwrap()
            .add_buffer(grid.references_buffer.clone())
            .unwrap()
            .build()
            .unwrap();
        builder
            .dispatch(
                [dimensions.width() / 16, dimensions.height() / 16, 1],
                self.pipeline.clone(),
                (ds, self.model_set.clone()),
                (),
            )
            .unwrap()
    }
}
