extern crate vulkano;
use vulkano::descriptor::descriptor_set;

use std::sync::Arc;

use cs;
use scene;

pub struct ComputePart<I: 'static + vulkano::image::traits::ImageViewAccess + Send + Sync> {
    pipeline: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,
    image: Arc<I>,
    pool: descriptor_set::FixedSizeDescriptorSetsPool<
        Arc<
            vulkano::pipeline::ComputePipeline<
                vulkano::descriptor::pipeline_layout::PipelineLayout<cs::Layout>,
            >,
        >,
    >,
    persistent_set: Arc<vulkano::descriptor::DescriptorSet + Send + Sync>,
}

impl<I: 'static + vulkano::image::traits::ImageViewAccess + Send + Sync> ComputePart<I> {
    pub fn new(
        device: &Arc<vulkano::device::Device>,
        image: Arc<I>,
        buffers: scene::ModelBuffers,
    ) -> Result<ComputePart<I>, descriptor_set::PersistentDescriptorSetError> {
        let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");
        let pipeline = Arc::new(
            vulkano::pipeline::ComputePipeline::new(
                device.clone(),
                &shader.main_entry_point(),
                &(),
            ).expect("failed to create compute pipeline"),
        );
        let sampler = vulkano::sampler::Sampler::simple_repeat_linear(device.clone());
        let pool = descriptor_set::FixedSizeDescriptorSetsPool::new(pipeline.clone(), 0);
        let persistent_set = Arc::new(
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
            image: image,
            pool: pool,
            persistent_set: persistent_set,
        })
    }

    pub fn render(
        &mut self,
        builder: vulkano::command_buffer::AutoCommandBufferBuilder,
        dimensions: [u32; 2],
        uniform: Arc<vulkano::buffer::BufferAccess + Send + Sync + 'static>,
    ) -> vulkano::command_buffer::AutoCommandBufferBuilder {
        builder
            .dispatch(
                [dimensions[0] / 16, dimensions[1] / 16, 1],
                self.pipeline.clone(),
                (self.next_set(uniform.clone()), self.persistent_set.clone()),
                (),
            )
            .unwrap()
    }

    fn next_set(
        &mut self,
        uniform: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    ) -> Arc<vulkano::descriptor::descriptor_set::DescriptorSet + Send + Sync> {
        Arc::new(
            self.pool
                .next()
                .add_image(self.image.clone())
                .unwrap()
                .add_buffer(uniform)
                .unwrap()
                .build()
                .unwrap(),
        )
    }
}
