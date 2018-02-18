extern crate vulkano;

use super::pair_counter::CountPairsResult;

use vulkano::sync::GpuFuture;

use std::sync::Arc;
use std::iter;

const WORKGROUP_SIZE: usize = 256;

mod write_pairs {
    #![allow(dead_code)]
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "shaders/write_pairs.comp"]
    struct Dummy;
}

pub struct PairWriter {
    pipeline: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,
    uniform_buffer_pool: vulkano::buffer::CpuBufferPool<write_pairs::ty::Params>,
    ds_pool: vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool<
        Arc<
            vulkano::pipeline::ComputePipeline<
                vulkano::descriptor::pipeline_layout::PipelineLayout<write_pairs::Layout>,
            >,
        >,
    >,
    work_groups_count: usize,
}

impl PairWriter {
    pub fn new(queue: Arc<vulkano::device::Queue>, triangle_count: usize) -> PairWriter {
        let device = queue.device();

        let pipeline = Arc::new({
            let shader =
                write_pairs::Shader::load(device.clone()).expect("failed to create shader module");
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

        let uniform_buffer_pool = vulkano::buffer::CpuBufferPool::uniform_buffer(device.clone());
        let ds_pool = vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool::new(
            pipeline.clone(),
            0,
        );

        PairWriter {
            pipeline,
            uniform_buffer_pool,
            ds_pool,
            work_groups_count,
        }
    }

    pub fn write_pairs(
        &mut self,
        queue: Arc<vulkano::device::Queue>,
        count_pairs_result: CountPairsResult,
        resolution: [u32; 3],
    ) -> (
        Arc<vulkano::buffer::BufferAccess + Send + Sync>,
        Arc<vulkano::buffer::BufferAccess + Send + Sync>,
        Box<vulkano::sync::GpuFuture>,
    ) {
        let device = queue.device();

        let params_buffer = self.uniform_buffer_pool
            .next(write_pairs::ty::Params { resolution })
            .expect("failed to create params buffer");

        let cell_count = resolution[0] * resolution[1] * resolution[2];
        let (current_idx_buffer, current_idx_future) =
            vulkano::buffer::ImmutableBuffer::from_iter(
                (0..cell_count).map(|_| 0u32),
                vulkano::buffer::BufferUsage::all(),
                queue.clone(),
            ).expect("failed to create references buffer");

        let ref_buffer = vulkano::buffer::DeviceLocalBuffer::<[u32]>::array(
            queue.device().clone(),
            count_pairs_result.pair_count,
            vulkano::buffer::BufferUsage::all(),
            iter::once(queue.family()),
        ).expect("can't create references buffer");

        let descriptor_set = self.ds_pool
            .next()
            .add_buffer(params_buffer)
            .unwrap()
            .add_buffer(count_pairs_result.min_cells_buffer)
            .unwrap()
            .add_buffer(count_pairs_result.max_cells_buffer)
            .unwrap()
            .add_buffer(count_pairs_result.cells_buffer.clone())
            .unwrap()
            .add_buffer(current_idx_buffer)
            .unwrap()
            .add_buffer(ref_buffer.clone())
            .unwrap()
            .build()
            .unwrap();

        let command_buffer =
            vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(
                device.clone(),
                queue.family(),
            ).unwrap()
                .dispatch(
                    [self.work_groups_count as u32, 1, 1],
                    self.pipeline.clone(),
                    descriptor_set,
                    (),
                )
                .unwrap()
                .build()
                .unwrap();

        let future = count_pairs_result
            .cells_buffer_future
            .join(current_idx_future)
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        (
            count_pairs_result.cells_buffer,
            ref_buffer,
            Box::new(future),
        )
    }
}
