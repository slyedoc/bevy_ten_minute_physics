use bevy::prelude::*;

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Ball {
    pub radius: f32,
    pub prev_pos: Vec3,
}

impl Default for Ball {
    fn default() -> Self {
        Self {
            radius: 0.5,
            prev_pos: Vec3::ZERO,
        }
    }
}

#[derive(Reflect, Component, Default, Deref, DerefMut)]
#[reflect(Component)]
pub struct Velocity(pub Vec3);
