#![feature(attr_literals)]

extern crate cgmath;
extern crate time;
extern crate tobj;
extern crate winit;

#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_text;
extern crate vulkano_win;

#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate regex;

mod args;
mod camera;
mod cs;
mod drawer;
mod event_manager;
mod fps_counter;
mod gl_types;
mod grid;
mod input;
mod offline;
mod realtime;
mod scene;
mod tracer;
mod vulkan_ctx;

use args::Args;
use event_manager::EventManager;
use fps_counter::FPSCounter;
use realtime::RealTimeRender;
use offline::OfflineRender;

fn get_layers<'a>(desired_layers: Vec<&'a str>) -> Vec<&'a str> {
    let available_layers: Vec<_> = vulkano::instance::layers_list().unwrap().collect();
    println!("Available layers:");
    for l in &available_layers {
        println!("\t{}", l.name());
    }
    desired_layers
        .into_iter()
        .filter(|&l| available_layers.iter().any(|li| li.name() == l))
        .collect()
}

fn print_message_callback(msg: &vulkano::instance::debug::Message) {
    lazy_static! {
        // Validation layers spams this error message, although this error is false positive
        // https://github.com/vulkano-rs/vulkano/issues/831
        static ref FENCE_ERROR_RE: regex::Regex =
            regex::Regex::new(r"Fence 0x\w* is in use.").unwrap();
    }
    if FENCE_ERROR_RE.is_match(msg.description) {
        return;
    }

    let ty = if msg.ty.error {
        "error"
    } else if msg.ty.warning {
        "warning"
    } else if msg.ty.performance_warning {
        "perf"
    } else if msg.ty.information {
        "info"
    } else if msg.ty.debug {
        "debug"
    } else {
        panic!("no-impl");
    };
    println!("{} [{}] : {}", msg.layer_prefix, ty, msg.description);
}

fn main() {
    let args = Args::get_matches();
    let extensions = vulkano::instance::InstanceExtensions {
        ext_debug_report: true,
        ..vulkano_win::required_extensions()
    };
    let layers = get_layers(vec!["VK_LAYER_LUNARG_standard_validation"]);
    println!("Using layers: {:?}", layers);
    let instance = vulkano::instance::Instance::new(None, &extensions, &layers)
        .expect("failed to create instance");

    let _debug_callback = vulkano::instance::debug::DebugCallback::new(
        &instance,
        args.log_level,
        print_message_callback,
    ).ok();

    let mut camera = camera::Camera::with_position(args.position, args.fov);

    if args.benchmark {
        let mut render = OfflineRender::new(&args, &instance, [args.resolution[0], args.resolution[1]]);
        let statistics = render.render(&camera);
        println!("=============== Statistics ===============");
        println!("{}", statistics);
    } else {
        let mut events_loop = winit::EventsLoop::new();
        let mut event_manager = EventManager::new();
        let mut fps_counter = FPSCounter::new(fps_counter::Duration::milliseconds(
            args.fps_update_interval,
        ));

        let mut render = RealTimeRender::new(&args, &events_loop, &instance);
        let mut previous_frame_end =
            Box::new(vulkano::sync::now(render.vulkan_ctx.device.clone())) as Box<_>;

        loop {
            previous_frame_end = render.render(
                &mut camera,
                &fps_counter,
                event_manager.recreate_swapchain(),
                previous_frame_end,
            );
            fps_counter.end_frame();

            events_loop.poll_events(|event| event_manager.process_event(event));
            camera.process_mouse_input(event_manager.mouse.fetch_mouse_delta());
            camera.process_keyboard_input(
                &event_manager.keyboard,
                args.sensitivity * fps_counter.average_render_time() as f32 / 1000.0,
            );
            if event_manager.done() {
                break;
            }
        }
    }
}
