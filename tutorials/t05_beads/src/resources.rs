use bevy::prelude::*;
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct Config {
    pub pixels_per_meter: f32,
    pub bead_count: u32,
    #[inspector(min = 1, max = 1000)]
    pub sub_steps: u32,
    pub gravity: Vec2,
    pub restitution: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pixels_per_meter: 100.0,
            bead_count: 5,
            sub_steps: 100,
            gravity: Vec2::new(0.0, -9.81),
            restitution: 1.0,
        }
    }
}
