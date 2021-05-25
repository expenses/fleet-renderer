mod gpu_buffer;
mod keyboard;
mod mouse;

pub use gpu_buffer::{GpuBuffer, ShipBuffer};
pub use keyboard::KeyboardState;
pub use mouse::{MouseButtonState, MouseState};
pub use ray_collisions::{BoundingBox, Projectile, Ray, SelectionFrustum};

use crate::components::ModelId;
use crate::model::Model;
use bevy_ecs::prelude::Entity;
use ultraviolet::{Mat4, Vec2, Vec3};

pub struct Paused(pub bool);

#[derive(Clone, Copy)]
pub enum MouseMode {
    Normal,
    Movement { plane_y: f32 },
}

#[derive(Default)]
pub struct AverageSelectedPosition(pub Option<Vec3>);

#[derive(Default)]
pub struct RayPlanePoint(pub Option<Vec3>);

pub struct TotalTime(pub f32);

pub struct DeltaTime(pub f32);

pub struct GpuInterface {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[derive(Default)]
pub struct ShipUnderCursor(pub Option<Entity>);

pub struct Models(pub [Model; Self::COUNT]);

impl Models {
    pub const COUNT: usize = 4;

    pub fn get(&self, id: ModelId) -> &Model {
        &self.0[id as usize]
    }
}

#[derive(Default)]
pub struct Camera {
    pub center: Vec3,
}

pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub fn to_vec(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
}

pub struct Orbit {
    pub longitude: f32,
    pub latitude: f32,
    distance: f32,
}

impl Orbit {
    pub fn new() -> Self {
        Self {
            longitude: 1.0,
            latitude: 0.0,
            distance: 10.0,
        }
    }

    pub fn rotate(&mut self, delta: Vec2) {
        use std::f32::consts::PI;
        let speed = 0.15;
        self.latitude -= delta.x.to_radians() * speed;
        self.longitude = (self.longitude - delta.y.to_radians() * speed)
            .max(std::f32::EPSILON)
            .min(PI - std::f32::EPSILON);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance * (1.0 + delta * 0.1)).max(1.0).min(250.0);
    }

    pub fn as_vector(&self) -> Vec3 {
        let y = self.longitude.cos();
        let horizontal_amount = self.longitude.sin();
        let x = horizontal_amount * self.latitude.sin();
        let z = horizontal_amount * self.latitude.cos();
        Vec3::new(x, y, z) * self.distance
    }
}

#[derive(Clone)]
pub struct PerspectiveView {
    pub perspective: Mat4,
    perspective_with_far_plane: Mat4,
    pub view: Mat4,
    view_without_movement: Mat4,
    pub perspective_view: Mat4,
    pub perspective_view_without_movement: Mat4,
    pub perspective_view_with_far_plane: Mat4,
}

impl PerspectiveView {
    pub fn new(fov: f32, aspect_ratio: f32, eye: Vec3, center: Vec3) -> Self {
        let perspective =
            ultraviolet::projection::perspective_infinite_z_wgpu_dx(fov, aspect_ratio, 0.1);
        let perspective_with_far_plane =
            ultraviolet::projection::perspective_wgpu_dx(fov, aspect_ratio, 0.1, 1000.0);

        let view = Mat4::look_at(eye + center, center, Vec3::unit_y());
        let view_without_movement = Mat4::look_at(Vec3::zero(), -eye, Vec3::unit_y());

        Self {
            view,
            view_without_movement,
            perspective,
            perspective_with_far_plane,
            perspective_view: perspective * view,
            perspective_view_without_movement: perspective * view_without_movement,
            perspective_view_with_far_plane: perspective_with_far_plane * view,
        }
    }

    fn recalculate(&mut self) {
        self.perspective_view = self.perspective * self.view;
        self.perspective_view_without_movement = self.perspective * self.view_without_movement;
        self.perspective_view_with_far_plane = self.perspective_with_far_plane * self.view;
    }

    pub fn set_perspective(&mut self, fov: f32, aspect_ratio: f32) {
        self.perspective =
            ultraviolet::projection::perspective_infinite_z_wgpu_dx(fov, aspect_ratio, 0.1);
        self.perspective_with_far_plane =
            ultraviolet::projection::perspective_wgpu_dx(fov, aspect_ratio, 0.1, 1000.0);
        self.recalculate();
    }

    pub fn set_view(&mut self, orbit: Vec3, center: Vec3) {
        self.view = Mat4::look_at(orbit + center, center, Vec3::unit_y());
        self.view_without_movement = Mat4::look_at(Vec3::zero(), -orbit, Vec3::unit_y());
        self.recalculate();
    }
}
