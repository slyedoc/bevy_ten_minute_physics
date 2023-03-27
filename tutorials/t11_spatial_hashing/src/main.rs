mod camera_controller;
mod components;
mod reset;
mod resources;
mod text_overlay;

use camera_controller::*;
use components::*;
use reset::*;
use resources::*;
use text_overlay::*;

#[cfg(feature = "debug")]
use bevy::utils::Instant;
use bevy::{pbr::NotShadowCaster, prelude::*};
#[allow(unused_imports)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::f32::consts::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugin(WorldInspectorPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugin(ResetPlugin)
        .add_plugin(CameraControllerPlugin)
        .add_plugin(TextOverlayPlugin)
        .init_resource::<Config>()
        .init_resource::<BallAssets>()
        .add_startup_system(setup)
        .add_system(simulate_ball.in_set(OnUpdate(ResetState::Playing)))
        .add_system(spawn_balls.in_schedule(OnEnter(ResetState::Playing)))
        .register_type::<Config>()
        .register_type::<Ball>()
        .register_type::<Velocity>()
        .run();
}

const BALL_RADIUS: f32 = 0.1;
const COUNT: usize = 24;
const BALL_COUNT: usize = COUNT * COUNT * COUNT;
const WORLD_BOUNDS_MIN: Vec3 = Vec3::new(0., 0., 0.);
const WORLD_BOUNDS_MAX: Vec3 = Vec3::splat(10.0);

#[derive(Reflect, Resource)]
#[reflect(Resource)]
struct BallAssets {
    mesh: Handle<Mesh>,
    red: Handle<StandardMaterial>,
    yellow: Handle<StandardMaterial>,
}

impl FromWorld for BallAssets {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        let red = materials.add(Color::RED.into());
        let yellow = materials.add(Color::YELLOW.into());

        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let mesh = meshes.add(Mesh::from(shape::UVSphere {
            radius: BALL_RADIUS,
            sectors: 8,
            stacks: 8,
        }));

        BallAssets { mesh, red, yellow }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0., 4., 25.),
            ..Default::default()
        },
        CameraController::default(),
        Keep,
    ));

    // light
    commands.spawn((
        DirectionalLightBundle {
            transform: Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Keep,
    ));

    // ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 40.,
                ..default()
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::DARK_GRAY,
                ..default()
            }),
            transform: Transform {
                rotation: Quat::from_rotation_y(FRAC_PI_2),
                ..default()
            },
            ..default()
        },
        Name::new("Ground"),
        Keep,
    ));

    // NOTE: for small size increase max number
    commands.insert_resource(SpatialHash::new(2.0 * BALL_RADIUS, BALL_COUNT));
    info!("Press 'R' to reset");
    info!("Press 'Space' to pause");
}

fn spawn_balls(mut commands: Commands, ball_assets: Res<BallAssets>) {
    let padding = f32::mul_add(BALL_RADIUS * 2., 2.0, 0.0);
    for x in 0..COUNT {
        for y in 0..COUNT {
            for z in 0..COUNT {
                let pos = Vec3::new(x as f32 * padding, y as f32 * padding, z as f32 * padding);
                commands.spawn((
                    PbrBundle {
                        mesh: ball_assets.mesh.clone(),
                        material: ball_assets.red.clone(),
                        transform: Transform::from_translation(pos),
                        ..default()
                    },
                    Velocity(Vec3::new(
                        fastrand::f32() - 0.5,
                        fastrand::f32() - 0.5,
                        fastrand::f32() - 0.5,
                    )),
                    Ball {
                        radius: BALL_RADIUS,
                        prev_pos: pos,
                    },
                    NotShadowCaster,
                    Name::new("Ball"),
                ));
            }
        }
    }
}

fn simulate_ball(
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut Velocity,
        &mut Ball,
        &mut Handle<StandardMaterial>,
    )>,
    time: Res<Time>,
    config: Res<Config>,
    mut hash: ResMut<SpatialHash>,
    ball_assets: Res<BallAssets>,
) {
    #[cfg(feature = "debug")]
    let t0 = Instant::now();
    let dt = time.delta_seconds();

    // zero time blows up
    if dt == 0. {
        return;
    }

    let min_dist = 2.0 * BALL_RADIUS;
    let min_dist_sq = min_dist * min_dist;

    // integrate
    for (_e, mut trans, mut velocity, mut ball, mut mat) in query.iter_mut() {
        velocity.0 += config.gravity * dt * config.scale;
        ball.prev_pos = trans.translation;
        trans.translation += velocity.0 * dt * config.scale;
        *mat = ball_assets.red.clone();
    }

    #[cfg(feature = "debug")]
    let t1 = Instant::now();

    let pos = query
        .iter()
        .map(|(_, trans, _, _, _)| trans.translation)
        .collect::<Vec<_>>();
    let entities = query.iter().map(|(e, _, _, _, _)| e).collect::<Vec<_>>();
    #[cfg(feature = "debug")]
    let t2 = Instant::now();
    hash.create(&pos);
    #[cfg(feature = "debug")]
    let t3 = Instant::now();

    unsafe {
        for i in 0..query.iter().len() {
            let (_e, mut trans, mut velocity, ball, mut mat) =
                query.get_unchecked(entities[i]).unwrap();

            // world collision
            for dim in 0..3 {
                if trans.translation[dim] < WORLD_BOUNDS_MIN[dim] + ball.radius {
                    trans.translation[dim] = WORLD_BOUNDS_MIN[dim] + ball.radius;
                    velocity.0[dim] = -velocity.0[dim];
                    *mat = ball_assets.yellow.clone();
                } else if trans.translation[dim] > WORLD_BOUNDS_MAX[dim] - ball.radius {
                    trans.translation[dim] = WORLD_BOUNDS_MAX[dim] - ball.radius;
                    velocity.0[dim] = -velocity.0[dim];
                    *mat = ball_assets.yellow.clone();
                }
            }

            // interball collision
            hash.query(&pos, i, 2.0 * BALL_RADIUS);

            for nr in 0..hash.query_size {
                let j = hash.query_ids[nr];
                let je = entities[j];

                let (_, mut trans_j, mut vel_j, _ball_j, mut mat_j) =
                    query.get_unchecked(je).unwrap();

                let mut normal = trans.translation - trans_j.translation;
                let d2 = normal.length_squared();

                // are the balls overlapping?
                if d2 > 0.0 && d2 < min_dist_sq {
                    let d = d2.sqrt();
                    normal /= d;

                    // separate the balls

                    let corr = (min_dist - d) * 0.5;

                    trans.translation += normal * corr;
                    trans_j.translation += normal * -corr;

                    // reflect velocities along normal

                    let vi = velocity.0.dot(normal);
                    let vj = vel_j.0.dot(normal);

                    velocity.0 += normal * (vj - vi);
                    vel_j.0 += normal * (vi - vj);

                    *mat = ball_assets.yellow.clone();
                    *mat_j = ball_assets.yellow.clone();
                }
            }
        }
        #[cfg(feature = "debug")]
        let t4 = Instant::now();

        #[cfg(feature = "debug")]
        info!(
            "integrate: {:?}, pos: {:?} hash: {:?} query: {:?} total: {:?}",
            t1 - t0,
            t2 - t1,
            t3 - t2,
            t4 - t3,
            t4 - t0
        );
    }
}
