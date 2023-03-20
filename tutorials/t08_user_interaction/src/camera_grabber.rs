use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use std::f32::consts::FRAC_PI_2;

use crate::{
    components::{Ball, Velocity},
    intersect::ray_sphere_intersect,
};

pub struct CameraGrabberPlugin;

impl Plugin for CameraGrabberPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Grabbed>()
            .add_plugin(ResourceInspectorPlugin::<Grabbed>::default())
            .add_state::<GrabState>()
            .add_system(update_camera_controller.in_set(OnUpdate(GrabState::None)))
            .add_system(handle_grab_none.in_set(OnUpdate(GrabState::None)))
            .add_system(handle_grab_start.in_schedule(OnEnter(GrabState::Moving)))
            .add_system(handle_grab_move.in_set(OnUpdate(GrabState::Moving)))
            .add_system(handle_grab_end.in_schedule(OnExit(GrabState::Moving)))
            .register_type::<CameraGrabber>();
    }
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct Grabbed {
    pub entity: Option<Entity>,
    pub mouse_grab: MouseButton,
    pub distance: f32,
    pub time: f32,
    pub prev_pos: Vec3,
    pub offset: Vec3,
}

impl Default for Grabbed {
    fn default() -> Self {
        Self {
            entity: None,
            mouse_grab: MouseButton::Left,
            distance: 0.,
            time: 0.,
            prev_pos: Vec3::ZERO,
            offset: Vec3::ZERO,
        }
    }
}

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum GrabState {
    #[default]
    None,
    Moving,
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct CameraGrabber {
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    pub mouse_look: MouseButton,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
}

impl Default for CameraGrabber {
    fn default() -> Self {
        Self {
            sensitivity: 0.2,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::E,
            key_down: KeyCode::Q,
            key_run: KeyCode::LShift,
            mouse_look: MouseButton::Right,
            walk_speed: 10.0,
            run_speed: 30.0,
            friction: 0.3,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
        }
    }
}

fn handle_grab_none(
    grabbed: Res<Grabbed>,
    mouse_input: Res<Input<MouseButton>>,
    mut grab_next_state: ResMut<NextState<GrabState>>,
) {
    if mouse_input.just_pressed(grabbed.mouse_grab) {
        grab_next_state.set(GrabState::Moving);
    }
}

fn handle_grab_start(
    mut grabbed: ResMut<Grabbed>,
    window_query: Query<&Window>,
    camera_query: Query<(&GlobalTransform, &Camera), With<CameraGrabber>>,
    mut grab_next_state: ResMut<NextState<GrabState>>,
    mut query_balls: Query<(Entity, &Transform, &mut Velocity, &Ball)>,
) {
    grabbed.time = 0.;

    // Raycast to find the entity to grab
    let window = window_query.single();
    let (camera_trans, camera) = camera_query.single();
    if let Some(cusor_pos) = window.cursor_position() {
        let ray = camera.viewport_to_world(camera_trans, cusor_pos).unwrap();

        // Bevy Mod Picker is not updated for 0.10 yet, doing our own raycast
        let mut closest = std::f32::MAX;
        let mut closest_entity = None;
        let mut closest_offset = Vec3::ZERO;
        let mut closest_pos = Vec3::ZERO;
        for (e, trans, _vel, ball) in query_balls.iter() {
            if let Some((t0, t1)) =
                ray_sphere_intersect(ray.origin, ray.direction, trans.translation, ball.0)
            {
                let t = t0.min(t1);
                
                if t < closest {
                    closest_entity = Some(e);                    
                    closest = t;
                    closest_pos = ray.origin + (ray.direction * closest);
                    closest_offset = trans.translation - closest_pos;
                }
            }
        }

        if closest_entity.is_some() {
            grabbed.entity = closest_entity;
            grabbed.distance = closest;
            grabbed.prev_pos = closest_pos;
            grabbed.offset = closest_offset;
            // set velocity to zero
            query_balls.get_mut(grabbed.entity.unwrap()).unwrap().2 .0 = Vec3::ZERO;
        } else {
            grabbed.entity = None;
            grabbed.time = 0.;
            grab_next_state.set(GrabState::None);
        }
    } else {
        // If we can't get the cursor position, we can't grab anything
        grab_next_state.set(GrabState::None);
    }
}

fn handle_grab_move(
    mouse_input: Res<Input<MouseButton>>,
    mut grabbed: ResMut<Grabbed>,
    mut grab_next_state: ResMut<NextState<GrabState>>,
    time: Res<Time>,
    mut query_balls: Query<(&mut Transform, &mut Velocity), With<Ball>>,
    window_query: Query<&Window>,
    camera_query: Query<(&GlobalTransform, &Camera), With<CameraGrabber>>,
) {
    if mouse_input.just_released(grabbed.mouse_grab) || grabbed.entity.is_none() {
        grab_next_state.set(GrabState::None);
        return;
    }

    grabbed.time += time.delta_seconds();

    let window = window_query.single();
    let (camera_trans, camera) = camera_query.single();
    if let Some(cusor_pos) = window.cursor_position() {
        let ray = camera.viewport_to_world(camera_trans, cusor_pos).unwrap();
        if let Ok((mut trans, mut vel)) = query_balls.get_mut(grabbed.entity.unwrap()) {
            let pos = ray.origin + (ray.direction * grabbed.distance);
            vel.0 = pos - grabbed.prev_pos;
            if grabbed.time > 0. {
                vel.0 /= grabbed.time;
            } else {
                vel.0 = Vec3::ZERO;
            }
            grabbed.prev_pos = pos;
            grabbed.time = 0.0;
            trans.translation = pos + grabbed.offset;
        }
    }
}

fn handle_grab_end(mut grabbed: ResMut<Grabbed>) {
    grabbed.entity = None;
}

fn update_camera_controller(
    time: Res<Time>,
    mut mouse_motion: EventReader<MouseMotion>,
    key_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut query: Query<(&mut Transform, &mut CameraGrabber)>,
    mut window_query: Query<&mut Window>,
) {
    let dt = time.delta_seconds();
    let mut window = window_query.single_mut();

    for (mut transform, mut controller) in query.iter_mut() {
        // Handle look mode
        // Handle key input
        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(controller.key_forward) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(controller.key_back) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(controller.key_right) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(controller.key_left) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(controller.key_up) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(controller.key_down) {
            axis_input.y -= 1.0;
        }

        // Apply movement update
        if axis_input != Vec3::ZERO {
            let max_speed = if key_input.pressed(controller.key_run) {
                controller.run_speed
            } else {
                controller.walk_speed
            };
            controller.velocity = axis_input.normalize() * max_speed;
        } else {
            let friction = controller.friction.clamp(0.0, 1.0);
            controller.velocity *= 1.0 - friction;
            if controller.velocity.length_squared() < 1e-6 {
                controller.velocity = Vec3::ZERO;
            }
        }
        let forward = transform.forward();
        let right = transform.right();
        transform.translation += controller.velocity.x * dt * right
            + controller.velocity.y * dt * Vec3::Y
            + controller.velocity.z * dt * forward;

        // Handle mouse look on mouse button
        let mut mouse_delta = Vec2::ZERO;
        if mouse_input.pressed(controller.mouse_look) {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        if mouse_input.just_released(controller.mouse_look) {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
        if mouse_input.pressed(controller.mouse_look) {
            for mouse_event in mouse_motion.iter() {
                mouse_delta += mouse_event.delta;
            }
        }

        if mouse_delta != Vec2::ZERO {
            let (mut yaw, mut pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
            yaw -= mouse_delta.x * controller.sensitivity * time.delta_seconds();
            pitch -= mouse_delta.y * controller.sensitivity * time.delta_seconds();

            let pitch = pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
            transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0)
        }
    }
}
