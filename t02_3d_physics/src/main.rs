
use bevy::prelude::*;
use bevy_inspector_egui::quick::{WorldInspectorPlugin};
use bevy_inspector_egui::prelude::*;
use sly_camera_controller::{CameraController, CameraControllerPlugin};

fn main() {
    App::new()    
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin)    
        .add_plugin(CameraControllerPlugin)            
        .init_resource::<Config>()
        .register_type::<Config>()
        .register_type::<Velocity>()
        .add_startup_system(setup)
        .add_system(simulate)
        .register_type::<Config>()
        .run();
}


#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Config {
    scale: f32,
    #[inspector(min = 10., max = 100.)]
    half_size: f32,
    #[inspector(min = 0, max = 100)]
    sub_steps: u32,
    gravity: Vec3,
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

#[derive(Component)]
struct Ball(pub f32);

#[derive(Reflect, Component, Default, Deref, DerefMut)]
#[reflect(Component)]
struct Velocity(pub Vec3);


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<Config>,
) {

    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0., 4., 25.),
        ..Default::default()
    },
    CameraController::default()
));

    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    // ground
    
    commands.spawn((        
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: config.half_size * 2.,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::DARK_GREEN,
                ..default()
            }),
            transform: Transform {
                rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                ..default()
            },
            ..default()
        },
        Name::new("Ground"),
    ));

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
        let offset = ball.0;
        let limit = Vec3::new(config.half_size - ball.0, config.half_size - ball.0, config.half_size - ball.0);    
        if trans.translation.x < -limit.x {            
            trans.translation.x = -limit.x;
            velocity.0.x = -velocity.0.x;
        }

        if trans.translation.x > limit.x {
            trans.translation.x = limit.x;
            velocity.0.x = -velocity.0.x;
        }

        if trans.translation.y < ball.0 {
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