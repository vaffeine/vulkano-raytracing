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
    ) -> ComputePart<I> {
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
        let persistent_set = descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 1)
            .add_buffer(buffers.positions.clone())
            .unwrap()
            .add_buffer(buffers.indices.clone())
            .unwrap()
            .add_buffer(buffers.normals.clone())
            .unwrap()
            .add_buffer(buffers.texcoords.clone())
            .unwrap()
            .add_buffer(buffers.models.clone())
            .unwrap()
            .add_buffer(buffers.materials.clone())
            .unwrap()
            .add_sampled_image(buffers.textures[0].clone(), sampler)
            .unwrap()
            .build()
            .unwrap();

        ComputePart {
            pipeline: pipeline,
            image: image,
            pool: pool,
            persistent_set: Arc::new(persistent_set),
        }
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
