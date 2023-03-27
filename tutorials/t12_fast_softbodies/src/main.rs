mod camera_grabber;
mod components;
mod intersect;
mod models;
mod resources;
mod state;

use camera_grabber::*;
use components::*;
use models::*;
use resources::*;
use state::*;

use bevy::{pbr::wireframe::Wireframe, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::f32::consts::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(TetMeshPlugin)
        .add_plugin(StatePlugin)
        .add_plugin(CameraGrabberPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<DragonAssets>()
        .init_resource::<Config>()
        .add_startup_system(setup)
        //.add_system(simulate_softbody)
        .add_system(spawn_dragon.in_schedule(OnEnter(AppState::Playing)))
        .register_type::<Config>()
        .register_type::<Ball>()
        .register_type::<Velocity>()
        .run();
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
struct DragonAssets {
    tet_mesh: Handle<TetMesh>,
    material: Handle<StandardMaterial>,
}

impl FromWorld for DragonAssets {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        let material = materials.add(Color::YELLOW.into());

        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let tet_mesh = asset_server.load("model/dragon.tet.json");
        
        DragonAssets { material, tet_mesh }
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
        CameraGrabber::default(),
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
                size: 50.0,
                ..default()
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::GRAY,
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

fn spawn_dragon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dragon_assets: Res<DragonAssets>,
    mut tet_meshes: ResMut<Assets<TetMesh>>,
) {
    info!("Spawning dragon");
    let dragon_tet_mesh = tet_meshes.get_mut(&dragon_assets.tet_mesh).unwrap();
    info!("Dragon found!");
    let mut dragon = SoftBody::new(dragon_tet_mesh, 50., 0.);

    let (mesh, _pos) = dragon.update_info();
    commands.spawn((
        PbrBundle {
            // mesh will be replaced every frame
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                ..default()
            }),
            transform: Transform::from_xyz(0., 1.0, 0.),
            ..default()
        },
        dragon,
        Name::new("Bunny"),
        Wireframe,
    ));
}

fn simulate_softbody(
    mut query: Query<(Entity, &mut Transform, &mut SoftBody, &mut Handle<Mesh>)>,
    time: Res<Time>,
    config: Res<Config>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let sdt = time.delta_seconds() / config.sub_steps as f32;

    // zero time blows up
    if sdt == 0. {
        return;
    }

    for _step in 0..config.sub_steps {
        for (_e, mut _trans, mut sb, _mesh_handle) in query.iter_mut() {
            sb.pre_solve(sdt, config.gravity);
        }

        for (_e, mut _trans, mut sb, _mesh_handle) in query.iter_mut() {
            sb.solve(sdt);
        }

        for (_e, mut _trans, mut sb, _mesh_handle) in query.iter_mut() {
            sb.post_solve(sdt);
        }
    }

    // update mesh
    // TODO: this is such a hack
    for (_e, mut trans, mut sb, mut mesh_handle) in query.iter_mut() {
        let (mesh, pos) = sb.update_info();
        trans.translation = pos;
        *mesh_handle = meshes.add(mesh);
    }
}

