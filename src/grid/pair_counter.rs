extern crate vulkano;

use vulkano::sync::GpuFuture;

use std::sync::Arc;

const WORKGROUP_SIZE: usize = 256;

mod count_ref {
    #![allow(dead_code)]
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "shaders/count_pairs.comp"]
    struct Dummy;
}

pub struct PairCounter {
    pipeline: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,
    input_ds: Arc<vulkano::descriptor::DescriptorSet + Send + Sync>,
    work_groups_count: usize,
}

impl PairCounter {
    pub fn new(
        queue: Arc<vulkano::device::Queue>,
        positions: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
        indices: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
        triangle_count: usize,
    ) -> PairCounter {
        let device = queue.device();

        let pipeline = Arc::new({
            let shader =
                count_ref::Shader::load(device.clone()).expect("failed to create shader module");
            vulkano::pipeline::ComputePipeline::new(
                device.clone(),
                &shader.main_entry_point(),
                &(),
            ).expect("failed to create compute pipeline")
        });

        let work_groups_count = triangle_count / WORKGROUP_SIZE;
        let work_groups_count = if triangle_count % WORKGROUP_SIZE == 0 {
            work_groups_count
        } else {
            work_groups_count + 1
        };

        let input_ds = Arc::new(
            vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(
                pipeline.clone(),
                0,
            ).add_buffer(positions)
                .unwrap()
                .add_buffer(indices)
                .unwrap()
                .build()
                .unwrap(),
        );

        PairCounter {
            pipeline,
            input_ds,
            work_groups_count,
        }
    }

    pub fn count_pairs(
        &self,
        queue: Arc<vulkano::device::Queue>,
        min_cell: [f32; 3],
        cell_size: [f32; 3],
        grid_resolution: [u32; 3],
    ) -> (
        u32,
        Arc<vulkano::buffer::BufferAccess + Send + Sync>,
        Box<vulkano::sync::GpuFuture>,
    ) {
        let device = queue.device();

        let parameters = vulkano::buffer::CpuAccessibleBuffer::from_data(
            device.clone(),
            vulkano::buffer::BufferUsage::uniform_buffer(),
            count_ref::ty::Params {
                min_cell,
                cell_size,
                resolution: grid_resolution,
                _dummy0: [0; 4],
                _dummy1: [0; 4],
            },
        ).expect("failed to create parameters buffer");

        let cell_count = grid_resolution[0] * grid_resolution[1] * grid_resolution[2];
        let ref_buffer = {
            let data_iter = (0..cell_count).map(|_| 0u32);
            vulkano::buffer::CpuAccessibleBuffer::from_iter(
                device.clone(),
                vulkano::buffer::BufferUsage::all(),
                data_iter,
            ).expect("failed to create cells buffer")
        };

        let output_ds = Arc::new(
            vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(
                self.pipeline.clone(),
                1,
            ).add_buffer(parameters)
                .unwrap()
                .add_buffer(ref_buffer.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let command_buffer =
            vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(
                device.clone(),
                queue.family(),
            ).unwrap()
                .dispatch(
                    [self.work_groups_count as u32, 1, 1],
                    self.pipeline.clone(),
                    (self.input_ds.clone(), output_ds.clone()),
                    (),
                )
                .unwrap()
                .build()
                .unwrap();

        let future = vulkano::sync::now(device.clone())
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();

        let mut pair_count = 0;
        let (cell_buffer, cell_future) = {
            let lock = ref_buffer.read().expect("failed to read cells buffer");
            let data_iter = lock.into_iter().map(|size| {
                let prev_count = pair_count;
                pair_count += size;
                prev_count
            });
            vulkano::buffer::ImmutableBuffer::from_iter(
                data_iter,
                vulkano::buffer::BufferUsage::all(),
                queue.clone(),
            ).expect("failed to create references buffer")
        };
        (pair_count, cell_buffer, Box::new(cell_future) as Box<_>)
    }
}
