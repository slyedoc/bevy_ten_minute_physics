use std::f32::consts::PI;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::{ResourceInspectorPlugin};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ResetState {
    Playing,
    Reset,
    
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugin(WorldInspectorPlugin)
        .init_resource::<Config>()
        .register_type::<Config>()
        .add_plugin(ResourceInspectorPlugin::<Config>::default())
        .add_state(ResetState::Playing)
        .insert_resource(ClearColor(Color::WHITE))
        .register_type::<Mass>()
        .register_type::<Velocity>()
        .add_system_set(SystemSet::on_enter(ResetState::Playing).with_system(spawn_balls))
        .add_system_set(SystemSet::on_update(ResetState::Playing).with_system(reset_listen))
        .add_system_set(SystemSet::on_update(ResetState::Reset).with_system(reset))
        .add_startup_system(setup)
        .add_system(simulate)
        .register_type::<Config>()
        .run();
}

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Config {
    scale: f32,
    #[inspector(min = 0, max = 100)]
    sub_steps: u32,
    gravity: Vec2,
    restitution: f32,
    number_balls: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scale: 50.,
            sub_steps: 5,
            gravity: Vec2::new(0., 0.),
            restitution: 1.0,
            number_balls: 20,
        }
    }
}

#[derive(Component)]
struct Keep;

#[derive(Component)]
struct Ball(pub f32);

#[derive(Reflect, Component, Default, Deref, DerefMut)]
#[reflect(Component)]
struct Velocity(pub Vec2);

#[derive(Reflect, Component, Default, Deref, DerefMut)]
#[reflect(Component)]
struct Mass(pub f32);

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0., 0., 100.),
            ..Default::default()
        },
        Keep,
    ));

    info!("Press 'R' to reset");
}

fn spawn_balls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<Config>,
    mut windows: ResMut<Windows>,
) {
    let window = windows.get_primary_mut().unwrap();
    let bounds = Vec2::new(window.width(), window.height());
    // Ball
    
    for _ in 0..config.number_balls {
        let pos = Vec2::new(fastrand::f32() * bounds.x, fastrand::f32() * bounds.y) - bounds * 0.5;
        let radius = 10.0 + fastrand::f32() * config.scale;
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::Circle::new(radius).into())
                    .into(),
                material: materials.add(ColorMaterial::from(Color::RED)),
                transform: Transform::from_xyz(pos.x, pos.y, 0.),
                ..default()
            },
            Mass(PI * radius * radius),
            Velocity(Vec2::new(
                -1. + 2.0 * fastrand::f32() * config.scale * 3.,
                -1. + 2.0 * fastrand::f32() * config.scale  * 3.,
            )),
            Ball(radius),
            Name::new("Ball"),
        ));
    }
}

fn simulate(
    mut query: Query<(&mut Transform, &mut Velocity, &Mass, &Ball)>,
    mut windows: ResMut<Windows>,
    time: Res<Time>,
    config: Res<Config>,
) {
    let window = windows.get_primary_mut().unwrap();
    let bounds = Vec2::new(window.width(), window.height());
    let half_bounds = bounds * 0.5;

    let sdt = time.delta_seconds() / config.sub_steps as f32;

    for (mut trans, mut velocity, _mass, _ball) in query.iter_mut() {
        // sub steps
        for _ in 0..config.sub_steps {
            velocity.0 += config.gravity * sdt;
            trans.translation += (velocity.0 * sdt).extend(0.);
        }
    }

    // Look for collisions
    let mut combinations = query.iter_combinations_mut();
    while let Some([ball_a, ball_b]) = combinations.fetch_next() {
        handle_ball_collision(ball_a, ball_b, &config)
    }

    // keep ball in bounds   
    for (trans, velocity, _mass, ball) in query.iter_mut() {
        handle_wall_collisions(trans, half_bounds - ball.0, velocity);
    }
}

fn handle_wall_collisions(mut trans: Mut<Transform>, half_limit: Vec2, mut velocity: Mut<Velocity>) {
    if trans.translation.x < -half_limit.x {
        trans.translation.x = -half_limit.x;
        velocity.0.x = -velocity.0.x;
    }

    if trans.translation.x > half_limit.x {
        trans.translation.x = half_limit.x;
        velocity.0.x = -velocity.0.x;
    }

    if trans.translation.y < -half_limit.y {
        trans.translation.y = -half_limit.y;
        velocity.0.y = -velocity.0.y;
    }

    if trans.translation.y > half_limit.y {
        trans.translation.y = half_limit.y;
        velocity.0.y = -velocity.0.y;
    }
}

fn handle_ball_collision( 
    mut ball_a: (Mut<Transform>, Mut<Velocity>, &Mass, &Ball),
    mut ball_b: (Mut<Transform>, Mut<Velocity>, &Mass, &Ball),
    config: &Config,
) {
    let dir3 = ball_b.0.translation - ball_a.0.translation;
    let mut dir = Vec2::new(dir3.x, dir3.y);
    let d = dir.length();
    if d == 0.0 || d > ball_a.3.0 + ball_b.3.0 {
        return;
    }

    dir = dir.normalize_or_zero();
    let corr = (ball_a.3.0 + ball_b.3.0 - d) * 0.5;
    ball_a.0.translation += (dir * -corr).extend(0.);
    ball_b.0.translation += (dir * corr).extend(0.);

    let v1 = ball_a.1.0.dot(dir);
    let v2 = ball_b.1.0.dot(dir);

    let m1 = ball_a.2.0;
    let m2 = ball_b.2.0;

    let new_v1 = ( m1 * v1 + m2 * v2 - m2 * (v1 - v2)  * config.restitution ) / (m1 + m2);
    let new_v2 = ( m1 * v1 + m2 * v2 - m1 * (v2 - v1)  * config.restitution ) / (m1 + m2);

    ball_a.1.0 += dir * (new_v1 - v1);
    ball_b.1.0 += dir * (new_v2 - v2);
}

fn reset(
    mut commands: Commands,
    query: Query<Entity, Without<Keep>>,
    mut app_state: ResMut<State<ResetState>>,
) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
    app_state.set(ResetState::Playing).unwrap();
}

pub fn reset_listen(
    mut keys: ResMut<Input<KeyCode>>,
    mut app_state: ResMut<State<ResetState>>
) {
    if keys.just_pressed(KeyCode::R) {
        if app_state.current() == &ResetState::Reset {
            return;
        }
        app_state.set(ResetState::Reset).unwrap();
        keys.reset(KeyCode::R);
    }
}
