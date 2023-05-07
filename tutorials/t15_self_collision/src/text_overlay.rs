use crate::bodies::Cloth;

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
            .add_system(update_fps)
            .add_system(update_verts)
            .add_system(update_tri_count);
    }
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct VertCountText;

#[derive(Component)]
struct TriCountText;

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
                            color: Color::GRAY,
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
                        value: "Verts: ".to_string(),
                        style: TextStyle {
                            font: ui_font.clone(),
                            font_size: UI_SIZE,
                            color: Color::GRAY,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: ui_font.clone(),
                            font_size: UI_SIZE,
                            color: Color::BLACK,
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        },
        Name::new("Vert Count"),
        Keep,
        VertCountText,
    ));

    commands.spawn((
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(10.),
                    bottom: Val::Px(50.),
                    ..Default::default()
                },
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Tri: ".to_string(),
                        style: TextStyle {
                            font: ui_font.clone(),
                            font_size: UI_SIZE,
                            color: Color::GRAY,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: ui_font.clone(),
                            font_size: UI_SIZE,
                            color: Color::BLACK,
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        },
        Name::new("Vert Count"),
        Keep,
        TriCountText,
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

fn update_verts(
    mut query: Query<&mut Text, With<VertCountText>>,
    cloth_query: Query<&Handle<Cloth>>,
    cloths: Res<Assets<Cloth>>,
) {
    let mut count = 0;
    for cloth_handle in cloth_query.iter() {
        // Update the value of the second section
        let cloth = cloths.get(cloth_handle).unwrap();
        count += cloth.vert_count()
    }
    for mut text in query.iter_mut() {
            text.sections[1].value = format!("{}", count);

    }
}


fn update_tri_count(
    mut query: Query<&mut Text, With<TriCountText>>,
    cloth_query: Query<&Handle<Cloth>>,
    cloths: Res<Assets<Cloth>>,
) {
    let mut count = 0;
    for cloth_handle in cloth_query.iter() {
        // Update the value of the second section
        let cloth = cloths.get(cloth_handle).unwrap();
        count += cloth.tri_count()
    }
    for mut text in query.iter_mut() {
            text.sections[1].value = format!("{}", count);

    }
}
