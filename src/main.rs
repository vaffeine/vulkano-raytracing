// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

#![feature(repr_align)]
#![feature(attr_literals)]

extern crate cgmath;
extern crate winit;
extern crate tobj;

#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate vulkano_text;

mod gl_types;
mod graphics;
mod tracer;
mod fps_counter;
mod input;
mod camera;
mod cs;

use vulkano::sync::{GpuFuture, now};
use vulkano_win::VkSurfaceBuild;
use vulkano_text::{UpdateTextCache, DrawTextTrait};

use graphics::GraphicsPart;
use tracer::ComputePart;
use fps_counter::FPSCounter;
use gl_types::{Vec3, UVec3};
use input::{Keyboard, Mouse};

use std::sync::Arc;
use std::path::Path;

fn obj_to_buffers(path: &Path) -> Result<(Vec<f32>, Vec<u32>), tobj::LoadError> {
    use tobj;
    let (mut models, _) = tobj::load_obj(&path)?;
    assert!(models.len() == 1);
    let mesh = models.pop().unwrap().mesh;
    Ok((mesh.positions, mesh.indices))
}

fn queue_paragraph(drawer: &mut vulkano_text::DrawText, x: f32, y: f32, size: f32, text: &str) {
    for (idx, line) in text.lines().enumerate() {
        drawer.queue_text(
            x,
            y + idx as f32 * size + size / 5.0,
            size,
            [1.0, 1.0, 1.0, 1.0],
            line,
        );
    }
}

fn main() {
    let extensions = vulkano_win::required_extensions();
    let instance = vulkano::instance::Instance::new(None, &extensions, &[])
        .expect("failed to create instance");

    let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");
    println!(
        "Using device: {} (type: {:?})",
        physical.name(),
        physical.ty()
    );

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_dimensions(1280, 720)
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();
    window.window().set_cursor(winit::MouseCursor::NoneCursor);

    let queue = physical
        .queue_families()
        .find(|&q| {
            q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false)
        })
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

    let (positions, indices) = {
        let (pos_vec, ind_vec) = obj_to_buffers(&Path::new(
            &std::env::args().nth(1).expect("no model passed"),
        )).expect("failed to load model");
        (
            vulkano::buffer::CpuAccessibleBuffer::from_iter(
                device.clone(),
                vulkano::buffer::BufferUsage::all(),
                pos_vec.chunks(3).enumerate().map(|(i, chunk)| {
                    let vec = Vec3 { position: [chunk[0], chunk[1], chunk[2] - 5.0] };
                    println!("{}: {:?}", i, vec);
                    vec
                }),
            ).expect("failed to create positions buffer"),
            vulkano::buffer::CpuAccessibleBuffer::from_iter(
                device.clone(),
                vulkano::buffer::BufferUsage::all(),
                ind_vec.chunks(3).map(|chunk| {
                    let vec = UVec3 { position: [chunk[0], chunk[1], chunk[2]] };
                    println!("{:?}", vec);
                    vec
                }),
            ).expect("failed to create indices buffer"),
        )
    };

    let mut graphics = GraphicsPart::new(device.clone(), &window, physical.clone(), queue.clone());
    let mut camera = camera::Camera::new([40.0, 40.0]);
    let uniform_buffer =
        vulkano::buffer::CpuBufferPool::<cs::ty::Constants>::uniform_buffer(device.clone());

    let mut compute = ComputePart::new(
        &device,
        graphics.texture.clone(),
        positions.clone(),
        indices.clone(),
    );
    let mut text_drawer = vulkano_text::DrawText::new(
        device.clone(),
        queue.clone(),
        graphics.swapchain.clone(),
        &graphics.images,
    );

    // TODO: move frambuffers to Graphics part
    let mut previous_frame_end = Box::new(now(device.clone())) as Box<GpuFuture>;

    let mut fps_counter = FPSCounter::new(fps_counter::Duration::milliseconds(100));
    let mut keyboard = Keyboard::new();
    let mut mouse = Mouse::new();

    loop {
        previous_frame_end.cleanup_finished();
        fps_counter.end_frame();

        if graphics.recreate_swapchain(&window) {
            continue;
        }

        graphics.recreate_framebuffers();

        let (image_num, future) = match graphics.acquire_next_image() {
            Ok(r) => r,
            Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                continue;
            }
            Err(err) => panic!("{:?}", err),
        };

        let uniform = Arc::new(uniform_buffer.next(cs::ty::Constants {
            camera: camera.gpu_camera::<cs::ty::Camera>(),
        }));

        let cb = {
            let mut cbb =
                vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(
                    device.clone(),
                    queue.family(),
                ).unwrap()
                    .update_text_cache(&mut text_drawer);

            cbb = compute.render(cbb, graphics.dimensions, uniform);
            cbb = graphics.draw(cbb, image_num);

            cbb.draw_text(
                &mut text_drawer,
                graphics.dimensions[0],
                graphics.dimensions[1],
            ).end_render_pass()
                .unwrap()
                .build()
                .unwrap()
        };

        let future = previous_frame_end
            .join(future)
            .then_execute(queue.clone(), cb)
            .unwrap()
            .then_swapchain_present(queue.clone(), graphics.swapchain.clone(), image_num)
            .then_signal_fence_and_flush()
            .unwrap();
        previous_frame_end = Box::new(future) as Box<_>;

        let current_fps = fps_counter.current_fps();
        let render_time = if current_fps != 0 {
            1000 / current_fps
        } else {
            0
        };
        queue_paragraph(
            &mut text_drawer,
            10.0,
            20.0,
            20.0,
            &format!(
                "Using device: {}\nRender time:  {} ms ({} FPS)\nCamera: {}",
                physical.name(),
                render_time,
                current_fps,
                camera
            ),
        );

        camera.process_keyboard_input(&keyboard, render_time as f32 / 1000.0);
        camera.process_mouse_input(mouse.fetch_mouse_delta());

        let mut done = false;
        events_loop.poll_events(|ev| match ev {
            winit::Event::WindowEvent { event: winit::WindowEvent::Closed, .. } => done = true,
            winit::Event::WindowEvent { event: winit::WindowEvent::Resized(_, _), .. } => {
                graphics.recreate_swapchain = true
            }
            winit::Event::WindowEvent {
                event: winit::WindowEvent::KeyboardInput { input, .. }, ..
            } => {
                keyboard.handle_keypress(&input);
            }
            winit::Event::DeviceEvent {
                event: winit::DeviceEvent::Motion { axis, value }, ..
            } => {
                mouse.handle_mousemove(axis, value);
            }
            _ => (),
        });
        if done {
            return;
        }
    }
}
