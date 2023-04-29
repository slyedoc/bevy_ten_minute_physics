mod camera_grabber;
mod intersect;
mod models;
mod resources;
mod softbody;
mod spatial_hash;
mod state;
mod text_overlay;

use camera_grabber::*;
use models::*;
use resources::*;
use softbody::*;
use state::*;
use text_overlay::*;

use bevy_atmosphere::prelude::*;

use bevy::{
    pbr::{
        wireframe::{Wireframe, WireframePlugin},
        NotShadowCaster,
    },
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use std::f32::consts::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(TetMeshPlugin)
        .add_plugin(StatePlugin)
        .add_plugin(TextOverlayPlugin)
        .add_plugin(CameraGrabberPlugin)
        .add_plugin(AtmospherePlugin)
        .add_plugin(WireframePlugin)
        .add_asset::<SoftBody>()
        //.insert_resource(ClearColor(Color::BLACK))
        .init_resource::<DragonAssets>()
        .init_resource::<Config>()
        .add_startup_system(setup)
        .add_system(spawn_dragon.in_schedule(OnEnter(AppState::Playing)))
        .add_system(simulate_softbody.in_set(OnUpdate(AppState::Playing)))
        //.add_system(fix_added_softbody.in_set(OnUpdate(AppState::Playing)).before(simulate_softbody))
        // debug
        .add_system(spawn_debug_children.in_schedule(OnEnter(DebugState::On)))
        .add_system(
            update_debug_children
                .in_set(OnUpdate(DebugState::On))
                .after(simulate_softbody),
        )
        .add_system(remove_debug_children.in_schedule(OnExit(DebugState::On)))
        .register_type::<Config>()
        .register_type::<Ball>()
        .register_type::<Velocity>()
        .register_type::<SoftBody>()
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
        // ambient light
        commands.insert_resource(AmbientLight {
            color: Color::ORANGE_RED,
            brightness: 0.02,
        });
        
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0., 2., 5.).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y ),
            ..Default::default()
        },
        CameraGrabber::default(),
        AtmosphereCamera::default(),
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

    // light
    commands.spawn((
        DirectionalLightBundle {
            transform: Transform::from_xyz(-50.0, 50.0, -50.0).looking_at(Vec3::ZERO, Vec3::Y),
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
                translation: Vec3::new(0., 0.0, 0.),
                rotation: Quat::from_rotation_y(FRAC_PI_2),
                ..default()
            },
            ..default()
        },
        Name::new("Ground"),
        Keep,
    ));

    info!("Press 'R' to reset");
    info!("Press 'F1' enable debug wireframe");
}

fn spawn_dragon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dragon_assets: Res<DragonAssets>,
    mut tet_meshes: ResMut<Assets<TetMesh>>,
    mut softbodies: ResMut<Assets<SoftBody>>,
) {
    info!("Spawning dragon");

    let dragon = tet_meshes.get_mut(&dragon_assets.tet_mesh).unwrap();
    let sb = SoftBody::new(dragon, 20., 0.0);
    let mesh_handle = meshes.add(Mesh::from(&sb));
    let sb_handle = softbodies.add(sb);

    commands.spawn((
        PbrBundle {
            // mesh will be replaced every frame
            mesh: mesh_handle,
            material: materials.add(StandardMaterial {
                base_color: Color::ORANGE,
                perceptual_roughness: 0.5,
                metallic: 0.5,
                ..default()
            }),
            transform: Transform::from_xyz(0., -0.5, 0.),
            ..default()
        },
        sb_handle,
        Name::new("Dragon"),
    ));
}

