mod components;
mod reset;
mod resources;

use std::f32::consts::PI;

use components::*;
use reset::*;
use resources::*;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<Config>()
        .init_resource::<Pendulms>()
        .add_plugin(ResourceInspectorPlugin::<Config>::default())
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(ResetPlugin)
        .add_startup_system(setup)
        .add_system(spawn_pendulum.in_schedule(OnEnter(ResetState::Playing)))
        .add_systems(
            (simulate, draw_lengths)
                .chain()
                .in_set(OnUpdate(ResetState::Playing)),
        )
        .register_type::<Config>()
        .run()
}

fn setup(mut commands: Commands) {
    // Setup Camera
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0., 0., 100.),
            projection: OrthographicProjection {
                scale: 0.01,
                ..default()
            },
            ..Default::default()
        },
        Keep,
    ));

    info!("Press 'R' to reset");
}

fn spawn_pendulum(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut pendulms: ResMut<Pendulms>,
) {
    let mut pos = Transform::default();
    let mut segements = Vec::new();
    for (index, (length, mass, angle)) in [
        (0.5f32, 1.0f32, -PI * 0.5),
        (1.0, 0.5, PI * 0.5),
        (2.0, 0.1, 0.0),
    ]
    .iter()
    .enumerate()
    {
        let radius = 0.05 + mass.sqrt() * 0.3;
        pos.rotate_local_z(*angle);
        pos.translation = pos.transform_point(Vec3::new(0.0, *length, 0.));
        let id = commands
            .spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(radius).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::RED)),
                    transform: pos.clone(),
                    ..Default::default()
                },
                PendulmSegment {
                    length: *length,
                    radius,
                    mass: *mass,
                    prev_pos: pos.translation.truncate(),
                    ..default()
                },
                Name::new(format!("Pendulm Segment {}", index)),
            ))
            .id();
        segements.push(id);
    }

    pendulms.list.push(segements);
}

fn draw_lengths(
    mut lines: ResMut<DebugLines>,
    query: Query<&Transform, With<PendulmSegment>>,
    pendulms: Res<Pendulms>,
    mut last_pos: Local<Option<Vec3>>,
) {
    for p in pendulms.list.iter() {
        let mut last = Vec3::ZERO;
        for (index, e) in p.iter().enumerate() {
            let trans = query.get(*e).unwrap();
            lines.line_colored(last, trans.translation, 0.0, Color::GRAY);
            last = trans.translation;

            // Draw trail of last segment
            if index == p.len() - 1 {
                if let Some(pos) = *last_pos {
                    lines.line_colored(last, pos, 1.5, Color::RED);
                }
                *last_pos = Some(last);
            }
        }
    }
}

fn simulate(
    mut query: Query<(&mut PendulmSegment, &mut Transform)>,
    pendulms: Res<Pendulms>,
    config: Res<Config>,
    time: Res<Time>,
) {
    if time.delta_seconds() == 0.0 {
        return;
    }

    let sdt = time.delta_seconds() / config.sub_steps as f32;

    for _ in 0..config.sub_steps {
        for (mut p, mut transform) in query.iter_mut() {
            p.velocity += config.gravity * sdt;
            p.prev_pos = transform.translation.truncate();
            transform.translation += p.velocity.extend(0.0) * sdt;
        }

        let mut p0 = PendulmSegment::default();
        let mut t0 = Transform::default();
        for p in pendulms.list.iter() {
            for (index, e) in p.iter().enumerate() {
                let (p0, t0, p1, t1) = match index {
                    0 => {
                        let (p_t, t_t) = query.get_mut(*e).unwrap();
                        (&mut p0, &mut t0, p_t.into_inner(), t_t.into_inner())
                    }
                    _ => {
                        let [(p0, t0), (p1, p2)] =
                            query.get_many_mut([p[index - 1], p[index]]).unwrap();
                        (
                            p0.into_inner(),
                            t0.into_inner(),
                            p1.into_inner(),
                            p2.into_inner(),
                        )
                    }
                };

                let dist = t1.translation.truncate() - t0.translation.truncate();
                let d = dist.length();

                let w0 = if p0.mass > 0.0 { 1.0 / p0.mass } else { 0.0 };

                let w1 = if p1.mass > 0.0 { 1.0 / p1.mass } else { 0.0 };

                let corr = (p1.length - d) / d / (w0 + w1);

                t0.translation -= (w0 * corr * dist).extend(0.0);
                t1.translation += (w1 * corr * dist).extend(0.0);
            }
        }

        for (mut p, transform) in query.iter_mut() {
            p.velocity = ((transform.translation - p.prev_pos.extend(0.)) / sdt).truncate();
        }
    }
}
