use glam::{Vec2, Vec3, Vec4};

pub struct CricleDescriptor {
    pub centre: Vec2,
    pub radius: f32,
    pub color: Vec3,
}

pub struct LineDescriptor {
    pub start: Vec2,
    pub end: Vec2,
    pub color: Vec3,
    pub stroke: f32,
}

pub struct VerticalLineDescriptor {
    pub y: u32,
    pub top_x: u32,
    pub bottom_x: u32,
    pub color: Vec4,
}