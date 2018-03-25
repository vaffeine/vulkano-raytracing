extern crate vulkano;

use grid;
use scene;
use tracer::Tracer;

use std::path::Path;
use std::sync::Arc;

pub struct VulkanCtx<'a> {
    pub physical: vulkano::instance::PhysicalDevice<'a>,
    pub device: Arc<vulkano::device::Device>,
    pub queue: Arc<vulkano::device::Queue>,
    pub grid_builder: grid::GridBuilder,
    pub tracer: Tracer,
}

impl<'a> VulkanCtx<'a> {
    pub fn new<P>(
        instance: &'a Arc<vulkano::instance::Instance>,
        model_path: &Path,
        predicate: P,
    ) -> (VulkanCtx<'a>, Box<vulkano::sync::GpuFuture>)
    where
        for<'r> P: FnMut(&'r vulkano::instance::QueueFamily) -> bool,
    {
        let physical = vulkano::instance::PhysicalDevice::enumerate(instance)
            .next()
            .expect("no device available");
        println!(
            "Using device: {} (type: {:?})",
            physical.name(),
            physical.ty()
        );
        let queue = physical
            .queue_families()
            .find(predicate)
            .expect("couldn't find a graphical queue family");
        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            ..vulkano::device::DeviceExtensions::none()
        };
        let (device, mut queues) = vulkano::device::Device::new(
            physical,
            physical.supported_features(),
            &device_ext,
            [(queue, 0.5)].iter().cloned(),
        ).expect("failed to create device");
        let queue = queues.next().unwrap();

        let (scene_buffers, load_future) =
            scene::ModelBuffers::from_obj(model_path, device.clone(), queue.clone())
                .expect("failed to load model");

        let tracer = Tracer::new(device.clone(), &scene_buffers).unwrap();

        let grid_builder = grid::GridBuilder::new(
            queue.clone(),
            scene_buffers.positions.clone(),
            scene_buffers.indices.clone(),
            scene_buffers.triangle_count,
        );

        (
            VulkanCtx {
                physical,
                device,
                queue,
                grid_builder,
                tracer,
            },
            load_future,
        )
    }
}
