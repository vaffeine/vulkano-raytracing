extern crate image;
extern crate tobj;
extern crate vulkano;

use gl_types::{UVec3, Vec2, Vec3};

use std::path::Path;
use std::sync::Arc;

use cs;

pub struct ModelBuffers {
    pub positions: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub indices: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub normals: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub texcoords: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub models: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub materials: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub textures: Vec<Arc<vulkano::image::ImmutableImage<vulkano::format::R8G8B8A8Srgb>>>,
}

impl ModelBuffers {
    pub fn from_obj(
        path: &Path,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
    ) -> Result<(ModelBuffers, Box<vulkano::sync::GpuFuture>), tobj::LoadError> {
        use tobj;
        let (mut models, mut materials) = tobj::load_obj(&path)?;
        assert!(models.len() == 1);
        let mesh = models.pop().unwrap().mesh;
        let material = materials.pop().unwrap();

        let positions = to_buffer_vec3::<f32, Vec3>(device.clone(), &mesh.positions);
        let indices = to_buffer_vec3::<u32, UVec3>(device.clone(), &mesh.indices);
        let normals = to_buffer_vec3::<f32, Vec3>(device.clone(), &mesh.normals);
        let texcoords = to_buffer_vec2::<f32, Vec2>(device.clone(), &mesh.texcoords);
        let models = vulkano::buffer::CpuAccessibleBuffer::from_data(
            device.clone(),
            vulkano::buffer::BufferUsage::all(),
            cs::ty::Model {
                indices_start: 0,
                indices_end: mesh.indices.len() as u32,
                material_idx: 0,
                _dummy0: [0; 4],
            },
        ).unwrap();
        let (gpu_material, texture, future) =
            load_material(&material, device.clone(), queue.clone()).unwrap();
        let materials = vulkano::buffer::CpuAccessibleBuffer::from_data(
            device.clone(),
            vulkano::buffer::BufferUsage::all(),
            gpu_material,
        ).unwrap();
        let textures = match texture {
            Some(t) => vec![t],
            None => Vec::new(),
        };

        Ok((
            ModelBuffers {
                positions,
                indices,
                normals,
                texcoords,
                models,
                materials,
                textures,
            },
            future,
        ))
    }
}

fn load_texture(
    path: &Path,
    queue: Arc<vulkano::device::Queue>,
) -> image::ImageResult<
    (
        Arc<vulkano::image::ImmutableImage<vulkano::format::R8G8B8A8Srgb>>,
        Box<vulkano::sync::GpuFuture>,
    ),
> {
    let image = image::open(path)?.to_rgba();
    let dimensions = image.dimensions();
    let image_data = image.into_raw().clone();

    let (texture, future) = vulkano::image::immutable::ImmutableImage::from_iter(
        image_data.iter().cloned(),
        vulkano::image::Dimensions::Dim2d {
            width: dimensions.0,
            height: dimensions.1,
        },
        vulkano::format::R8G8B8A8Srgb,
        queue,
    ).unwrap();
    Ok((texture, Box::new(future)))
}

fn load_material(
    material: &tobj::Material,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
) -> image::ImageResult<
    (
        cs::ty::Material,
        Option<Arc<vulkano::image::ImmutableImage<vulkano::format::R8G8B8A8Srgb>>>,
        Box<vulkano::sync::GpuFuture>,
    ),
> {
    let gpu_material = cs::ty::Material {
        ambient: material.ambient,
        diffuse: material.diffuse,
        specular: material.specular,
        shininess: material.shininess,
        dissolve: material.dissolve,
        optical_density: material.optical_density,
        ambient_texture_idx: -1,
        diffuse_texture_idx: 0,
        specular_texture_idx: -1,
        normal_texture_idx: -1,
        disolve_texture_idx: -1,
        _dummy0: [0; 4],
        _dummy1: [0; 4],
        _dummy2: [0; 4],
    };
    if material.diffuse_texture != "" {
        let (texture, future) = load_texture(&Path::new(&material.diffuse_texture), queue)?;
        Ok((gpu_material, Some(texture), future))
    } else {
        Ok((
            gpu_material,
            None,
            Box::new(vulkano::sync::now(device.clone())),
        ))
    }
}

trait FromArr3<T> {
    fn from_arr3(position: [T; 3]) -> Self;
}
impl FromArr3<f32> for Vec3 {
    fn from_arr3(position: [f32; 3]) -> Self {
        Vec3 { position }
    }
}
impl FromArr3<u32> for UVec3 {
    fn from_arr3(position: [u32; 3]) -> Self {
        UVec3 { position }
    }
}

trait FromArr2<T> {
    fn from_arr2(position: [T; 2]) -> Self;
}
impl FromArr2<f32> for Vec2 {
    fn from_arr2(position: [f32; 2]) -> Self {
        Vec2 { position }
    }
}

fn to_buffer_vec2<'a, T, V>(
    device: Arc<vulkano::device::Device>,
    vec: &[T],
) -> Arc<vulkano::buffer::BufferAccess + Send + Sync>
where
    V: 'static + FromArr2<T> + Sync + Send,
    T: Copy,
{
    vulkano::buffer::CpuAccessibleBuffer::from_iter(
        device,
        vulkano::buffer::BufferUsage::all(),
        vec.chunks(2)
            .map(|chunk| V::from_arr2([chunk[0], chunk[1]])),
    ).expect("failed to create indices buffer")
}

fn to_buffer_vec3<'a, T, V>(
    device: Arc<vulkano::device::Device>,
    vec: &[T],
) -> Arc<vulkano::buffer::BufferAccess + Send + Sync>
where
    V: 'static + FromArr3<T> + Sync + Send,
    T: Copy,
{
    vulkano::buffer::CpuAccessibleBuffer::from_iter(
        device,
        vulkano::buffer::BufferUsage::all(),
        vec.chunks(3)
            .map(|chunk| V::from_arr3([chunk[0], chunk[1], chunk[2]])),
    ).expect("failed to create indices buffer")
}
