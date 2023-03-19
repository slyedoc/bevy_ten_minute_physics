use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::default())
        .insert_resource(ClearColor(Color::WHITE))
        .init_resource::<Config>()                
        .add_startup_system(setup)
        .add_system(simulate)
        .register_type::<Config>()
        .register_type::<Velocity>()
        .run();
}

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Config {
    scale: f32,
    #[inspector(min = 0, max = 100)]
    sub_steps: u32,
    gravity: Vec2,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scale: 5.,
            sub_steps: 5,
            gravity: Vec2::new(0., -9.81),
        }
    }
}

#[derive(Component)]
struct Ball(pub f32);

#[derive(Reflect, Component, Default, Deref, DerefMut)]
#[reflect(Component)]
struct Velocity(pub Vec2);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<Config>,
) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 100.),
        ..Default::default()
    });

    // Ball
    let size = 5.0;
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Circle::new(size * config.scale).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Velocity(Vec2::new(-20., 10.) * config.scale),
        Ball(size),
        Name::new("Ball"),
    ));
}

fn simulate(
    mut query: Query<(&mut Transform, &mut Velocity, &Ball)>,
    window_query: Query<&Window>,
    time: Res<Time>,
    config: Res<Config>,
) {
    let window = window_query.single();
    let sdt = time.delta_seconds() / config.sub_steps as f32;

    for (mut trans, mut velocity, ball) in query.iter_mut() {
        // sub steps
        for _ in 0..config.sub_steps {
            velocity.0 += config.gravity * sdt * config.scale;
            trans.translation += (velocity.0 * sdt * config.scale).extend(0.);
        }

        // keep ball in bounds
        let limit = Vec2::new(
            window.width() * 0.5 - (ball.0 * config.scale),
            window.height() * 0.5 - (ball.0 * config.scale),
        );
        if trans.translation.x < -limit.x {
            trans.translation.x = -limit.x;
            velocity.0.x = -velocity.0.x;
        }

        if trans.translation.x > limit.x {
            trans.translation.x = limit.x;
            velocity.0.x = -velocity.0.x;
        }

        if trans.translation.y < -limit.y {
            trans.translation.y = -limit.y;
            velocity.0.y = -velocity.0.y;
        }

        if trans.translation.y > limit.y {
            trans.translation.y = limit.y;
            velocity.0.y = -velocity.0.y;
        }
    }
}
