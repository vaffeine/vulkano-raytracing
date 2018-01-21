#![allow(dead_code)]
#[derive(VulkanoShader)]
#[ty = "compute"]
#[path = "shaders/raytracing.comp"]
struct Dummy;

use camera::GPUCamera;

impl GPUCamera for ty::Camera {
    fn new(position: [f32; 3], view: [f32; 3], up: [f32; 3], right: [f32; 3]) -> Self {
        ty::Camera {
            position: position,
            view: view,
            up: up,
            right: right,
            _dummy0: [0; 4],
            _dummy1: [0; 4],
            _dummy2: [0; 4],
        }
    }
}
