mod components;
mod reset;
mod ui;

use std::f32::consts::PI;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_inspector_egui::prelude::*;
//use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use components::*;
use reset::*;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugin(WorldInspectorPlugin)
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(ResetPlugin)
        .add_plugin(UiPlugin)
        .init_resource::<Config>()
        .init_resource::<Border>()
        .init_resource::<Score>()
        //.add_plugin(ResourceInspectorPlugin::<Config>::default())
        .insert_resource(ClearColor(Color::WHITE))
        .add_startup_system(setup)
        .add_system(spawn_balls.in_schedule(OnEnter(ResetState::Playing)))
        .add_system(flipper_simulate.before(simulate))
        .add_system(simulate)
        .add_system(draw_boarder)
        .add_system(spawn_flipper)
        .register_type::<Config>()
        .register_type::<Score>()
        .register_type::<Mass>()
        .register_type::<Velocity>()
        .register_type::<Border>()
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
            gravity: Vec2::new(0., -9.8),
            restitution: 1.0,
            number_balls: 20,
        }
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Score(pub u32);

fn scale_vec2(pos: Vec2, scale: f32) -> Vec2 {
    (pos + Vec2::splat(-0.5)) * scale
}

fn setup(
    mut commands: Commands,
    window_query: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut border: ResMut<Border>,
) {
    // Setup Camera
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0., 0., 100.),
            ..Default::default()
        },
        Keep,
    ));

    // Setup Board based on window size
    let window = window_query.single();
    let scale = window.width().min(window.height());

    let offset = Vec2::new(0.25, 0.10);
    let top_height = 1.0 - 0.68;
    let pit_height = 0.15 + offset.y;
    let pit_width_inset = offset.x + 0.12;

    // setup border
    border.push(scale_vec2(
        Vec2::new(1.0 - pit_width_inset, pit_height),
        scale,
    )); // pit top right
    border.push(scale_vec2(Vec2::new(1.0 - offset.x, top_height), scale));
    border.push(scale_vec2(Vec2::new(1.0 - offset.x, 1.0 - offset.y), scale)); // top right
    border.push(scale_vec2(Vec2::new(offset.x, 1.0 - offset.y), scale)); // top left
    border.push(scale_vec2(Vec2::new(offset.x, top_height), scale));
    border.push(scale_vec2(Vec2::new(pit_width_inset, pit_height), scale)); // pit top left
    border.push(scale_vec2(Vec2::new(pit_width_inset, offset.y), scale)); // pit bottom left
    border.push(scale_vec2(
        Vec2::new(1.0 - pit_width_inset, offset.y),
        scale,
    )); // pit bottom right

    // setup obstacles
    for (size, position) in [
        (0.055, Vec2::new(0.35, 0.72)),
        (0.07, Vec2::new(0.6, 0.62)),
        (0.055, Vec2::new(0.37, 0.42)),
        (0.055, Vec2::new(0.65, 0.39)),
    ]
    .iter()
    {
        let radius = *size * scale;
        let pos = scale_vec2(*position, scale).extend(0.);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(radius).into()).into(),
                material: materials.add(ColorMaterial::from(Color::ORANGE)),
                transform: Transform::from_translation(pos),
                ..Default::default()
            },
            Obstacle {
                radius,
                push_velocity: 200.0,
            },
            Keep,
            Name::new("Bumper"),
        ));
    }

    // setup flippers
    let flipper_radius = 0.015;
    let flipper_length = 0.10;
    let flipper_max_rotaiton = 1.0;
    let flipper_rest_angle = PI * 1.33;
    let angular_velocity = 10.0;

    for (position, rest_angle, max_rotation, sign, key) in [
        (
            Vec2::new(pit_width_inset, pit_height - flipper_radius),
            flipper_rest_angle,
            flipper_max_rotaiton,
            1.0,
            KeyCode::LShift,
        ),
        (
            Vec2::new(1.0 - pit_width_inset, pit_height - flipper_radius),
            -flipper_rest_angle,
            -flipper_max_rotaiton,
            -1.0,
            KeyCode::RShift,
        ),
    ]
    .iter()
    {
        let pos = scale_vec2(*position, scale).extend(0.);
        commands.spawn((
            TransformBundle {
                local: Transform {
                    translation: pos,
                    rotation: Quat::from_rotation_z(*rest_angle),
                    ..Default::default()
                },
                ..Default::default()
            },
            VisibilityBundle::default(),
            Flipper::new(
                flipper_radius * scale,
                flipper_length * scale,
                *max_rotation,
                *rest_angle,
                angular_velocity,
                *sign,
                *key,
            ),
            Restitution(0.0),
            Keep,
            Name::new("Flipper"),
        ));
    }

    info!("Press 'R' to reset");
    info!("Press 'LShift' and `RShift` to control the flippers");
}

