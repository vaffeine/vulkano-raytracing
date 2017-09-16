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
