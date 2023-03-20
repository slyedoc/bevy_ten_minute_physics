use bevy::prelude::*;
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct Config {
    pub scale: f32,
    #[inspector(min = 10., max = 100.)]
    pub half_size: f32,
    #[inspector(min = 0, max = 100)]
    pub sub_steps: u32,
    pub gravity: Vec3,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scale: 1.,
            half_size: 10.,
            sub_steps: 5,
            gravity: Vec3::new(0., -9.81, 0.),
        }
    }
}