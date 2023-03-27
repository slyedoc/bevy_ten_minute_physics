use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use std::f32::consts::FRAC_PI_2;

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(update_camera_controller)
            .register_type::<CameraController>();
    }
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct CameraController {
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

impl Default for CameraController {
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

fn update_camera_controller(
    time: Res<Time>,
    mut mouse_motion: EventReader<MouseMotion>,
    key_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut query: Query<(&mut Transform, &mut CameraController)>,
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
