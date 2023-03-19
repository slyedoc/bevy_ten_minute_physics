use bevy::prelude::*;

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Ball(pub f32);

impl Default for Ball {
    fn default() -> Self {
        Self(0.5)
    }
}

#[derive(Reflect, Component, Default, Deref, DerefMut)]
#[reflect(Component)]
pub struct Velocity(pub Vec3);