fn spawn_flipper(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Flipper), Added<Flipper>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let flipper_color = Color::RED;
    for (e, flipper) in query.iter_mut() {
        commands.entity(e).with_children(|parent| {
            parent.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(flipper.radius).into()).into(),
                    material: materials.add(ColorMaterial::from(flipper_color)),
                    ..Default::default()
                },
                Name::new("Base"),
            ));
            parent.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(
                            shape::Quad::new(Vec2::new(flipper.radius * 2., flipper.length)).into(),
                        )
                        .into(),
                    material: materials.add(ColorMaterial::from(flipper_color)),
                    transform: Transform::from_xyz(0.0, flipper.length * 0.5, 0.),
                    ..Default::default()
                },
                Name::new("Length"),
            ));

            parent.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(flipper.radius).into()).into(),
                    material: materials.add(ColorMaterial::from(flipper_color)),
                    transform: Transform::from_xyz(0., flipper.length, 0.),
                    ..Default::default()
                },
                Name::new("Tip"),
            ));
        });
    }
}

fn flipper_simulate(
    mut query: Query<(&mut Flipper, &mut Transform)>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut flipper, mut trans) in query.iter_mut() {
        let mut rotation = trans.rotation.to_euler(EulerRot::YXZ).2;

        flipper.prev_rotation = rotation;

        if keyboard_input.pressed(flipper.key) {
            rotation += dt * flipper.angular_velocity * flipper.sign;
        } else {
            rotation -= dt * flipper.angular_velocity * flipper.sign;
        }
        rotation = rotation.clamp(
            flipper.rest_angle.min(flipper.max_rotation),
            flipper.rest_angle.max(flipper.max_rotation),
        );
        flipper.current_angular_velocity = (rotation - flipper.prev_rotation) / dt;
        trans.rotation = Quat::from_axis_angle(Vec3::Z, rotation);
    }
}

fn draw_boarder(mut lines: ResMut<DebugLines>, border: ResMut<Border>) {
    // Draw the border
    for index in 0..border.points.len() {
        let start = border.points[index];
        let end = border.points[(index + 1) % border.points.len()];
        lines.line_colored(start, end, 0.0, Color::BLACK);
    }
}

fn spawn_balls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<Config>,
    window_query: Query<&Window>,
) {
    let window = window_query.single();
    let scale = window.width().min(window.height());
    // Ball

    for _ in 0..2 {
        let pos = scale_vec2(Vec2::new(0.3 + (fastrand::f32() * 0.5), 0.7), scale);
        let radius = 0.02 * scale;
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(radius).into()).into(),
                material: materials.add(ColorMaterial::from(Color::BLACK)),
                transform: Transform::from_xyz(pos.x, pos.y, 0.),
                ..default()
            },
            Mass(PI * radius * radius),
            Velocity(Vec2::new(
                -1. + 2.0 * fastrand::f32() * config.scale * 3.,
                -1. + 2.0 * fastrand::f32() * config.scale * 3.,
            )),
            Restitution(0.9),
            Ball { radius },
            Name::new("Ball"),
        ));
    }
}

fn simulate(
    mut balls: Query<
        (&mut Transform, &mut Velocity, &Mass, &Restitution, &Ball),
        (Without<Flipper>, Without<Obstacle>),
    >,
    obstacles: Query<(&Transform, &Obstacle), (Without<Ball>, Without<Flipper>)>,
    mut flipper: Query<(&mut Transform, &Flipper), (Without<Ball>, Without<Obstacle>)>,
    time: Res<Time>,
    config: Res<Config>,
    mut score: ResMut<Score>,
    border: Res<Border>,
) {
    let dt = time.delta_seconds();
    // simulate balls
    for (mut trans, mut velocity, _mass, _res, _ball) in balls.iter_mut() {
        velocity.0 += config.gravity * dt * 20.;
        trans.translation += (velocity.0 * dt).extend(0.);
    }

    // Look for ball ball collisions
    let mut combinations = balls.iter_combinations_mut();
    while let Some(
        [(mut trans_a, mut vel_a, mass_a, rest_a, ball_a), (mut trans_b, mut vel_b, mass_b, rest_b, ball_b)],
    ) = combinations.fetch_next()
    {
        handle_ball_ball_collision(
            &mut trans_a,
            &mut vel_a,
            mass_a,
            rest_a,
            ball_a,
            &mut trans_b,
            &mut vel_b,
            mass_b,
            rest_b,
            ball_b,
        )
    }

    for (mut trans_a, mut vel_a, _mass_a, res_a, ball_a) in balls.iter_mut() {
        for (trans_b, obstacle) in obstacles.iter() {
            handle_ball_obstacle_collision(
                &mut trans_a,
                &mut vel_a,
                ball_a,
                trans_b,
                obstacle,
                &mut score,
            );
        }

        for (mut trans_b, flipper) in flipper.iter_mut() {
            handle_ball_flipper_collision(&mut trans_a, &mut vel_a, ball_a, &mut trans_b, flipper);
        }
        handle_ball_border_collision(&mut trans_a, &mut vel_a, res_a, ball_a, &border);
    }
}

