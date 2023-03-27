use crate::BALL_COUNT;

use super::Keep;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

pub struct TextOverlayPlugin;

impl Plugin for TextOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_system(setup_overlay)
            .add_system(update_fps);
    }
}

#[derive(Component)]
struct FpsText;

const UI_SIZE: f32 = 20.0;

fn setup_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ui_font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands.spawn((
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(10.),
                    bottom: Val::Px(10.),
                    ..Default::default()
                },
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: TextStyle {
                            font: ui_font.clone(),
                            font_size: UI_SIZE,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: ui_font.clone(),
                            font_size: UI_SIZE,
                            color: Color::GREEN,
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        },
        Name::new("ui FPS"),
        Keep,
        FpsText,
    ));

    commands.spawn((
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(10.),
                    bottom: Val::Px(30.),
                    ..Default::default()
                },
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Count: ".to_string(),
                        style: TextStyle {
                            font: ui_font.clone(),
                            font_size: UI_SIZE,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: BALL_COUNT.to_string(),
                        style: TextStyle {
                            font: ui_font.clone(),
                            font_size: UI_SIZE,
                            color: Color::GREEN,
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        },
        Name::new("ui Ball Count"),
        Keep,
    ));
}

fn update_fps(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Update the value of the second section
                text.sections[1].value = format!("{:.0}", average);
                text.sections[1].style.color = match average {
                    x if x >= 50.0 => Color::GREEN,
                    x if x > 40.0 && x < 50.0 => Color::YELLOW,
                    x if x <= 40.0 => Color::RED,
                    _ => Color::WHITE,
                };
            }
        }
    }
}
