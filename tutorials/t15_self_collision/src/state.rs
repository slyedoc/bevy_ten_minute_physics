use bevy::prelude::*;

use crate::{assets::{TetMesh}, DragonAssets};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AppState>()
            .add_state::<DebugState>()
            .add_system(load_listen.in_set(OnUpdate(AppState::Loading)))
            .add_system(reset_listen.in_set(OnUpdate(AppState::Playing)))
            .add_system(pause_listen.in_set(OnUpdate(AppState::Playing)))
            .add_system(pause_stop_listen.in_set(OnUpdate(AppState::Pause)))
            .add_system(reset.in_set(OnUpdate(AppState::Reset)))
            .add_system(debug_start_listen.in_set(OnUpdate(DebugState::Off)))
            .add_system(debug_stop_listen.in_set(OnUpdate(DebugState::On)));
    }
}
#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Playing,
    Pause,
    Reset,
}

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum DebugState {
    #[default]
    Off,
    On,
}

#[derive(Component)]
pub struct Keep;

fn reset(
    mut commands: Commands,
    query: Query<Entity, (Without<Keep>, Without<Window>, Without<Parent>)>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
    app_state.set(AppState::Playing);
}

pub fn reset_listen(keys: Res<Input<KeyCode>>, mut app_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::R) {
        app_state.set(AppState::Reset);
    }
}

pub fn pause_listen(keys: Res<Input<KeyCode>>, mut app_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Space) {
        app_state.set(AppState::Pause);
    }
}

pub fn pause_stop_listen(keys: Res<Input<KeyCode>>, mut app_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Space) {
        app_state.set(AppState::Playing);
    }
}

pub fn debug_start_listen(
    keys: Res<Input<KeyCode>>,
    mut debug_state: ResMut<NextState<DebugState>>,
) {
    if keys.just_pressed(KeyCode::F1) {
        debug_state.set(DebugState::On);
    }
}

pub fn debug_stop_listen(
    keys: Res<Input<KeyCode>>,
    mut debug_state: ResMut<NextState<DebugState>>,
) {
    if keys.just_pressed(KeyCode::F1) {
        debug_state.set(DebugState::Off);
    }
}

fn load_listen(
    dragon_assets: Res<DragonAssets>,
    tet_meshes: Res<Assets<TetMesh>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    
    let tet_mesh = tet_meshes.get(&dragon_assets.tet_mesh);
   
    if tet_mesh.is_some() {
        app_state.set(AppState::Playing);
    }
}