fn simulate_softbody(
    mut query: Query<(&mut Transform, &Handle<SoftBody>, &Handle<Mesh>)>,
    time: Res<Time>,
    config: Res<Config>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut softbodies: ResMut<Assets<SoftBody>>,
) {
    let sdt = time.delta_seconds() / config.sub_steps as f32;

    // zero time blows up on startup
    if sdt == 0. {
        return;
    }

    for _step in 0..config.sub_steps {
        for (mut _trans, sb_handle, _mesh_handle) in query.iter_mut() {
            let sb = softbodies.get_mut(sb_handle).unwrap();
            sb.pre_solve(sdt, config.gravity);
        }

        for (mut _trans, sb_handle, _mesh_handle) in query.iter_mut() {
            let sb = softbodies.get_mut(sb_handle).unwrap();
            sb.solve(sdt);
        }

        for (mut _trans, sb_handle, _mesh_handle) in query.iter_mut() {
            let sb = softbodies.get_mut(sb_handle).unwrap();
            sb.post_solve(sdt);
        }
    }

    // update mesh, kind of hacky
    for (mut trans, sb_handle, mesh_handle) in query.iter_mut() {
        let sb = softbodies.get_mut(sb_handle).unwrap();
        sb.update_transform(&mut trans);

        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            sb.update_visual_mesh(&trans, mesh);
        }
    }
}

#[derive(Component)]
struct TetMeshDebug;

#[derive(Component)]
struct SphereDebug;

fn spawn_debug_children(
    mut commands: Commands,
    mut softbodies: ResMut<Assets<SoftBody>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &Handle<SoftBody>)>,
) {
    for (e, sb_handle) in query.iter_mut() {
        let sb = softbodies.get_mut(sb_handle).unwrap();
        let tet_mesh_handle = meshes.add(sb.create_tet_mesh());
        let tet_child = commands
            .spawn((
                PbrBundle {                    
                    mesh: tet_mesh_handle,
                    transform: Transform::from_xyz(0., 0.0, 0.),
                    ..default()
                },
                Wireframe,
                TetMeshDebug,
                NotShadowCaster,
                Name::new("TetMesh"),
            ))
            .id();

        // let sphere_child = commands.spawn((
        //     PbrBundle {
        //         // mesh will be replaced every frame
        //         mesh: meshes.add(Mesh::from(shape::UVSphere {
        //             radius: sb.radius,
        //             sectors: 18,
        //             stacks: 9,
        //         })),
        //         material: materials.add(StandardMaterial {
        //             // base_color: Color::R,
        //             alpha_mode: AlphaMode::Add,
        //             ..default()
        //         }),
        //         transform: Transform::from_xyz(0., 0., 0.),
        //         ..default()
        //     },
        //     Wireframe,
        //     SphereDebug,
        //     NotShadowCaster,
        //     Name::new("Radius"),
        // )).id();
        commands.entity(e).push_children(&[tet_child]);
    }
}

fn update_debug_children(
    mut query: Query<
        (&Transform, &Handle<SoftBody>, &Children),
        (Without<SphereDebug>, Without<TetMeshDebug>),
    >,
    tet_mesh_child: Query<&Handle<Mesh>, (With<TetMeshDebug>, Without<SphereDebug>)>,
    mut sphere_mesh_child: Query<(&mut Transform, &mut Handle<Mesh>), With<SphereDebug>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut softbodies: ResMut<Assets<SoftBody>>,
) {
    for (trans, sb_handle, children) in query.iter_mut() {
        let sb = softbodies.get_mut(sb_handle).unwrap();

        // update tet mesh
        for child in children.iter() {
            if let Ok(child_mesh_handle) = tet_mesh_child.get(*child) {
                let tet_mesh = meshes.get_mut(child_mesh_handle).unwrap();
                sb.update_tet_mesh(trans, tet_mesh);
            }
            if let Ok((mut _trans, mut child_mesh_handle)) = sphere_mesh_child.get_mut(*child) {
                // TODO: Hate recreating the mesh, will replace with gizmo in 0.11
                *child_mesh_handle = meshes.add(Mesh::from(shape::UVSphere {
                    radius: sb.radius,
                    sectors: 18,
                    stacks: 9,
                }));
            }
        }
    }
}

fn remove_debug_children(
    mut commands: Commands,
    mut tet_query: Query<Entity, With<TetMeshDebug>>,
    mut sphere_query: Query<Entity, With<SphereDebug>>,
) {
    for e in tet_query.iter_mut() {
        commands.entity(e).despawn_recursive();
    }
    for e in sphere_query.iter_mut() {
        commands.entity(e).despawn_recursive();
    }
}
