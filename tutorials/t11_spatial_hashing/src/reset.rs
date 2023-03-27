use bevy::prelude::*;

pub struct ResetPlugin;

impl Plugin for ResetPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<ResetState>()
            .add_system(reset_listen.in_set(OnUpdate(ResetState::Playing)))
            .add_system(pause_listen.in_set(OnUpdate(ResetState::Playing)))
            .add_system(pause_stop_listen.in_set(OnUpdate(ResetState::Pause)))
            .add_system(reset.in_set(OnUpdate(ResetState::Reset)));
    }
}
#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum ResetState {
    #[default]
    Playing,
    Pause,
    Reset,
}

#[derive(Component)]
pub struct Keep;

fn reset(
    mut commands: Commands,
    query: Query<Entity, (Without<Keep>, Without<Window>, Without<Parent>)>,
    mut app_state: ResMut<NextState<ResetState>>,
) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
    app_state.set(ResetState::Playing);
}

pub fn reset_listen(keys: Res<Input<KeyCode>>, mut app_state: ResMut<NextState<ResetState>>) {
    if keys.just_pressed(KeyCode::R) {
        app_state.set(ResetState::Reset);
    }
}

pub fn pause_listen(keys: Res<Input<KeyCode>>, mut app_state: ResMut<NextState<ResetState>>) {
    if keys.just_pressed(KeyCode::Space) {
        info!("Pause");
        app_state.set(ResetState::Pause);
    }
}

pub fn pause_stop_listen(keys: Res<Input<KeyCode>>, mut app_state: ResMut<NextState<ResetState>>) {
    if keys.just_pressed(KeyCode::Space) {
        app_state.set(ResetState::Playing);
    }
}
