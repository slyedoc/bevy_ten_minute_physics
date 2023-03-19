use bevy::prelude::*;
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct Config {
    #[inspector(min = 1, max = 500)]
    pub sub_steps: u32,
    pub gravity: Vec2,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sub_steps: 100,
            gravity: Vec2::new(0.0, -9.81),
        }
    }
}
