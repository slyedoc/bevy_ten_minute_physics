use bevy::prelude::*;

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct Pendulms {
    pub list: Vec<Vec<Entity>>,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct PendulmSegment {
    pub length: f32,
    pub radius: f32,
    pub mass: f32,
    pub prev_pos: Vec2,
    pub velocity: Vec2,
}
