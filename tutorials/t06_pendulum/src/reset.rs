use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLinesMesh;

use crate::components::Pendulms;

pub struct ResetPlugin;

impl Plugin for ResetPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<ResetState>()            
            .add_system(reset_listen.in_set(OnUpdate(ResetState::Playing)))
            .add_system(reset.in_set(OnUpdate(ResetState::Reset)));
    }
}
#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum ResetState {
    #[default]
    Playing,
    Reset,
}

#[derive(Component)]
pub struct Keep;

fn reset(
    mut commands: Commands,
    query: Query<Entity, (Without<Keep>, Without<Window>, Without<Parent>, Without<DebugLinesMesh>)>,
    mut app_state: ResMut<NextState<ResetState>>,   
    mut pendulms : ResMut<Pendulms>,
) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
    app_state.set(ResetState::Playing);    
    pendulms.list.clear();
}

pub fn reset_listen(
    keys: Res<Input<KeyCode>>,
    mut app_state: ResMut<NextState<ResetState>>,    
) {
    if keys.just_pressed(KeyCode::R) {        
        app_state.set(ResetState::Reset);        
    }
}
