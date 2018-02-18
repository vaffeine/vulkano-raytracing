extern crate vulkano;

use gl_types::Vec3;

use vulkano::sync::GpuFuture;

use std::sync::Arc;
use std::iter;

const WORKGROUP_SIZE: usize = 256;

mod count_pairs {
    #![allow(dead_code)]
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "shaders/count_pairs.comp"]
    struct Dummy;
}

pub struct PairCounter {
    pipeline: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,
    input_ds: Arc<vulkano::descriptor::DescriptorSet + Send + Sync>,
    uniform_buffer_pool: vulkano::buffer::CpuBufferPool<count_pairs::ty::Params>,
    output_ds_pool: vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool<
        Arc<
            vulkano::pipeline::ComputePipeline<
                vulkano::descriptor::pipeline_layout::PipelineLayout<count_pairs::Layout>,
            >,
        >,
    >,
    triangle_count: usize,
    work_groups_count: usize,
}

pub struct CountPairsResult {
    pub pair_count: usize,
    pub cells_buffer: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub cells_buffer_future: Box<vulkano::sync::GpuFuture>,
    pub min_cells_buffer: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub max_cells_buffer: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
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
                count_pairs::Shader::load(device.clone()).expect("failed to create shader module");
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

        let uniform_buffer_pool = vulkano::buffer::CpuBufferPool::uniform_buffer(device.clone());
        let output_ds_pool =
            vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool::new(
                pipeline.clone(),
                1,
            );

        PairCounter {
            pipeline,
            input_ds,
            uniform_buffer_pool,
            output_ds_pool,
            triangle_count,
            work_groups_count,
        }
    }

    pub fn count_pairs(
        &mut self,
        queue: Arc<vulkano::device::Queue>,
        min_cell: [f32; 3],
        cell_size: [f32; 3],
        grid_resolution: [u32; 3],
    ) -> CountPairsResult {
        let device = queue.device();

        let parameters = self.uniform_buffer_pool
            .next(count_pairs::ty::Params {
                min_cell,
                cell_size,
                resolution: grid_resolution,
                _dummy0: [0; 4],
                _dummy1: [0; 4],
            })
            .expect("failed to create parameters buffer");

        let cell_count = grid_resolution[0] * grid_resolution[1] * grid_resolution[2];
        let ref_buffer = {
            // create one more cell so the last one contains total references count
            let data_iter = (0..cell_count + 1).map(|_| 0u32);
            vulkano::buffer::CpuAccessibleBuffer::from_iter(
                device.clone(),
                vulkano::buffer::BufferUsage::all(),
                data_iter,
            ).expect("failed to create cells buffer")
        };

        let min_cells_buffer = vulkano::buffer::DeviceLocalBuffer::<[Vec3]>::array(
            queue.device().clone(),
            self.triangle_count,
            vulkano::buffer::BufferUsage::all(),
            iter::once(queue.family()),
        ).expect("can't create references buffer");

        let max_cells_buffer = vulkano::buffer::DeviceLocalBuffer::<[Vec3]>::array(
            queue.device().clone(),
            self.triangle_count,
            vulkano::buffer::BufferUsage::all(),
            iter::once(queue.family()),
        ).expect("can't create references buffer");

        let output_ds = self.output_ds_pool
            .next()
            .add_buffer(parameters)
            .unwrap()
            .add_buffer(ref_buffer.clone())
            .unwrap()
            .add_buffer(min_cells_buffer.clone())
            .unwrap()
            .add_buffer(max_cells_buffer.clone())
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
                    (self.input_ds.clone(), output_ds),
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
        let (cells_buffer, cells_future) = {
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
        CountPairsResult {
            pair_count: pair_count as usize,
            cells_buffer,
            cells_buffer_future: Box::new(cells_future),
            min_cells_buffer,
            max_cells_buffer,
        }
    }
}
