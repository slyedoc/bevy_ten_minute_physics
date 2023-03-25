
use bevy::prelude::*;

pub fn ray_sphere_intersect(
    ray: Ray,    
    sphere_center: Vec3,
    sphere_radius: f32,
) -> Option<(f32, f32)> {
    let m = sphere_center - ray.origin;
    let a = ray.direction.dot(ray.direction);
    let b = m.dot(ray.direction);
    let c = m.dot(m) - sphere_radius * sphere_radius;

    let b2 = b * b;
    let delta = b2 - (a * c);

    if delta < 0.0 {
        None
    } else {
        let inv_a = 1.0 / a;
        let delta_root = delta.sqrt();
        let t1 = inv_a * (b - delta_root);
        let t2 = inv_a * (b + delta_root);
        Some((t1, t2))
    }
}

pub fn ray_triangle_intersect(
    ray: Ray,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<f32> {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = ray.direction.cross(edge2);
    let a = edge1.dot(h);

    if a > -0.00001 && a < 0.00001 {
        return None;
    }

    let f = 1.0 / a;
    let s = ray.origin - v0;
    let u = f * s.dot(h);

    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q = s.cross(edge1);
    let v = f * ray.direction.dot(q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * edge2.dot(q);

    if t > 0.00001 {
        Some(t)
    } else {
        None
    }
}