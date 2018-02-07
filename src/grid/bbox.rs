extern crate vulkano;

use vulkano::sync::GpuFuture;

use gl_types::{FromArr3, Vec3, vec3_max, vec3_min};

use std::sync::Arc;
use std::f32;

const WORKGROUP_SIZE: usize = 256;

mod bbox {
    #![allow(dead_code)]
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "shaders/bbox.comp"]
    struct Dummy;
}

pub struct BBox {
    pub min: Vec3,
    pub max: Vec3,
}

pub struct BBoxFinder {
    pipeline: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,
    output_buffer: Arc<vulkano::buffer::CpuAccessibleBuffer<[Vec3]>>,
    descriptor_set: Arc<vulkano::descriptor::DescriptorSet + Send + Sync>,
    work_groups_count: usize,
}

impl BBoxFinder {
    pub fn new(
        queue: Arc<vulkano::device::Queue>,
        positions: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
        triangle_count: usize,
    ) -> BBoxFinder {
        let device = queue.device();

        let pipeline = Arc::new({
            let shader =
                bbox::Shader::load(device.clone()).expect("failed to create shader module");
            vulkano::pipeline::ComputePipeline::new(
                device.clone(),
                &shader.main_entry_point(),
                &(),
            ).expect("failed to create compute pipeline")
        });

        let work_groups_count = triangle_count / (2 * WORKGROUP_SIZE);
        let work_groups_count = if triangle_count % (2 * WORKGROUP_SIZE) == 0 {
            work_groups_count
        } else {
            work_groups_count + 1
        };

        let output_buffer = {
            let data_iter = (0..2 * work_groups_count).map(|_| Vec3::from_arr3([-1.0; 3]));
            vulkano::buffer::CpuAccessibleBuffer::from_iter(
                device.clone(),
                vulkano::buffer::BufferUsage::all(),
                data_iter,
            ).expect("failed to create buffer")
        };

        let descriptor_set = Arc::new(
            vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(
                pipeline.clone(),
                0,
            ).add_buffer(positions)
                .unwrap()
                .add_buffer(output_buffer.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        BBoxFinder {
            pipeline,
            output_buffer,
            descriptor_set,
            work_groups_count,
        }
    }

    pub fn calculate_bbox(
        &self,
        queue: Arc<vulkano::device::Queue>,
        future: Box<vulkano::sync::GpuFuture>,
    ) -> BBox {
        let device = queue.device();

        let command_buffer =
            vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(
                device.clone(),
                queue.family(),
            ).unwrap()
                .dispatch(
                    [self.work_groups_count as u32, 1, 1],
                    self.pipeline.clone(),
                    self.descriptor_set.clone(),
                    (),
                )
                .unwrap()
                .build()
                .unwrap();

        let future = future
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();

        let output = self.output_buffer
            .read()
            .expect("failed to lock bbox output buffer");
        let (minimum_buffer, maximum_buffer) = output.split_at(self.work_groups_count);

        let min = minimum_buffer
            .into_iter()
            .fold(Vec3::from_arr3([f32::MAX; 3]), |min, v| vec3_min(&min, v));
        let max = maximum_buffer
            .into_iter()
            .fold(Vec3::from_arr3([f32::MIN; 3]), |max, v| vec3_max(&max, v));

        BBox { min, max }
    }
}
