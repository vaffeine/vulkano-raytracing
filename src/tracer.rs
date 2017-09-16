extern crate vulkano;
use vulkano::descriptor::descriptor_set;

use std::sync::Arc;

use gl_types::{Vec3, UVec3};

use super::cs;

pub struct ComputePart<I: 'static + vulkano::image::traits::ImageViewAccess + Send + Sync> {
    pub pipeline: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,
    image: Arc<I>,
    positions: Arc<vulkano::buffer::CpuAccessibleBuffer<[Vec3]>>,
    indices: Arc<vulkano::buffer::CpuAccessibleBuffer<[UVec3]>>,
}

impl<I: 'static + vulkano::image::traits::ImageViewAccess + Send + Sync> ComputePart<I> {
    pub fn new(device: &Arc<vulkano::device::Device>,
               image: Arc<I>,
               positions: Arc<vulkano::buffer::CpuAccessibleBuffer<[Vec3]>>,
               indices: Arc<vulkano::buffer::CpuAccessibleBuffer<[UVec3]>>)
               -> ComputePart<I> {
        let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");
        let pipeline =
            Arc::new(vulkano::pipeline::ComputePipeline::new(device.clone(),
                                                             &shader.main_entry_point(),
                                                             &())
                .expect("failed to create compute pipeline"));

        ComputePart {
            pipeline: pipeline,
            image: image,
            positions: positions,
            indices: indices,
        }
    }

    pub fn next_set(&mut self,
                    uniform: Arc<vulkano::buffer::BufferAccess + Send + Sync>)
                    -> Arc<vulkano::descriptor::descriptor_set::DescriptorSet + Send + Sync> {
        Arc::new(descriptor_set::PersistentDescriptorSet::start(self.pipeline.clone(), 0)
            .add_image(self.image.clone())
            .unwrap()
            .add_buffer(uniform)
            .unwrap()
            .add_buffer(self.positions.clone())
            .unwrap()
            .add_buffer(self.indices.clone())
            .unwrap()
            .build()
            .unwrap())
    }
}
