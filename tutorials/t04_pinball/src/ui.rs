use bevy::prelude::*;

use crate::{reset::Keep, Score};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FontAssets>()
            .add_startup_system(setup)
            .add_system(update_score);
    }
}

#[derive(Resource)]
pub struct FontAssets {
    pub ui_font: Handle<Font>,
}

impl FromWorld for FontAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let ui_font = asset_server.load("fonts/FiraSans-Bold.ttf");
        Self { ui_font }
    }
}

#[derive(Component)]
pub struct ScoreText;

fn setup(mut commands: Commands, fonts: ResMut<FontAssets>) {
    commands.spawn((
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(10.0),
                    top: Val::Px(10.0),
                    ..Default::default()
                },
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Score: ".to_string(),
                        style: TextStyle {
                            font: fonts.ui_font.clone(),
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: fonts.ui_font.clone(),
                            font_size: 30.0,
                            color: Color::GOLD,
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        },
        Keep,
        Name::new("ui Score"),
        ScoreText,
    ));
}

fn update_score(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if !score.is_changed() {
        return;
    }
    for mut text in query.iter_mut() {
        text.sections[1].value = format!("{:?}", score.0);
    }
}
