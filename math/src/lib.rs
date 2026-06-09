use glam::Vec3;
use rand::RngExt;


pub fn rangle_3(rng:&mut rand::rngs::ThreadRng, from:f32, to:f32) -> Vec3 {
    Vec3::new(rng.random_range(from..to),rng.random_range(from..to),rng.random_range(from..to))
}

pub fn atv(angles:&Vec3) -> Vec3 {
    Vec3::new(
        angles.y.cos()+angles.z.cos(),
        angles.x.sin()+angles.z.sin(),
        angles.x.cos()+angles.y.sin(),
    )
}