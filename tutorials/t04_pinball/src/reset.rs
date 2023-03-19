use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLinesMesh;

use crate::Score;

pub struct ResetPlugin;

impl Plugin for ResetPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state(ResetState::Playing)            
            .add_system_set(SystemSet::on_update(ResetState::Playing).with_system(reset_listen))
            .add_system_set(SystemSet::on_update(ResetState::Reset).with_system(reset));
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ResetState {
    Playing,
    Reset,
}

#[derive(Component)]
pub struct Keep;

fn reset(
    mut commands: Commands,
    query: Query<Entity, (Without<Keep>, Without<Parent>, Without<DebugLinesMesh>)>,
    mut app_state: ResMut<State<ResetState>>,
    mut score: ResMut<Score>,
) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
    app_state.set(ResetState::Playing).unwrap();
    score.0 = 0;
}

pub fn reset_listen(mut keys: ResMut<Input<KeyCode>>, mut app_state: ResMut<State<ResetState>>) {
    if keys.just_pressed(KeyCode::R) {
        if app_state.current() == &ResetState::Reset {
            return;
        }
        app_state.set(ResetState::Reset).unwrap();
        keys.reset(KeyCode::R);
    }
}
