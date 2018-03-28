extern crate vulkano;

use control::Camera;
use grid::Grid;
use tracers::TracingShader;

use std::sync::Arc;

mod shader {
    #![allow(dead_code)]
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "target/shaders/raycasting.comp"]
    struct Dummy;
}

pub struct RaycastingShader {}

impl TracingShader for RaycastingShader {
    type Uniform = ty::Uniform;
    type Shader = self::shader::Shader;
    type Layout = self::shader::Layout;

    fn load_shader(self, device: Arc<vulkano::device::Device>) -> Self::Shader {
        Self::Shader::load(device.clone()).expect("failed to create shader module")
    }
    fn new_uniform(self, camera: &Camera, grid: &Grid) -> Self::Uniform {
        Self::Uniform::new(camera, grid)
    }
}

pub use self::shader::{ty, Layout, Shader};

impl ty::Uniform {
    pub fn new(camera: &Camera, grid: &Grid) -> ty::Uniform {
        ty::Uniform {
            camera: ty::Camera::new(&camera),
            grid: ty::Grid::new(&grid),
            _dummy0: [0; 4],
        }
    }
}

impl ty::Camera {
    fn new(camera: &Camera) -> ty::Camera {
        let (up, right) = camera.axises();
        ty::Camera {
            position: camera.position(),
            view: camera.view(),
            up,
            right,
            _dummy0: [0; 4],
            _dummy1: [0; 4],
            _dummy2: [0; 4],
        }
    }
}

impl ty::Grid {
    fn new(grid: &Grid) -> ty::Grid {
        ty::Grid {
            minimum_cell: grid.bbox.min.position,
            maximum_cell: grid.bbox.max.position,
            resolution: grid.resolution,
            cell_size: grid.cell_size,
            _dummy0: [0; 4],
            _dummy1: [0; 4],
            _dummy2: [0; 4],
        }
    }
}
