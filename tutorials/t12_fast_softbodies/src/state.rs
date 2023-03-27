use bevy::prelude::*;

use crate::{models::TetMesh, DragonAssets};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AppState>()
            .add_system(load_listen.in_set(OnUpdate(AppState::Loading)))
            .add_system(reset_listen.in_set(OnUpdate(AppState::Playing)))
            .add_system(pause_listen.in_set(OnUpdate(AppState::Playing)))
            .add_system(pause_stop_listen.in_set(OnUpdate(AppState::Pause)))
            .add_system(reset.in_set(OnUpdate(AppState::Reset)));
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

#[derive(Component)]
pub struct Keep;

fn reset(
    mut commands: Commands,
    query: Query<Entity, (Without<Keep>, Without<Window>, Without<Parent>)>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for e in query.iter() {
        commands.entity(e).despawn();
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
        info!("Pause");
        app_state.set(AppState::Pause);
    }
}

pub fn pause_stop_listen(keys: Res<Input<KeyCode>>, mut app_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Space) {
        app_state.set(AppState::Playing);
    }
}

fn load_listen(
    state: Res<DragonAssets>,
    tet_assets: Res<Assets<TetMesh>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    let tet_mesh = tet_assets.get(&state.tet_mesh);
    if tet_mesh.is_none() {
        return;
    }
    info!("Loading done");
    app_state.set(AppState::Playing);
}
