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
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(MenuRoot)
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(175.0), Val::Px(65.0)),
                        margin: Rect {
                            left: Val::Auto,
                            right: Val::Auto,
                            top: Val::Auto,
                            bottom: Val::Px(10.0),
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..Default::default()
                })
                .insert(MenuButton::Play)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Play",
                            TextStyle {
                                font: font_handle.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(175.0), Val::Px(65.0)),
                        margin: Rect {
                            top: Val::Px(1.0),
                            bottom: Val::Px(1.0),
                            left: Val::Auto,
                            right: Val::Auto,
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..Default::default()
                })
                .insert(MenuButton::Draw1)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Draw One",
                            TextStyle {
                                font: font_handle.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(175.0), Val::Px(65.0)),
                        margin: Rect {
                            top: Val::Px(1.0),
                            bottom: Val::Auto,
                            left: Val::Auto,
                            right: Val::Auto,
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..Default::default()
                })
                .insert(MenuButton::Draw3)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Draw Three",
                            TextStyle {
                                font: font_handle.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
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
    mut game_state: ResMut<State<game::GameState>>,
    interaction_query: Query<(&MenuButton, &Interaction), Changed<Interaction>>,
    mut q_buttons: Query<(&MenuButton, &mut UiColor)>,
) {
    for (button, interaction) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => {
                match button {
                    MenuButton::Play => {
                        game_state.set(game::GameState::Shuffle).unwrap();
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
    mut game_state: ResMut<State<game::GameState>>,
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
    win_text: Query<Entity, With<WinText>>,

) {
    for (entity, interaction) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => {
                game_state.set(game::GameState::Menu).unwrap();
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
    mut game_state: ResMut<State<game::GameState>>,
    mut draw_mode: ResMut<DrawMode>,
    interaction_query: Query<(&Interaction, &ResetButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, button) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => {
                match button {
                    ResetButton::Draw1 => {
                        *draw_mode = DrawMode::Draw1;
                    },
                    ResetButton::Draw3 => {
                        *draw_mode = DrawMode::Draw3;
                    },
                }
                game_state.set(game::GameState::Shuffle).unwrap();
            },
            _ => {},
        }
    }
}

pub fn spawn_win_screen(
    mut commands: Commands,
    windows: Res<Windows>,
    font: Res<FontHandle>,
    mut reset_menu: Query<&mut Style, With<ResetMenuRoot>>,
) {
    let window = windows.get_primary().unwrap();
    commands.spawn_bundle(Text2dBundle {
            text: Text::with_section(
                "You Won!",
                TextStyle {
                    font: font.0.clone(),
                    font_size: 100.0,
                    color: Color::WHITE,
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    vertical: VerticalAlign::Center,
                },
            ),
            transform: Transform::from_xyz(window.width() / 2.0, window.height() * 0.3, 100.0),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(Text2dBundle {
                text: Text::with_section(
                    "You Won!",
                    TextStyle {
                        font: font.0.clone(),
                        font_size: 100.0,
                        color: Color::BLACK,
                    },
                    // Note: You can use `Default::default()` in place of the `TextAlignment`
                    TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        vertical: VerticalAlign::Center,
                    },
                ),
                transform: Transform::from_xyz(3.0, -3.0, -1.0),
                ..Default::default()
            });
        })
        .insert(WinText);

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(175.0), Val::Px(65.0)),
                // center button
                margin: Rect {
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
            color: Color::rgb(0.15, 0.15, 0.15).into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "New Game",
                    TextStyle {
                        font: font.0.clone(),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        });

    for mut style in reset_menu.iter_mut() {
        style.display = Display::None;
    }
}
