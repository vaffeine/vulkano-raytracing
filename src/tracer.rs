extern crate vulkano;
use vulkano::descriptor::descriptor_set;

use std::sync::Arc;

use camera::Camera;
use cs;
use grid::Grid;
use scene;

type TracerDescriptorSetsPool = descriptor_set::FixedSizeDescriptorSetsPool<
    Arc<
        vulkano::pipeline::ComputePipeline<
            vulkano::descriptor::pipeline_layout::PipelineLayout<cs::Layout>,
        >,
    >,
>;

pub struct Tracer {
    pipeline: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,
    uniform_buffer_pool: vulkano::buffer::CpuBufferPool<cs::ty::Uniform>,
    ds_pool: TracerDescriptorSetsPool,
    model_set: Arc<vulkano::descriptor::DescriptorSet + Send + Sync>,
}

impl Tracer {
    pub fn new(
        device: Arc<vulkano::device::Device>,
        buffers: &scene::ModelBuffers,
    ) -> Result<Tracer, descriptor_set::PersistentDescriptorSetError> {
        let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");
        let pipeline = Arc::new(
            vulkano::pipeline::ComputePipeline::new(
                device.clone(),
                &shader.main_entry_point(),
                &(),
            ).expect("failed to create compute pipeline"),
        );
        let uniform_buffer_pool = vulkano::buffer::CpuBufferPool::uniform_buffer(device.clone());
        let ds_pool = descriptor_set::FixedSizeDescriptorSetsPool::new(pipeline.clone(), 0);
        let sampler = vulkano::sampler::Sampler::simple_repeat_linear(device.clone());
        let model_set = Arc::new(
            descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 1)
                .add_buffer(buffers.positions.clone())?
                .add_buffer(buffers.indices.clone())?
                .add_buffer(buffers.normals.clone())?
                .add_buffer(buffers.texcoords.clone())?
                .add_buffer(buffers.models.clone())?
                .add_buffer(buffers.materials.clone())?
                .enter_array()?
                .add_sampled_image(buffers.textures[0].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[1].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[2].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[3].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[4].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[5].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[6].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[7].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[8].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[9].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[10].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[11].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[12].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[13].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[14].clone(), sampler.clone())?
                .add_sampled_image(buffers.textures[15].clone(), sampler.clone())?
                .leave_array()?
                .build()
                .unwrap(),
        );

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
            .next(cs::ty::Uniform::new(&camera, &grid))
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
