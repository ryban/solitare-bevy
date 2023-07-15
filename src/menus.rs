use bevy::prelude::*;
use bevy::ui::Display;

use crate::game::{self, DrawMode, FontHandle};

#[derive(Component)]
pub struct WinText;

#[derive(Component)]
pub enum MenuButton {
    Play,
    Draw1,
    Draw3
}

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub enum ResetButton {
    Draw1,
    Draw3,
}

#[derive(Component)]
pub struct ResetMenuRoot;


pub fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>, mut reset_menu: Query<&mut Style, With<ResetMenuRoot>>) {
    let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(MenuRoot)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(175.0),
                        height: Val::Px(65.0),
                        margin: UiRect {
                            left: Val::Auto,
                            right: Val::Auto,
                            top: Val::Auto,
                            bottom: Val::Px(10.0),
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..Default::default()
                })
                .insert(MenuButton::Play)
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Play",
                            TextStyle {
                                font: font_handle.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            }
                        ),
                        ..Default::default()
                    });
                });
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(175.0),
                        height: Val::Px(65.0),
                        margin: UiRect {
                            top: Val::Px(1.0),
                            bottom: Val::Px(1.0),
                            left: Val::Auto,
                            right: Val::Auto,
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..Default::default()
                })
                .insert(MenuButton::Draw1)
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Draw One",
                            TextStyle {
                                font: font_handle.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ),
                        ..Default::default()
                    });
                });
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(175.0),
                        height: Val::Px(65.0),
                        margin: UiRect {
                            top: Val::Px(1.0),
                            bottom: Val::Auto,
                            left: Val::Auto,
                            right: Val::Auto,
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..Default::default()
                })
                .insert(MenuButton::Draw3)
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Draw Three",
                            TextStyle {
                                font: font_handle.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ),
                        ..Default::default()
                    });
                });
        });

    for mut style in reset_menu.iter_mut() {
        style.display = Display::None;
    }
}

pub fn show_menu(mut menu_root: Query<&mut Style, With<MenuRoot>>) {
    for mut style in menu_root.iter_mut() {
        style.display = Display::Flex;
    }
}

pub fn hide_menu(mut menu_root: Query<&mut Style, With<MenuRoot>>) {
    for mut style in menu_root.iter_mut() {
        style.display = Display::None;
    }
}

pub fn main_menu(
    mut draw_mode: ResMut<DrawMode>,
    mut game_state: ResMut<NextState<game::GameState>>,
    interaction_query: Query<(&MenuButton, &Interaction), Changed<Interaction>>,
    mut q_buttons: Query<(&MenuButton, &mut BackgroundColor)>,
) {
    for (button, interaction) in interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                match button {
                    MenuButton::Play => {
                        game_state.set(game::GameState::Shuffle);
                    },
                    MenuButton::Draw1 => {
                        *draw_mode = DrawMode::Draw1;
                    },
                    MenuButton::Draw3 => {
                        *draw_mode = DrawMode::Draw3;
                    },
                }
            },
            _ => {},
        }
    }
    if draw_mode.is_changed() {
        for (button, mut color) in q_buttons.iter_mut() {
            match button {
                MenuButton::Draw1 => {
                    if *draw_mode == DrawMode::Draw1 {
                        *color = Color::rgb(0.15, 0.15, 0.15).into();
                    } else {
                        *color = Color::rgb(0.5, 0.5, 0.5).into();
                    }
                },
                MenuButton::Draw3 => {
                    if *draw_mode == DrawMode::Draw3 {
                        *color = Color::rgb(0.15, 0.15, 0.15).into();
                    } else {
                        *color = Color::rgb(0.5, 0.5, 0.5).into();
                    }
                },
                MenuButton::Play => {},
            }
        }
    }
}

pub fn win_screen(
    mut commands: Commands,
    mut game_state: ResMut<NextState<game::GameState>>,
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
    win_text: Query<Entity, With<WinText>>,

) {
    for (entity, interaction) in interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                game_state.set(game::GameState::Menu);
                commands.entity(entity).despawn_recursive();
                for e in win_text.iter() {
                    commands.entity(e).despawn_recursive();
                }
            },
            _ => {},
        }
    }
}

pub fn reset_game_button(
    mut game_state: ResMut<NextState<game::GameState>>,
    mut draw_mode: ResMut<DrawMode>,
    interaction_query: Query<(&Interaction, &ResetButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, button) in interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                match button {
                    ResetButton::Draw1 => {
                        *draw_mode = DrawMode::Draw1;
                    },
                    ResetButton::Draw3 => {
                        *draw_mode = DrawMode::Draw3;
                    },
                }
                game_state.set(game::GameState::Shuffle);
            },
            _ => {},
        }
    }
}

pub fn spawn_win_screen(
    mut commands: Commands,
    windows: Query<&Window>,
    font: Res<FontHandle>,
    mut reset_menu: Query<&mut Style, With<ResetMenuRoot>>,
) {
    let window = if let Ok(w) = windows.get_single() {w} else {return};
    commands.spawn(Text2dBundle {
            text: Text::from_section(
                "You Won!",
                TextStyle {
                    font: font.0.clone(),
                    font_size: 100.0,
                    color: Color::WHITE,
                }
            ).with_alignment(TextAlignment::Center),
            transform: Transform::from_xyz(0.0, -window.height() * 0.1, 100.0),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    "You Won!",
                    TextStyle {
                        font: font.0.clone(),
                        font_size: 100.0,
                        color: Color::BLACK,
                    }
                )
                .with_alignment(TextAlignment::Center),
                transform: Transform::from_xyz(3.0, -3.0, -1.0),
                ..Default::default()
            });
        })
        .insert(WinText);

    commands
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(175.0),
                height: Val::Px(65.0),
                // center button
                margin: UiRect {
                    bottom: Val::Px(100.0),
                    top: Val::Auto,
                    left: Val::Auto,
                    right: Val::Auto,
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "New Game",
                    TextStyle {
                        font: font.0.clone(),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ),
                ..Default::default()
            });
        });

    for mut style in reset_menu.iter_mut() {
        style.display = Display::None;
    }
}
