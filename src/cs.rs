use camera::Camera;
use grid::Grid;

mod shader {
    #![allow(dead_code)]
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "shaders/tracer.comp"]
    struct Dummy;
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