fn handle_ball_ball_collision(
    trans_a: &mut Transform,
    vel_a: &mut Velocity,
    mass_a: &Mass,
    rest_a: &Restitution,
    ball_a: &Ball,
    trans_b: &mut Transform,
    vel_b: &mut Velocity,
    mass_b: &Mass,
    rest_b: &Restitution,
    ball_b: &Ball,
) {
    let restitution = rest_a.min(rest_b.0);
    let mut dir = (trans_b.translation - trans_a.translation).truncate();
    let d = dir.length();
    if d == 0.0 || d > ball_a.radius + ball_b.radius {
        return;
    }

    dir = dir.normalize_or_zero();
    let corr = (ball_a.radius + ball_b.radius - d) * 0.5;
    trans_a.translation += (dir * -corr).extend(0.);
    trans_b.translation += (dir * corr).extend(0.);

    let v1 = vel_a.dot(dir);
    let v2 = vel_b.dot(dir);

    let m1 = mass_a.0;
    let m2 = mass_b.0;

    let new_v1 = (m1 * v1 + m2 * v2 - m2 * (v1 - v2) * restitution) / (m1 + m2);
    let new_v2 = (m1 * v1 + m2 * v2 - m1 * (v2 - v1) * restitution) / (m1 + m2);

    vel_a.0 += dir * (new_v1 - v1);
    vel_b.0 += dir * (new_v2 - v2);
}

fn handle_ball_obstacle_collision(
    trans_a: &mut Transform,
    vel_a: &mut Velocity,
    ball_a: &Ball,
    trans_b: &Transform,
    obstacle: &Obstacle,
    mut score: &mut Score,
) {
    let mut dir = (trans_a.translation - trans_b.translation).truncate();

    let d = dir.length();

    if d == 0.0 || d > ball_a.radius + obstacle.radius {
        return;
    }

    dir = dir.normalize();

    let corr = ball_a.radius + obstacle.radius - d;
    trans_a.translation += (dir * corr).extend(0.);

    let v = vel_a.0.dot(dir);
    vel_a.0 += dir * (obstacle.push_velocity - v);
    score.0 += 1;
}

fn handle_ball_flipper_collision(
    trans_a: &mut Transform,
    vel_a: &mut Velocity,
    ball_a: &Ball,

    trans_b: &mut Transform,
    flipper_b: &Flipper,
) {
    let closest = closest_point_on_segment(
        trans_a.translation,
        trans_b.translation,
        flipper_b.get_tip(&trans_b),
    );
    let mut dir = trans_a.translation - closest;
    let d = dir.length();
    if d == 0.0 || d > ball_a.radius + flipper_b.radius {
        return;
    }

    dir = dir.normalize();

    let corr = ball_a.radius + flipper_b.radius - d;
    trans_a.translation += dir * corr;

    // update velocitiy

    let mut radius = closest.clone();
    radius += dir * flipper_b.radius;
    radius -= trans_b.translation;
    let mut surface_vel = Vec3::new(-radius.y, radius.x, 0.); // perp
    surface_vel *= flipper_b.current_angular_velocity;

    // TODO: this feels wrong, balls stick to the flipper,
    // but it's better than having no control
    let v = vel_a.extend(0.).dot(dir);
    let vnew = surface_vel.dot(dir);
    vel_a.0 += (dir * (vnew - v)).truncate();
}

fn handle_ball_border_collision(
    trans_a: &mut Transform,
    vel_a: &mut Velocity,
    rest_a: &Restitution,
    ball_a: &Ball,
    border: &Border,
) {
    if border.points.len() < 3 {
        return;
    }
    // find closest segment;

    #[allow(unused_assignments)]
    let mut d = Vec3::ZERO;
    let mut closest = Vec3::ZERO;
    let mut normal = Vec3::ZERO;

    let mut min_dist = 0.0;

    for index in 0..border.points.len() {
        let a = border.points[index];
        let b = border.points[(index + 1) % border.points.len()];
        let c = closest_point_on_segment(trans_a.translation, a, b);
        d = trans_a.translation - c;
        let dist = d.length();
        if index == 0 || dist < min_dist {
            min_dist = dist;
            closest = c;
            let ab = b - a;
            normal = Vec3::new(-ab.y, ab.x, 0.0); // 2d only
        }
    }

    // push out
    d = trans_a.translation - closest;
    let mut dist = d.length();

    if dist == 0.0 {
        d = normal;
        dist = normal.length();
    }

    d *= dist.recip();

    if d.dot(normal) >= 0.0 {
        if dist > ball_a.radius {
            return;
        }
        trans_a.translation += d * (ball_a.radius - dist);
    } else {
        trans_a.translation += d * -(dist + ball_a.radius);
    }

    // update velocity
    let v = vel_a.extend(0.).dot(d);
    let vnew = v.abs() * rest_a.0;

    vel_a.0 += (d * (vnew - v)).truncate();
}

fn closest_point_on_segment(p: Vec3, a: Vec3, b: Vec3) -> Vec3 {
    let ab = b - a;

    let mut t = ab.dot(ab);
    if t == 0.0 {
        return a;
    }

    t = 0.0f32.max(1.0f32.min((p.dot(ab) - a.dot(ab)) / t));
    let closest = a;
    return closest + (ab * t);
}
