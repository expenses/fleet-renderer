use ultraviolet::{Mat3, Mat4, Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PushConstants {
    pub perspective_view: Mat4,
    pub light_dir: Vec3,
}

#[repr(C)]
#[derive(Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub rotation: Mat3,
    pub translation: Vec3,
    pub colour: Vec3,
    pub scale: f32,
    pub diffuse_multiplier: f32,
    pub diffuse_texture: u32,
    pub emissive_texture: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

#[repr(C)]
#[derive(Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BackgroundVertex {
    pub position: Vec3,
    pub colour: Vec3,
}

#[repr(C)]
#[derive(Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LaserVertex {
    pub position: Vec3,
    pub colour: Vec3,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlurSettings {
    pub scale: f32,
    pub strength: f32,
    pub direction: i32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GodraySettings {
    pub density_div_num_samples: f32,
    pub decay: f32,
    pub weight: f32,
    pub num_samples: u32,
    pub uv_space_light_pos: Vec2,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleInstance {
    pub translation: Vec3,
    pub scale: f32,
    pub colour: Vec4,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RangeInstance {
    pub translation: Vec3,
    pub scale: f32,
    pub colour: Vec4,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub pos: Vec2,
    pub colour: Vec3,
}

#[repr(C)]
#[derive(Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndexedIndirect {
    pub index_count: u32,
    pub instance_count: u32, // The number of instances to draw.
    pub base_index: u32,     // The base index within the index buffer.
    pub vertex_offset: i32, // The value added to the vertex index before indexing into the vertex buffer.
    pub base_instance: u32, // The instance ID of the first instance to draw.
}
