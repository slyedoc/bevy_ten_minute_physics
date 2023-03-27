mod components;
mod reset;
mod resources;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use components::*;
use reset::*;
use resources::*;
use std::f32::consts::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(ResetPlugin)
        .init_resource::<Config>()
        .add_startup_system(setup)
        .add_system(simulate)
        .add_system(spawn_ball.in_schedule(OnEnter(ResetState::Playing)))
        .register_type::<Config>()
        .register_type::<Ball>()
        .register_type::<Velocity>()
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<Config>,
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0., 4., 25.),
            ..Default::default()
        },
        Keep,
    ));

    // light
    commands.spawn((
        DirectionalLightBundle {
            transform: Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..default()
            },

            ..default()
        },
        Keep,
    ));

    // ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: config.half_size * 2.,
                ..default()
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::DARK_GREEN,
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

    info!("Press 'R' to reset");
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<Config>,
) {
    // Ball
    let size = 1.0;
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: size,
                sectors: 32,
                stacks: 10,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                ..default()
            }),
            transform: Transform::from_xyz(0., 2., 0.),
            ..default()
        },
        Velocity(Vec3::new(-3., 8., -6.) * config.scale),
        Ball(size),
        Name::new("Ball"),
    ));
}

fn simulate(
    mut query: Query<(&mut Transform, &mut Velocity, &Ball)>,
    time: Res<Time>,
    config: Res<Config>,
) {
    let sdt = time.delta_seconds() / config.sub_steps as f32;

    for (mut trans, mut velocity, ball) in query.iter_mut() {
        // sub steps
        for _ in 0..config.sub_steps {
            velocity.0 += config.gravity * sdt * config.scale;
            trans.translation += velocity.0 * sdt * config.scale
        }

        // keep ball in bounds
        let limit = Vec3::new(
            config.half_size - ball.0,
            config.half_size - ball.0,
            config.half_size - ball.0,
        );
        if trans.translation.x < -limit.x {
            trans.translation.x = -limit.x;
            velocity.0.x = -velocity.0.x;
        }

        if trans.translation.x > limit.x {
            trans.translation.x = limit.x;
            velocity.0.x = -velocity.0.x;
        }

        if trans.translation.y < ball.0 {
            // limit to ground + ball radius
            trans.translation.y = ball.0;
            velocity.0.y = -velocity.0.y;
        }

        if trans.translation.y > limit.y {
            trans.translation.y = limit.y;
            velocity.0.y = -velocity.0.y;
        }

        if trans.translation.z < -limit.z {
            trans.translation.z = -limit.z;
            velocity.0.z = -velocity.0.z;
        }

        if trans.translation.z > limit.z {
            trans.translation.z = limit.z;
            velocity.0.z = -velocity.0.z;
        }
    }
}
