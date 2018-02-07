extern crate image;
extern crate tobj;
extern crate vulkano;

use gl_types::{FromArr2, FromArr3, UVec3, Vec2, Vec3};

use vulkano::sync::GpuFuture;

use std::path::Path;
use std::sync::Arc;

use cs;

pub struct ModelBuffers {
    pub models: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub positions: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub indices: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub normals: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub texcoords: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub materials: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    pub textures: Vec<Arc<vulkano::image::ImmutableImage<vulkano::format::R8G8B8A8Srgb>>>,
    pub triangle_count: usize,
}

impl ModelBuffers {
    pub fn from_obj(
        path: &Path,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
    ) -> Result<(ModelBuffers, Box<vulkano::sync::GpuFuture>), tobj::LoadError> {
        use tobj;
        let (obj_models, obj_materials) = tobj::load_obj(&path)?;
        let (models, positions, indices, normals, texcoords) = load_mesh(obj_models);
        let (materials, textures, textures_future) =
            load_materials(device.clone(), queue.clone(), obj_materials);

        let (buffer_models, models_future) = vulkano::buffer::ImmutableBuffer::from_iter(
            models.into_iter(),
            vulkano::buffer::BufferUsage {
                storage_buffer: true,
                ..vulkano::buffer::BufferUsage::none()
            },
            queue.clone(),
        ).unwrap();

        let (buffer_positions, positions_future) =
            to_buffer_vec3::<f32, Vec3>(queue.clone(), &positions);
        let (buffer_indices, indices_future) =
            to_buffer_vec3::<u32, UVec3>(queue.clone(), &indices);
        let (buffer_normals, normals_future) =
            to_buffer_vec3::<f32, Vec3>(queue.clone(), &normals);
        let (buffer_texcoords, texcoords_future) =
            to_buffer_vec2::<f32, Vec2>(queue.clone(), &texcoords);
        let (buffer_materials, materials_future) = vulkano::buffer::ImmutableBuffer::from_iter(
            materials.into_iter(),
            vulkano::buffer::BufferUsage {
                storage_buffer: true,
                ..vulkano::buffer::BufferUsage::none()
            },
            queue.clone(),
        ).unwrap();

        let future = Box::new(
            textures_future
                .join(models_future)
                .join(positions_future)
                .join(indices_future)
                .join(normals_future)
                .join(texcoords_future)
                .join(materials_future),
        ) as Box<_>;

        Ok((
            ModelBuffers {
                models: buffer_models,
                positions: buffer_positions,
                indices: buffer_indices,
                normals: buffer_normals,
                texcoords: buffer_texcoords,
                materials: buffer_materials,
                textures: textures,
                triangle_count: indices.len() / 3,
            },
            future,
        ))
    }
}

fn load_materials(
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
    obj_materials: Vec<tobj::Material>,
) -> (
    Vec<cs::ty::Material>,
    Vec<Arc<vulkano::image::ImmutableImage<vulkano::format::R8G8B8A8Srgb>>>,
    Box<vulkano::sync::GpuFuture>,
) {
    let mut materials = Vec::new();
    let mut textures = Vec::with_capacity(16);
    let (ei, mut future) = empty_image(queue.clone());
    for obj_material in obj_materials {
        let (material, texture, f) = load_material(
            &obj_material,
            textures.len() as i32,
            device.clone(),
            queue.clone(),
        ).unwrap();
        materials.push(material);
        future = Box::new(future.join(f));
        match texture {
            Some(t) => {
                textures.push(t);
            }
            None => (),
        };
    }
    for _ in 0..16 - textures.len() {
        textures.push(ei.clone());
    }
    (materials, textures, future)
}

