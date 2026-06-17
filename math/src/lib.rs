use glam::{Mat4, Vec3};
use rand::RngExt;


pub fn rangle_3(rng:&mut rand::rngs::ThreadRng, from:f32, to:f32) -> Vec3 {
    Vec3::new(rng.random_range(from..to),rng.random_range(from..to),rng.random_range(from..to))
}

/// Angles to a vector 
pub fn atv(angles:&Vec3) -> Vec3 {
    Vec3::new(
        angles.y.cos()+angles.z.cos(),
        angles.x.sin()+angles.z.sin(),
        angles.x.cos()+angles.y.sin(),
    )
}

pub fn lerp_f32(a:f32, b:f32, t:f32) -> f32 {
    a+(b-a)*t
}

pub fn lerp_mat4(a:Mat4, b:Mat4, t:f32) -> Mat4 {
    a+(b-a)*t
}