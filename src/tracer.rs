extern crate vulkano;
use vulkano::descriptor::descriptor_set;

use std::sync::Arc;

use cs;
use scene;

pub struct ComputePart {
    pipeline: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,
    fsds_pool: descriptor_set::FixedSizeDescriptorSetsPool<
        Arc<
            vulkano::pipeline::ComputePipeline<
                vulkano::descriptor::pipeline_layout::PipelineLayout<cs::Layout>,
            >,
        >,
    >,
    model_set: Arc<vulkano::descriptor::DescriptorSet + Send + Sync>,
}

impl ComputePart {
    pub fn new(
        device: Arc<vulkano::device::Device>,
        buffers: &scene::ModelBuffers,
    ) -> Result<ComputePart, descriptor_set::PersistentDescriptorSetError> {
        let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");
        let pipeline = Arc::new(
            vulkano::pipeline::ComputePipeline::new(
                device.clone(),
                &shader.main_entry_point(),
                &(),
            ).expect("failed to create compute pipeline"),
        );
        let fsds_pool = descriptor_set::FixedSizeDescriptorSetsPool::new(pipeline.clone(), 0);
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

        Ok(ComputePart {
            pipeline: pipeline,
            fsds_pool: fsds_pool,
            model_set: model_set,
        })
    }

    pub fn render(
        &mut self,
        builder: vulkano::command_buffer::AutoCommandBufferBuilder,
        texture: Arc<vulkano::image::StorageImage<vulkano::format::R8G8B8A8Unorm>>,
        uniform: Arc<vulkano::buffer::BufferAccess + Send + Sync + 'static>,
        statistics: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    ) -> vulkano::command_buffer::AutoCommandBufferBuilder {
        let dimensions = texture.dimensions();
        builder
            .dispatch(
                [dimensions.width() / 16, dimensions.height() / 16, 1],
                self.pipeline.clone(),
                (
                    self.next_set(texture, uniform, statistics),
                    self.model_set.clone(),
                ),
                (),
            )
            .unwrap()
    }

    fn next_set(
        &mut self,
        texture: Arc<vulkano::image::StorageImage<vulkano::format::R8G8B8A8Unorm>>,
        uniform: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
        statistics: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    ) -> Arc<vulkano::descriptor::descriptor_set::DescriptorSet + Send + Sync> {
        Arc::new(
            self.fsds_pool
                .next()
                .add_image(texture)
                .unwrap()
                .add_buffer(uniform)
                .unwrap()
                .add_buffer(statistics)
                .unwrap()
                .build()
                .unwrap(),
        )
    }
}