fn load_mesh(
    obj_models: Vec<tobj::Model>,
) -> (Vec<cs::ty::Model>, Vec<f32>, Vec<u32>, Vec<f32>, Vec<f32>) {
    let mut models = Vec::new();
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut texcoords = Vec::new();

    for obj_model in obj_models {
        let mut mesh = obj_model.mesh;

        let material_idx = match mesh.material_id {
            Some(id) => id as i32,
            None => -1,
        };
        models.push(cs::ty::Model {
            indices_start: indices.len() as u32 / 3,
            indices_end: (indices.len() + mesh.indices.len()) as u32 / 3,
            material_idx: material_idx,
            _dummy0: [0; 4],
        });

        indices.extend(
            mesh.indices
                .into_iter()
                .map(|i| i + positions.len() as u32 / 3),
        );
        positions.append(&mut mesh.positions);
        normals.append(&mut mesh.normals);
        texcoords.append(&mut mesh.texcoords);
    }
    (models, positions, indices, normals, texcoords)
}

fn empty_image(
    queue: Arc<vulkano::device::Queue>,
) -> (
    Arc<vulkano::image::ImmutableImage<vulkano::format::R8G8B8A8Srgb>>,
    Box<vulkano::sync::GpuFuture>,
) {
    let pixel = vec![255u8; 4];
    let (texture, future) = vulkano::image::immutable::ImmutableImage::from_iter(
        pixel.into_iter(),
        vulkano::image::Dimensions::Dim2d {
            width: 1,
            height: 1,
        },
        vulkano::format::R8G8B8A8Srgb,
        queue,
    ).unwrap();
    (texture, Box::new(future))
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
    texture_idx: i32,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
) -> image::ImageResult<
    (
        cs::ty::Material,
        Option<Arc<vulkano::image::ImmutableImage<vulkano::format::R8G8B8A8Srgb>>>,
        Box<vulkano::sync::GpuFuture>,
    ),
> {
    let (texture, future, texture_idx) = if material.diffuse_texture != "" {
        let (texture, future) = load_texture(&Path::new(&material.diffuse_texture), queue)?;
        (Some(texture), future, texture_idx)
    } else {
        (
            None,
            Box::new(vulkano::sync::now(device.clone())) as Box<vulkano::sync::GpuFuture>,
            -1,
        )
    };
    let gpu_material = cs::ty::Material {
        ambient: material.ambient,
        diffuse: material.diffuse,
        specular: material.specular,
        shininess: material.shininess,
        dissolve: material.dissolve,
        optical_density: material.optical_density,
        ambient_texture_idx: -1,
        diffuse_texture_idx: texture_idx,
        specular_texture_idx: -1,
        normal_texture_idx: -1,
        disolve_texture_idx: -1,
        _dummy0: [0; 4],
        _dummy1: [0; 4],
        _dummy2: [0; 4],
    };
    Ok((gpu_material, texture, future))
}

fn to_buffer_vec2<'a, T, V>(
    queue: Arc<vulkano::device::Queue>,
    vec: &[T],
) -> (
    Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    Box<vulkano::sync::GpuFuture>,
)
where
    V: 'static + FromArr2<T> + Sync + Send,
    T: Copy,
{
    let (buffer, future) = vulkano::buffer::ImmutableBuffer::from_iter(
        vec.chunks(2)
            .map(|chunk| V::from_arr2([chunk[0], chunk[1]])),
        vulkano::buffer::BufferUsage {
            storage_buffer: true,
            ..vulkano::buffer::BufferUsage::none()
        },
        queue,
    ).expect("failed to create indices buffer");
    (buffer, Box::new(future))
}

fn to_buffer_vec3<'a, T, V>(
    queue: Arc<vulkano::device::Queue>,
    vec: &[T],
) -> (
    Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    Box<vulkano::sync::GpuFuture>,
)
where
    V: 'static + FromArr3<T> + Sync + Send,
    T: Copy,
{
    let (buffer, future) = vulkano::buffer::ImmutableBuffer::from_iter(
        vec.chunks(3)
            .map(|chunk| V::from_arr3([chunk[0], chunk[1], chunk[2]])),
        vulkano::buffer::BufferUsage {
            storage_buffer: true,
            ..vulkano::buffer::BufferUsage::none()
        },
        queue,
    ).expect("failed to create indices buffer");
    (buffer, Box::new(future))
}
