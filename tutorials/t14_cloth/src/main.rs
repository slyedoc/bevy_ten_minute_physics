mod assets;
mod bodies;
mod camera_grabber;
mod intersect;
mod math;
mod resources;
mod spatial_hash;
mod state;
mod text_overlay;

use assets::*;
use bodies::*;
use camera_grabber::*;
use resources::*;
use state::*;
use text_overlay::*;

use bevy_atmosphere::prelude::*;

use bevy::{
    pbr::{
        wireframe::{Wireframe, WireframePlugin},
        NotShadowCaster, CascadeShadowConfigBuilder, NotShadowReceiver,
    },
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use std::f32::consts::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(MeshAssetsPlugin)
        .add_plugin(StatePlugin)
        .add_plugin(TextOverlayPlugin)
        .add_plugin(CameraGrabberPlugin)
        .add_plugin(AtmospherePlugin)
        .add_plugin(WireframePlugin)
        .add_asset::<SoftBody>()
        .add_asset::<Cloth>()
        //.insert_resource(ClearColor(Color::BLACK))
        .init_resource::<DragonAssets>()
        .init_resource::<Config>()
        .add_startup_system(setup)
        .add_system(spawn_cloth.in_schedule(OnEnter(AppState::Playing)))
        //.add_system(spawn_dragon.in_schedule(OnEnter(AppState::Playing)))
        .add_system(simulate.in_set(OnUpdate(AppState::Playing)))
        //.add_system(fix_added_softbody.in_set(OnUpdate(AppState::Playing)).before(simulate_softbody))
        // debug
        .add_system(spawn_debug_children.in_schedule(OnEnter(DebugState::On)))
        .add_system(
            update_debug_children
                .in_set(OnUpdate(DebugState::On))
                .after(simulate),
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
}

impl FromWorld for DragonAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let tet_mesh = asset_server.load("model/dragon.tet.json");

        DragonAssets { tet_mesh }
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
            transform: Transform::from_xyz(0., 2., 5.)
                .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
            ..Default::default()
        },
        CameraGrabber::default(),
        AtmosphereCamera::default(),
        Keep,
    ));

    commands.spawn((DirectionalLightBundle {
        transform: Transform::from_xyz(-5.0, 5.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    },
    Keep)
);

    // // light
    // commands.spawn((
    //     DirectionalLightBundle {
    //         transform: Transform::from_xyz(-50.0, 50.0, -50.0).looking_at(Vec3::ZERO, Vec3::Y),
    //         directional_light: DirectionalLight {
    //             shadows_enabled: true,
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     Keep,
    // ));

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
                translation: Vec3::new(0., -0.0001, 0.),
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

fn spawn_cloth(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cloth: ResMut<Assets<Cloth>>,
) {
    info!("Spawning cloth");

    let subdivisions = 20;
    let mesh = Mesh::from(shape::Plane {
        size: 1.0,
        subdivisions,
        ..default()
    });
    
    let offset = Transform {  
        translation: Vec3::new(0., 2.0, 0.),
        rotation: Quat::from_euler(EulerRot::XYZ, -FRAC_PI_2, FRAC_PI_2, 0.0  ),
        ..default()
    };

    let z_vertex_count = subdivisions + 2;
    let x_vertex_count = subdivisions + 2;    
    let corner_index = ((z_vertex_count - 1) * (x_vertex_count - 1)) as usize;

    let c = Cloth::new( &mesh, 0.9, &offset, &[0,  corner_index] );

    commands.spawn((
        PbrBundle {
            // mesh will be replaced every frame
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::ORANGE,
                perceptual_roughness: 0.5,
                cull_mode: None,
                metallic: 0.5,
                ..default()
            }),
            transform: Transform::from_xyz(0., 2.0, 0.),
            ..default()
        },
        cloth.add(c),
        // Wireframe,
        NotShadowReceiver,
        Name::new("Cloth"),
    ));
}

#[allow(dead_code)]
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

fn simulate(
    mut query_softbody: Query<
        (&mut Transform, &Handle<SoftBody>, &Handle<Mesh>),
        Without<Handle<Cloth>>,
    >,
    mut query_cloth: Query<
        (&mut Transform, &Handle<Cloth>, &Handle<Mesh>),
        Without<Handle<SoftBody>>,
    >,
    time: Res<Time>,
    config: Res<Config>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut softbodies: ResMut<Assets<SoftBody>>,
    mut cloths: ResMut<Assets<Cloth>>,
) {
    let sdt = time.delta_seconds() / config.sub_steps as f32;

    // zero time blows up on startup
    if sdt == 0. {
        return;
    }

    for _step in 0..config.sub_steps {
        for (mut _trans, sb_handle, _mesh_handle) in query_softbody.iter_mut() {
            let sb = softbodies.get_mut(sb_handle).unwrap();
            sb.pre_solve(sdt, config.gravity);
        }

        for (mut _trans, cloth_handle, _mesh_handle) in query_cloth.iter_mut() {
            let cloth = cloths.get_mut(cloth_handle).unwrap();
            cloth.pre_solve(sdt, config.gravity);
        }

        for (mut _trans, sb_handle, _mesh_handle) in query_softbody.iter_mut() {
            let sb = softbodies.get_mut(sb_handle).unwrap();
            sb.solve(sdt);
        }

        for (mut _trans, cloth_handle, _mesh_handle) in query_cloth.iter_mut() {
            let cloth = cloths.get_mut(cloth_handle).unwrap();
            cloth.solve(sdt);
        }

        for (mut _trans, sb_handle, _mesh_handle) in query_softbody.iter_mut() {
            let sb = softbodies.get_mut(sb_handle).unwrap();
            sb.post_solve(sdt);
        }

        for (mut _trans, cloth_handle, _mesh_handle) in query_cloth.iter_mut() {
            let cloth = cloths.get_mut(cloth_handle).unwrap();
            cloth.post_solve(sdt);
        }
    }

    // update mesh, kind of hacky
    for (mut trans, sb_handle, mesh_handle) in query_softbody.iter_mut() {
        let sb = softbodies.get_mut(sb_handle).unwrap();
        sb.update_transform(&mut trans);

        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            sb.update_visual_mesh(&trans, mesh);
        }
    }

    for (mut trans, cloth_handle, mesh_handle) in query_cloth.iter_mut() {
        let cloth = cloths.get_mut(cloth_handle).unwrap();
        cloth.update_transform(&mut trans);

        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            cloth.update_visual_mesh(&trans, mesh);
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
