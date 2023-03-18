mod components;
mod reset;
mod resources;

use components::*;
use reset::*;
use resources::*;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::WHITE))
        .init_resource::<Config>()
        .add_plugin(ResourceInspectorPlugin::<Config>::default())
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(ResetPlugin)
        .add_startup_system(setup)
        .add_system(draw_wires)
        .add_system(spawn_beads.in_schedule(OnEnter(ResetState::Playing)))
        .add_system(simulate)
        .register_type::<Wire>()
        .register_type::<Config>()
        .run()
}

fn setup(mut commands: Commands) {
    // Setup Camera
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0., 0., 100.),
            projection: OrthographicProjection {
                scale: 0.5,
                ..default()
            },
            ..Default::default()
        },
        Keep,
    ));

    // Setup Wire
    commands.spawn((
        TransformBundle {
            local: Transform::from_xyz(0., 0., 0.),
            ..Default::default()
        },
        Wire {
            radius: 100.,
            line_segments: 70,
        },
        Name::new("Wire"),
        Keep,
    ));

    info!("Press 'R' to reset");
}

fn spawn_beads(
    mut commands: Commands,
    wires: Query<(&Wire, &Transform)>,
    config: Res<Config>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (wire, trans) in wires.iter() {
        for i in 0..config.bead_count {
            let radius = 10.0 + (fastrand::f32() * 10.0);
            let mut pos =
                Transform::from_translation(trans.translation + Vec3::new(wire.radius, 0., 0.));
            pos.rotate_around(
                Vec3::ZERO,
                Quat::from_rotation_z(2. * PI / config.bead_count as f32 * i as f32),
            );

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(radius).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::RED)),
                    transform: pos,
                    ..Default::default()
                },
                Bead {
                    radius,
                    mass: radius * radius,
                    ..default()
                },
                Name::new(format!("Bead {}", i)),
            ));
        }
    }
}

fn draw_wires(mut lines: ResMut<DebugLines>, query: Query<(&Wire, &Transform)>) {
    for (wire, transform) in query.iter() {
        let mut last = transform.translation + Vec3::new(0.0, wire.radius, 0.);
        let mut trans = transform.clone();
        let rotation = Quat::from_rotation_z(2. * std::f32::consts::PI / wire.line_segments as f32);
        for _ in 0..wire.line_segments {
            trans.rotate(rotation);
            let current = trans.transform_point(Vec3::new(0.0, wire.radius, 0.));
            lines.line_colored(last, current, 0.0, Color::BLACK);
            last = current;
        }
    }
}

fn simulate(
    mut beads: Query<(&mut Bead, &mut Transform), Without<Wire>>,
    wires: Query<(&Wire, &Transform), Without<Bead>>,
    config: Res<Config>,
    time: Res<Time>,
) {
    let sdt = time.delta_seconds() / config.sub_steps as f32;

    for _ in 0..config.sub_steps {
        for (mut bead, mut transform) in beads.iter_mut() {
            bead.start_step(&mut transform, sdt, &config);
        }

        // Note: Assuming only 1 wire, all beads use it
        for (wire, wire_trans) in wires.iter() {
            let wire_center = wire_trans.translation;

            for (mut bead, mut transform) in beads.iter_mut() {
                bead.keep_on_wire(wire_center, wire.radius, &mut transform);
            }
        }
        for (mut bead, mut transform) in beads.iter_mut() {
            bead.end_step(&mut transform, sdt);
        }

        let mut combinations = beads.iter_combinations_mut();
        while let Some([(mut bead_a, mut trans_a), (mut bead_b, mut trans_b)]) =
            combinations.fetch_next()
        {
            handle_bead_bead_collision(
                &mut bead_a,
                &mut trans_a,
                &mut bead_b,
                &mut trans_b,
                &config,
            )
        }
    }
}

fn handle_bead_bead_collision(
    bead_a: &mut Bead,
    trans_a: &mut Transform,
    bead_b: &mut Bead,
    trans_b: &mut Transform,
    config: &Config,
) {
    let mut dir = (trans_b.translation - trans_a.translation).truncate();
    let d = dir.length();
    if d == 0.0 || d > bead_a.radius + bead_b.radius {
        return;
    }

    dir = dir.normalize_or_zero();
    let corr = (bead_a.radius + bead_b.radius - d) * 0.5;
    trans_a.translation += (dir * -corr).extend(0.);
    trans_b.translation += (dir * corr).extend(0.);

    let v1 = bead_a.velocity.dot(dir);
    let v2 = bead_b.velocity.dot(dir);

    let m1 = bead_a.mass;
    let m2 = bead_b.mass;

    let new_v1 = (m1 * v1 + m2 * v2 - m2 * (v1 - v2) * config.restitution) / (m1 + m2);
    let new_v2 = (m1 * v1 + m2 * v2 - m1 * (v2 - v1) * config.restitution) / (m1 + m2);

    bead_a.velocity += dir * (new_v1 - v1);
    bead_b.velocity += dir * (new_v2 - v2);
}
