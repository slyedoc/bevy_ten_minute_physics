use bevy::prelude::*;

#[derive(Component)]
pub struct Ball {
    pub radius: f32,
}

#[derive(Reflect, Component, Default, Deref, DerefMut)]
#[reflect(Component)]
pub struct Velocity(pub Vec2);

#[derive(Reflect, Component, Default, Deref, DerefMut)]
#[reflect(Component)]
pub struct Mass(pub f32);

#[derive(Reflect, Component, Default, Deref, DerefMut)]
#[reflect(Component)]
pub struct Restitution(pub f32);

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Flipper {
    pub prev_rotation: f32,
    pub radius: f32,
    pub length: f32,
    pub max_rotation: f32,
    pub rest_angle: f32,
    pub angular_velocity: f32,
    pub current_angular_velocity: f32,
    pub sign: f32,
    pub key: KeyCode,
    // pressed
    //pub pressed: bool,
}

impl Default for Flipper {
    fn default() -> Self {
        Self {
            prev_rotation: 0.0,
            radius: 0.5,
            length: 1.0,
            max_rotation: 0.5,
            rest_angle: 0.0,
            sign: 1.0,
            angular_velocity: 0.0,
            current_angular_velocity: 0.0,
            key: KeyCode::Space,
        }
    }
}

impl Flipper {
    pub fn new(
        radius: f32,
        length: f32,
        max_rotation: f32,
        rest_angle: f32,
        angular_velocity: f32,
        sign: f32,
        key: KeyCode,
    ) -> Self {
        let rest = Quat::from_axis_angle(Vec3::Z, rest_angle)
            .to_euler(EulerRot::YXZ)
            .2;
        let max_rot = Quat::from_axis_angle(Vec3::Z, rest_angle + max_rotation)
            .to_euler(EulerRot::YXZ)
            .2;
        Self {
            radius,
            length,
            max_rotation: max_rot,
            rest_angle: rest,
            angular_velocity,
            sign,
            key,
            ..default()
        }
    }

    pub fn get_tip(&self, trans: &Transform) -> Vec3 {
        return trans.transform_point(Vec3::new(0., self.length, 0.));
    }
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Obstacle {
    pub radius: f32,
    pub push_velocity: f32,
}

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct Border {
    pub points: Vec<Vec3>,
}

impl Border {
    pub fn push(&mut self, point: Vec2) {
        self.points.push(Vec3::new(point.x, point.y, 0.));
    }
}
