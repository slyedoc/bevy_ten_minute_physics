use bevy::prelude::*;

use crate::resources::Config;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Wire {
    pub radius: f32,
    pub line_segments: u32,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Bead {
    pub radius: f32,
    pub mass: f32,
    pub prev_position: Vec2,
    pub velocity: Vec2,
}

impl Bead {
    pub fn start_step(&mut self, transform: &mut Transform, dt: f32, config: &Config) {
        self.velocity += config.gravity * config.pixels_per_meter * dt;
        self.prev_position = transform.translation.truncate();
        transform.translation += self.velocity.extend(0.0) * dt;
    }

    pub fn keep_on_wire(
        &mut self,
        wire_center: Vec3,
        wire_radius: f32,
        transform: &mut Transform,
    ) {
        let mut dir = transform.translation - wire_center;
        let len = dir.length();
        if len == 0.0 {
            return;
        }
        dir = dir.normalize_or_zero();

        let lambda = wire_radius - len;
        transform.translation += dir * lambda;
    }

    pub fn end_step(&mut self, transform: &mut Transform, dt: f32) {
        self.velocity = transform.translation.truncate() - self.prev_position;
        if dt > 0.0 {
            self.velocity *= 1.0 / dt;
        }
        
    }
}
