#[repr(align(16))]
#[derive(Debug, Clone)]
pub struct Vec2 {
    pub position: [f32; 2],
}
impl_vertex!(Vec2, position);

#[repr(align(16))]
#[derive(Debug, Clone)]
pub struct Vec3 {
    pub position: [f32; 3],
}
impl_vertex!(Vec3, position);

#[repr(align(16))]
#[derive(Debug, Clone)]
pub struct UVec3 {
    pub position: [u32; 3],
}
impl_vertex!(UVec3, position);

pub trait FromArr3<T> {
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

pub trait FromArr2<T> {
    fn from_arr2(position: [T; 2]) -> Self;
}

impl FromArr2<f32> for Vec2 {
    fn from_arr2(position: [f32; 2]) -> Self {
        Vec2 { position }
    }
}
