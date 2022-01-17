use std::collections::HashMap;
use bevy::prelude::*;
use bevy::render::camera::{WindowOrigin, ScalingMode};
use bevy::render::view::Visibility;
use bevy::input::{keyboard::KeyCode, Input, mouse::MouseMotion};
use bevy::ui::Display;
// use bevy_easings::*;
use rand::prelude::*;

// const BACK_GREEN: usize = 5 * 13;
const BACK_BLUE: usize = 6 * 13;
// const BACK_RED: usize = 7 * 13;
const FINAL_STACKS: usize = 7 * 13 + 9;
const EMPTY_SPACE: usize = 6 * 13 + 12;

const CARD_WIDTH: f32 = 140.0;
const CARD_HEIGHT: f32 = 190.0;
const CARD_STACK_SPACE: f32 = 35.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Suit {
    Spades,
    Clubs,
    Hearts,
    Diamonds,
}

impl Suit {
    fn can_stack(&self, other: &Self) -> bool {
        match self {
            Self::Spades | Self::Clubs => match other {
                Self::Hearts | Self::Diamonds => true,
                Self::Spades | Self::Clubs => false,
            },
            Self::Hearts | Self::Diamonds => match other {
                Self::Spades | Self::Clubs => true,
                Self::Hearts | Self::Diamonds => false,
            }
        }
    }
}

impl Suit {
    fn row(&self) -> usize {
        match self {
            Suit::Spades => 0,
            Suit::Clubs => 1,
            Suit::Diamonds => 2,
            Suit::Hearts => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CardKind {
    Ace,
    Number(usize),
    Jack,
    Queen,
    King,
    // Joker,
}

impl CardKind {
    fn column(&self) -> usize {
        match self {
            CardKind::Ace => 0,
            // 2:10 => 1:9
            CardKind::Number(n) => n - 1,
            CardKind::Jack => 10,
            CardKind::Queen => 11,
            CardKind::King => 12,
        }
    }

    fn can_stack(&self, other: &Self) -> bool {
        self != &CardKind::Ace && other != &CardKind::Ace && self.column() < other.column() && (other.column() - self.column() == 1)
    }

    fn next(&self) -> Option<CardKind> {
        match self {
            CardKind::Ace => Some(CardKind::Number(2)),
            // 2:10 => 1:9
            CardKind::Number(n) => Some(if *n < 10 {CardKind::Number(n + 1)} else {CardKind::Jack}),
            CardKind::Jack => Some(CardKind::Queen),
            CardKind::Queen => Some(CardKind::King),
            CardKind::King => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
struct Card {
    suit: Suit,
    kind: CardKind,
}

impl Card {
    fn texture_index(&self) -> usize {
        (self.suit.row() * 13) + self.kind.column()
    }

    /// Return true if other can be stacked below self
    fn can_stack(&self, other: &Card) -> bool {
        self.suit.can_stack(&other.suit) && self.kind.can_stack(&other.kind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StackKind {
    Ordered(Suit),
    Stack,
}

#[derive(Debug, Component)]
struct Deck {
    cards: Vec<Card>,
}

// XXX: Possibly change this to be a Card field instead of an individual component?
#[derive(Debug, Clone, Copy, PartialEq, Component)]
enum CardFace {
    Down,
    Up,
}

#[derive(Debug)]
struct Area {
    pos: Vec2,
    size: Vec2,
}

impl Area {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            size: Vec2::new(width, height),
        }
    }

    fn contains(&self, pos: Vec2) -> bool {
        // FIXME: Change it so the position is relative to the center of the entity? Or should it always be global like it is now?
        pos.x >= self.pos.x &&
        pos.x <= (self.pos.x + self.size.x) &&
        pos.y >= self.pos.y &&
        pos.y <= (self.pos.y + self.size.y)
    }
}

#[derive(Debug, Component)]
struct Clickable {
    zone: Area,
}

impl Default for Clickable {
    fn default() -> Self {
        Self {
            zone: Area {pos: Vec2::new(0.0, 0.0), size: Vec2::new(CARD_WIDTH, CARD_HEIGHT)},
        }
    }
}

impl Clickable {
    fn at(pos: Vec2) -> Self {
        let size = Vec2::new(CARD_WIDTH, CARD_HEIGHT);
        Self {
            zone: Area {pos: pos - (size / 2.0), size: size},
            ..Default::default()
        }
    }
}


#[derive(Component)]
struct WinText;

#[derive(Component)]
enum MenuButton {
    Play,
    Draw1,
    Draw3
}

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
enum ResetButton {
    Draw1,
    Draw3,
}

#[derive(Component)]
struct ResetMenuRoot;

#[derive(Debug, PartialEq)]
enum DrawMode {
    Draw1,
    Draw3,
}

impl DrawMode {
    fn num(&self) -> usize {
        match self {
            DrawMode::Draw1 => 1,
            DrawMode::Draw3 => 3,
        }
    }
}

#[derive(Debug, Clone, Component, PartialEq)]
enum MouseInteraction {
    /// The mouse is over this entity
    Hovered,
    /// The left mouse is held down on this entity
    Clicked(Vec2),
    /// Entity is being dragged
    Dragging {
        /// 2D global start location of the dragged entity
        global_start_pos: Vec3,
        /// Original start translation
        local_start_pos: Vec3,
        /// Position of the mouse on the screen when the drag started
        start_mouse: Vec2,
    },
}

impl MouseInteraction {
    fn is_hovered(&self) -> bool {
        match self {
            MouseInteraction::Hovered => true,
            _ => false,
        }
    }

    fn is_clicked(&self) -> bool {
        match self {
            MouseInteraction::Clicked(_) => true,
            _ => false,
        }
    }

    fn is_dragging(&self) -> bool {
        match self {
            MouseInteraction::Dragging {..} => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
struct Clicked(Entity, Vec2);

#[derive(Debug, Clone)]
struct Released(Entity, Vec2);

#[derive(Debug, Clone, Component)]
struct LeftClicked(Vec2);

#[derive(Debug, Component)]
struct WasClicked(Timer);

// TODO: Also put the old Transform in here
// to more easilly reset the position when its dropped
#[derive(Debug, Component, Default)]
struct Dragging(Vec2);

#[derive(Debug, Component)]
struct Draggable;

#[derive(Debug, Component)]
struct Droppable {
    zone: Area,
}

#[derive(Debug)]
struct Dropped(Entity, Vec3, Vec2);

#[derive(Debug, Default, Component)]
struct DiscardPile;

#[derive(Debug, Component, Clone)]
struct Stack {
    kind: StackKind,
}

impl Stack {
    fn new(kind: StackKind) -> Self {
        Self {
            kind: kind,
        }
    }

    fn can_stack(&self, stack_card: Option<&Card>, target: Card, has_children: bool) -> bool {
        // info!("can_stack? stack={:?} stack_card={:?} target={:?}", self, stack_card, target);
        match self.kind {
            StackKind::Ordered(suit) => {
                if has_children {
                    false
                } else if stack_card.is_none() && suit == target.suit && target.kind == CardKind::Ace {
                    true
                } else if suit == target.suit && Some(target.kind) == stack_card.map(|c| c.kind.next()).flatten() {
                    true
                } else {
                    false
                }
            },
            StackKind::Stack => {
                if stack_card.is_none() && target.kind == CardKind::King {
                    true
                } else if let Some(stack_card) = stack_card {
                    target.can_stack(&stack_card)
                } else {
                    false
                }
            },
        }
    }
}

/// Resource for holding the card texture atlas
struct CardsTextureHandle(Handle<TextureAtlas>);
struct FontHandle(Handle<Font>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    Menu,
    Playing,
    Shuffle,
    Won,
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Solitaire".to_string(),
            width: 1280.,
            height: 960.,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.3, 0.7, 0.1)))
        .insert_resource(DrawMode::Draw1)
        .add_state(GameState::Menu)
        .add_event::<Released>()
        .add_event::<Dropped>()
        .add_startup_system(setup)
        .add_startup_system(setup_menu)
        .add_system_set(
            SystemSet::on_enter(GameState::Menu)
                .with_system(show_menu)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Menu)
                .with_system(main_menu)
        )
        .add_system_set(
            SystemSet::on_exit(GameState::Menu)
                .with_system(hide_menu)
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Shuffle)
                .with_system(clean_cards)
                .with_system(reset_cards)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(update_click_timers)
                // Run this before the mouse system so the double click sets the translation to the
                // completed pile rather than the discard pile resetting it
                // It would nice to make this event based so its not running constantly anyways
                .with_system(discard_update_system.before("mouse"))
                .with_system(mouse_interaction_system.label("mouse"))
                .with_system(click_system.after("mouse"))
                .with_system(drop_system.after("mouse"))
                .with_system(win_check_system)
                .with_system(card_texture_update_system)
                .with_system(deck_update_system)
                .with_system(debug_duplicate_children)
                .with_system(reset_game_button)
                .with_system(auto_win_system)
        )
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(clickable_bounds_update_system)
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Won)
                .with_system(spawn_win_screen)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Won)
                .with_system(win_screen)
        )
        .add_system_set(
            SystemSet::on_exit(GameState::Won)
                .with_system(clean_cards)
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    camera_bundle.orthographic_projection.scaling_mode = ScalingMode::WindowSize;
    commands.spawn_bundle(camera_bundle);

    commands.spawn_bundle(UiCameraBundle::default());
    // commands.spawn_bundle(UiCameraBundle::default());

    // Pre-load this now
    let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.insert_resource(FontHandle(font_handle.clone()));
    let card_texture_handle = asset_server.load("playingCards.png");
    let card_atlas = TextureAtlas::from_grid(card_texture_handle, Vec2::new(CARD_WIDTH, CARD_HEIGHT), 13, 8);
    let card_atlas_handle = texture_atlases.add(card_atlas);
    commands.insert_resource(CardsTextureHandle(card_atlas_handle.clone()));

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                display: Display::None,
                margin: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(5.0),
                    top: Val::Auto,
                    right: Val::Auto,
                },
                justify_content: JustifyContent::FlexStart,
                align_content: AlignContent::FlexStart,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(ResetMenuRoot)
        .with_children(|parent| {
            parent.spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(20.0)),
                        margin: Rect {
                            right: Val::Px(5.0),
                            ..Default::default()
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
                .insert(ResetButton::Draw1)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "New Single Draw",
                            TextStyle {
                                font: font_handle.clone(),
                                font_size: 20.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });

            parent.spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(20.0)),
                        margin: Rect {
                            left: Val::Px(5.0),
                            ..Default::default()
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
                .insert(ResetButton::Draw3)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "New Triple Draw",
                            TextStyle {
                                font: font_handle.clone(),
                                font_size: 20.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
        });
}

fn clean_cards(mut commands: Commands, cleanup: Query<Entity, Or<(With<Card>, With<Stack>, With<Deck>, With<DiscardPile>)>>) {
    for entity in cleanup.iter() {
        commands.entity(entity).despawn_recursive();
    }

}

fn reset_cards(
    mut commands: Commands,
    mut game_state: ResMut<State<GameState>>,
    card_texture: Res<CardsTextureHandle>,
    window: Res<WindowDescriptor>,
    mut reset_menu: Query<&mut Style, With<ResetMenuRoot>>
) {
    let mut rng = rand::thread_rng();
    let mut deck = Vec::new();
    for suit in [Suit::Spades, Suit::Clubs, Suit::Hearts, Suit::Diamonds] {
        deck.push(Card {suit: suit, kind: CardKind::Ace});
        for n in 2..=10 {
            deck.push(Card {suit: suit, kind: CardKind::Number(n)});
        }
        deck.push(Card {suit: suit, kind: CardKind::Jack});
        deck.push(Card {suit: suit, kind: CardKind::Queen});
        deck.push(Card {suit: suit, kind: CardKind::King});
    }

    deck.shuffle(&mut rng);

    let stacks_y = window.height * 0.6;

    fn stack_x(stack: usize) -> f32 {
        50.0 + (175.0 * (stack as f32)) + (CARD_WIDTH / 2.0)
    }

    let mut stacks: Vec<Vec<Entity>> = (0..7).map(|stack| {
        let pos = Vec2::new(stack_x(stack), stacks_y);
        vec![
            commands.spawn_bundle(SpriteSheetBundle {
                    texture_atlas: card_texture.0.clone(),
                    sprite: TextureAtlasSprite {
                        index: EMPTY_SPACE,
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                    ..Default::default()
                })
                .insert(Stack::new(StackKind::Stack))
                // Droppable area extending to the bottom of the screen
                .insert(Droppable {zone: Area::new(pos.x - CARD_WIDTH / 2.0, 0.0, CARD_WIDTH, pos.y + CARD_HEIGHT)})
                .id()
        ]
    }).collect();

    for stack_index in 0..7 {
        let next_card = deck.pop().unwrap();

        let y = if stacks[stack_index].len() == 1 {
            0.0
        } else {
            -CARD_STACK_SPACE
        };
        let new = commands.spawn_bundle(SpriteSheetBundle {
                    texture_atlas: card_texture.0.clone(),
                    transform: Transform::from_xyz(0.0, y, 1.0),
                    ..Default::default()
                })
                .insert(next_card)
                .insert(CardFace::Up)
                .insert(Clickable::default())
                .insert(Draggable)
                .id();
        commands.entity(*stacks[stack_index].last().unwrap()).add_child(new);
        stacks[stack_index].push(new);

        for next_stack in (stack_index + 1)..7 {
            let card = deck.pop().unwrap();
            let y = if stacks[next_stack].len() == 1 {
                0.0
            } else {
                -CARD_STACK_SPACE
            };
            let new = commands.spawn_bundle(SpriteSheetBundle {
                        texture_atlas: card_texture.0.clone(),
                        transform: Transform::from_xyz(0.0, y, 1.0),
                        ..Default::default()
                    })
                    .insert(card)
                    .insert(CardFace::Down)
                    .insert(Clickable::default())
                    .id();
            commands.entity(*stacks[next_stack].last().unwrap()).add_child(new);
            stacks[next_stack].push(new);
        }
    }

    let deck_pos = Vec2::new((CARD_WIDTH / 2.0) + 50.0, window.height - 25.0 - (CARD_HEIGHT / 2.0));
    commands.spawn_bundle(SpriteSheetBundle {
            texture_atlas: card_texture.0.clone(),
            transform: Transform::from_xyz((CARD_WIDTH / 2.0) + 50.0, window.height - 25.0 - (CARD_HEIGHT / 2.0), 1.0),
            sprite: TextureAtlasSprite {
                index: BACK_BLUE,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Deck {cards: deck})
        .insert(Clickable::at(deck_pos))
        .with_children(|parent| {
            // Empty space below the deck
            parent.spawn_bundle(SpriteSheetBundle {
                texture_atlas: card_texture.0.clone(),
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                sprite: TextureAtlasSprite {
                    index: EMPTY_SPACE,
                    ..Default::default()
                },
                ..Default::default()
            });
        });

    commands.spawn_bundle(SpriteSheetBundle {
            transform: Transform::from_xyz((CARD_WIDTH / 2.0) + 50.0 + 175.0, window.height - 25.0 - (CARD_HEIGHT / 2.0), 1.0),
            texture_atlas: card_texture.0.clone(),
            sprite: TextureAtlasSprite {
                index: EMPTY_SPACE,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(DiscardPile::default());

    for suit in [Suit::Spades, Suit::Clubs, Suit::Hearts, Suit::Diamonds] {
        let stack_x = (deck_pos.x + (175.0 * 3.0)) + (175.0 * suit.row() as f32);
        let stack_pos = Vec2::new(stack_x, deck_pos.y);
        // TODO: Add texture for suits
        commands.spawn_bundle(SpriteSheetBundle {
            texture_atlas: card_texture.0.clone(),
            sprite: TextureAtlasSprite {
                index: FINAL_STACKS + suit.row(),
                ..Default::default()
            },
            transform: Transform::from_xyz(stack_pos.x, stack_pos.y, 0.0),
            ..Default::default()
        })
        .insert(Stack::new(StackKind::Ordered(suit)))
        .insert(Droppable {zone: Area::new(stack_pos.x - CARD_WIDTH / 2.0, stack_pos.y - CARD_HEIGHT / 2.0, CARD_WIDTH, CARD_HEIGHT)});
    }

    for mut style in reset_menu.iter_mut() {
        style.display = Display::Flex;
    }

    game_state.set(GameState::Playing).unwrap();
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>, mut reset_menu: Query<&mut Style, With<ResetMenuRoot>>) {
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

fn auto_win_system(mut game_state: ResMut<State<GameState>>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        game_state.set(GameState::Won).unwrap()
    }
}


fn show_menu(mut menu_root: Query<&mut Style, With<MenuRoot>>) {
    for mut style in menu_root.iter_mut() {
        style.display = Display::Flex;
    }
}

fn hide_menu(mut menu_root: Query<&mut Style, With<MenuRoot>>) {
    for mut style in menu_root.iter_mut() {
        style.display = Display::None;
    }
}

fn main_menu(
    mut draw_mode: ResMut<DrawMode>,
    mut game_state: ResMut<State<GameState>>,
    interaction_query: Query<(&MenuButton, &Interaction), Changed<Interaction>>,
    mut q_buttons: Query<(&MenuButton, &mut UiColor)>,
) {
    for (button, interaction) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => {
                match button {
                    MenuButton::Play => {
                        game_state.set(GameState::Shuffle).unwrap();
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

fn win_screen(
    mut commands: Commands,
    mut game_state: ResMut<State<GameState>>,
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
    win_text: Query<Entity, With<WinText>>,

) {
    for (entity, interaction) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => {
                game_state.set(GameState::Menu).unwrap();
                commands.entity(entity).despawn_recursive();
                for e in win_text.iter() {
                    commands.entity(e).despawn_recursive();
                }
            },
            _ => {},
        }
    }
}

fn reset_game_button(
    mut game_state: ResMut<State<GameState>>,
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
                game_state.set(GameState::Shuffle).unwrap();
            },
            _ => {},
        }
    }
}

fn debug_duplicate_children(keys: Res<Input<KeyCode>>, q_children: Query<(Entity, &Children)>) {
    if keys.just_pressed(KeyCode::Q) {
        info!("Checking for duplicate parents...");
        let mut all_children: HashMap<Entity, Vec<Entity>> = HashMap::new();
        for (parent, children) in q_children.iter() {
            for child in children.iter() {
                if let Some(parents) = all_children.get_mut(child) {
                    parents.push(parent);
                } else {
                    all_children.insert(*child, vec![parent]);
                }
            }
        }
        for (child, parents) in all_children.iter() {
            if parents.len() > 1 {
                error!("{:?} has multiple parents: {:?}", child, parents);
            }
        }
    }
}

fn spawn_win_screen(
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

fn deck_update_system(mut decks: Query<(&Deck, &mut Visibility), Changed<Deck>>) {
    for (deck, mut visible) in decks.iter_mut() {
        visible.is_visible = deck.cards.len() > 0;
    }
}

fn discard_update_system(
    q_discard: Query<Entity, With<DiscardPile>>,
    q_children: Query<&Children>,
    mut q_transform: Query<&mut Transform>,
    q_interaction: Query<&MouseInteraction>,
) {
    let discard = q_discard.single();
    let first_child = q_children.get(discard).ok().map(|children| children.first()).flatten().cloned();
    let mut children = Vec::new();
    walk_children(first_child, &q_children, &mut |child| children.push(child));

    for (i, child) in children.iter().rev().enumerate() {
        if q_interaction.get(*child).ok().map(|interaction| interaction.is_dragging()).unwrap_or(false) {
            continue
        }
        if i < 2 && Some(*child) != first_child {
            let mut transform = q_transform.get_mut(*child).unwrap();
            *transform.translation = *Vec3::new(CARD_STACK_SPACE, 0.0, 1.0);
        } else {
            let mut transform = q_transform.get_mut(*child).unwrap();
            *transform.translation = *Vec3::new(0.0, 0.0, 1.0);
        }
    }
}

fn win_check_system(mut game_state: ResMut<State<GameState>>, q_stacks: Query<(Entity, &Stack)>, q_card: Query<&Card>, q_children: Query<&Children>) {
    let mut completed = 0;
    for (stack_entity, _) in q_stacks.iter().filter(|(_entity, stack)| match stack.kind {StackKind::Ordered(_) => true, _ => false}) {
        let top = top_entity(stack_entity, &q_children);
        if let Ok(card) = q_card.get(top) {
            if card.kind == CardKind::King {
                completed += 1;
            }
        }
    }
    if completed == 4 {
        info!("Game Won!");
        game_state.set(GameState::Won).unwrap();
    }

    // TODO: Check if there are no cards in the deck/discard and all cards are face up we can trivially auto solve it
    // Just change/push the state to a solve state and move the cards one at a time with an animation at like 1 every 50ms
}

fn update_click_timers(mut commands: Commands, time: Res<Time>, mut timers: Query<(Entity, &mut WasClicked)>) {
    for (entity, mut timer) in timers.iter_mut() {
        if timer.0.tick(time.delta()).finished() {
            commands.entity(entity).remove::<WasClicked>();
        }
    }
}

fn card_texture_update_system(mut cards: Query<(&Card, &CardFace, &mut TextureAtlasSprite), Changed<CardFace>>) {
    for (card, face, mut sprite) in cards.iter_mut() {
        match face {
            CardFace::Up => {
                sprite.index = card.texture_index();
            },
            CardFace::Down => {
                sprite.index = BACK_BLUE;
            },
        }
    }
}

fn clickable_bounds_update_system(
    mut clickables: Query<(&mut Clickable, &GlobalTransform), Changed<GlobalTransform>>,
) {
    for (mut clickable, global_transform) in clickables.iter_mut() {
        clickable.zone.pos = Vec2::new(global_transform.translation.x, global_transform.translation.y) - (clickable.zone.size / 2.0);
    }
}

fn mouse_interaction_system(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    mut mouse_movements: EventReader<MouseMotion>,
    windows: Res<Windows>,
    q_clickable: Query<(Entity, &GlobalTransform, &Clickable)>,
    mut q_interaction: Query<(Entity, &mut MouseInteraction, &GlobalTransform)>,
    mut q_transform: Query<&mut Transform>,
    q_draggable: Query<&Draggable>,
    mut ev_released: EventWriter<Released>,
    mut ev_dropped: EventWriter<Dropped>,
) {
    let window = windows.get_primary().unwrap();
    let mouse_position = match window.cursor_position() {
        Some(p) => p,
        None => return
    };
    if mouse.just_pressed(MouseButton::Left) {
        let mut hovered = q_interaction
                                .iter_mut()
                                .filter(|(_, interaction, _)| interaction.is_hovered())
                                .map(|(_, interaction, gtransform)| (interaction, gtransform))
                                .collect::<Vec<(Mut<'_, MouseInteraction>, &GlobalTransform)>>();
        hovered.sort_by(|(_, a), (_, b)| a.translation.z.partial_cmp(&b.translation.z).unwrap_or(std::cmp::Ordering::Equal));
        if let Some((interaction, pos)) = hovered.last_mut() {
            let offset = Vec2::new(pos.translation.x, pos.translation.y) - mouse_position;
            // Borrow checker wasn't happy unless I used as_mut() and I'm not sure why
            // The borrow out of the Vec seems to mess with it?
            *interaction.as_mut() = MouseInteraction::Clicked(offset);
        }
    } else if mouse.just_released(MouseButton::Left) {
        // Release/Drop
        for (entity, interaction, _) in q_interaction.iter() {
            match *interaction {
                // We have not moved
                // Change these to Events?
                MouseInteraction::Clicked(offset) => {
                    ev_released.send(Released(entity, offset));
                    commands.entity(entity).remove::<MouseInteraction>();
                },
                MouseInteraction::Dragging {local_start_pos, ..} => {
                    ev_dropped.send(Dropped(entity, local_start_pos, mouse_position));
                    commands.entity(entity).remove::<MouseInteraction>();
                },
                _ => {}
            };
        }
    } else {
        // Remove MouseInteraction from anything that is Dropped or Released
        // This is done first because we would immediately remove the MouseInteraction if it was done below
        // We only care if there was any motion since we are not using the delta
        // Update Hovers
        // We could only do this when the mouse moves, but we would need to re-add the
        // Hovered state to any components that the mouse is still covering otherwise
        // the mouse needs to be moved before the entity can be clicked again
        // Could update hoverables in the mouse.just_released block as well...
        for (entity, _, clickable) in q_clickable.iter() {
            if clickable.zone.contains(mouse_position) {
                // If it does not have an interaction, add Hover
                if q_interaction.get(entity).is_err() {
                    commands.entity(entity).insert(MouseInteraction::Hovered);
                }
            } else {
                // This makes it hard to know when something stops being hovered, but don't really care about that though
                // We only want to remove the component if the entity is not being dragged. Other wise moving the mouse very
                // quickly can cause the entity to stop being dragged
                if q_interaction.get(entity).ok().map(|(_, interaction, _)| !interaction.is_dragging()).unwrap_or(false) {
                    commands.entity(entity).remove::<MouseInteraction>();
                }
            }
        }
        if let Some(_) = mouse_movements.iter().next() {
            // Dragging
            if mouse.pressed(MouseButton::Left) {
                for (entity, mut interaction, gpos) in q_interaction.iter_mut().filter(|(_, interaction, _)| interaction.is_dragging() || interaction.is_clicked()) {
                    match *interaction {
                        MouseInteraction::Clicked(_) => {
                            if q_draggable.get(entity).is_ok() {
                                *interaction = MouseInteraction::Dragging {
                                    global_start_pos: gpos.translation,
                                    local_start_pos: q_transform.get_mut(entity).unwrap().translation,
                                    start_mouse: mouse_position,
                                };
                            }
                        },
                        MouseInteraction::Dragging {global_start_pos: _, local_start_pos, start_mouse} => {
                            let mut transform = q_transform.get_mut(entity).unwrap();

                            let mouse_delta = mouse_position - start_mouse;
                            let pos = local_start_pos + Vec3::new(mouse_delta.x, mouse_delta.y, 500.0);
                            *transform.translation = *pos;
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}

fn click_system(
    mut commands: Commands,
    draw_mode: ResMut<DrawMode>,
    mut ev_released: EventReader<Released>,
    card_texture: Res<CardsTextureHandle>,
    q_card: Query<(&Card, &CardFace, Option<&WasClicked>)>,
    mut q_deck: Query<&mut Deck>,
    q_parent: Query<&Parent>,
    q_children: Query<&Children>,
    q_stacks: Query<(Entity, &Stack)>,
    q_discard: Query<Entity, With<DiscardPile>>,
    q_gtransform: Query<&GlobalTransform>,
    mut q_transform: Query<&mut Transform>,
) {
    for Released(entity, _offset) in ev_released.iter() {
        if let Ok(mut deck) = q_deck.get_mut(*entity) {
            if deck.cards.len() == 0 {
                let discard = q_discard.single();
                let top = top_entity(discard, &q_children);
                let top = if top == discard {
                    None
                } else {
                    Some(top)
                };
                walk(top, &q_parent, &mut |entity| {
                    if let Ok((card, _, _)) = q_card.get(entity) {
                        deck.cards.push(*card);
                    }
                });
                if let Ok(children) = q_children.get(discard) {
                    for child in children.iter() {
                        commands.entity(*child).despawn_recursive();
                    }
                }
            } else {
                let discard = q_discard.single();
                let discard_positon = q_gtransform.get(discard).unwrap();
                let click_position = Vec2::new(discard_positon.translation.x, discard_positon.translation.y);
                let mut top = top_entity(discard, &q_children);
                if top != discard {
                    // Make the current top card undraggable
                    commands.entity(top).remove::<Draggable>();
                }
                for _ in 0..draw_mode.num() {
                    match deck.cards.pop() {
                        Some(card) => {
                            let new = commands.spawn_bundle(SpriteSheetBundle {
                                                    texture_atlas: card_texture.0.clone(),
                                                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                                                    ..Default::default()
                                                })
                                                .insert(card)
                                                .insert(CardFace::Up)
                                                .insert(Clickable::at(click_position))
                                                .id();
                            commands.entity(top).add_child(new);
                            top = new;
                        },
                        None => {
                            break
                        }
                    }
                }
                // Make the current new top card draggable
                commands.entity(top).insert(Draggable);
            }
            continue
        }

        if let Ok((card, face, maybe_clicked)) = q_card.get(*entity) {
            match face {
                CardFace::Up => {
                    let has_children = q_children.get(*entity).ok().map(|c| !c.is_empty()).unwrap_or(false);
                    if maybe_clicked.is_some() && !has_children {
                        // double click
                        commands.entity(*entity).remove::<WasClicked>();
                        if let Some((stack_entity, stack)) = q_stacks.iter().filter(|(_, stack)| stack.kind == StackKind::Ordered(card.suit)).nth(0) {
                            let target = top_entity(stack_entity, &q_children);
                            let target_card = q_card.get(target).ok().map(|(c, _, _)| c);
                            if stack.can_stack(target_card, *card, false) {
                                if let Ok(parent) = q_parent.get(*entity) {
                                    commands.entity(parent.0).remove_children(&[*entity]);
                                    if let Ok((_, face, _)) = q_card.get(parent.0) {
                                        commands.entity(parent.0).insert(Draggable);
                                        if face == &CardFace::Down {
                                            commands.entity(parent.0)
                                                        .insert(CardFace::Up);
                                        }
                                    }
                                }
                                commands.entity(target).add_child(*entity);
                                let mut transform = q_transform.get_mut(*entity).unwrap();
                                *transform.translation = *Vec3::new(0.0, 0.0, 1.0);
                                break
                            }
                        }
                    } else {
                        commands.entity(*entity).insert(WasClicked(Timer::from_seconds(0.5, false)));
                    }
                },
                CardFace::Down => {
                    // This shouldn't ever happen since its done autoamtically
                    let has_children = q_children.get(*entity).ok().map(|c| !c.is_empty()).unwrap_or(false);
                    if !has_children {
                        commands.entity(*entity)
                                    .insert(CardFace::Up)
                                    .insert(Draggable);
                    }
                }
            }
        }
    }
}

fn drop_system(
    mut commands: Commands,
    mut ev_dropped: EventReader<Dropped>,
    q_droppable: Query<(Entity, &Droppable)>,
    q_children: Query<&Children>,
    q_parent: Query<&Parent>,
    q_card: Query<&Card>,
    q_card_face: Query<&CardFace>,
    q_stack: Query<&Stack>,
    mut q_transform: Query<&mut Transform>,
) {
    for Dropped(dropped, local_start_pos, mouse_position) in ev_dropped.iter() {
        let mut was_dropped = false;
        for (droppable_entity, droppable) in q_droppable.iter() {
            if droppable.zone.contains(*mouse_position) {
                // Find the bottom of the drop stack
                let top_entity = top_entity(droppable_entity, &q_children);
                // If the entity is not a card we will get None
                let top_card = q_card.get(top_entity).ok();
                let stack = q_stack.get(droppable_entity).unwrap();
                // If we are not dragging a card this will fail
                let dropped_card = q_card.get(*dropped).unwrap();
                let has_children = q_children.get(*dropped).ok().map(|c| !c.is_empty()).unwrap_or(false);
                if stack.can_stack(top_card, *dropped_card, has_children) {
                    if let Ok(parent) = q_parent.get(*dropped) {
                        commands.entity(parent.0).remove_children(&[*dropped]);
                        if let Ok(face) = q_card_face.get(parent.0) {
                            // Make the new stack top draggable
                            commands.entity(parent.0).insert(Draggable);
                            if face == &CardFace::Down {
                                commands.entity(parent.0).insert(CardFace::Up);
                            }
                        }
                    }
                    commands.entity(top_entity).add_child(*dropped);
                    let mut transform = q_transform.get_mut(*dropped).unwrap();
                    *transform.translation = *match stack.kind {
                        StackKind::Stack => {
                            if top_card.is_some() {
                                Vec3::new(0.0, -CARD_STACK_SPACE, 1.0)
                            } else {
                                Vec3::new(0.0, 0.0, 1.0)
                            }
                        },
                        StackKind::Ordered(_) => {
                            Vec3::new(0.0, 0.0, 1.0)
                        }
                    };
                    was_dropped = true;
                    break
                }
            }
        }
        if !was_dropped {
            // Move back to the old position
            let mut transform = q_transform.get_mut(*dropped).unwrap();
            *transform.translation = **local_start_pos;
        }
    }
}


fn walk(mut node: Option<Entity>, query: &Query<&Parent>, func: &mut dyn FnMut(Entity)) {
    while let Some(entity) = node {
        func(entity);
        node = if let Ok(parent) = query.get(entity) {
            Some(parent.0)
        } else {
            None
        }
    }
}

fn walk_children(mut node: Option<Entity>, query: &Query<&Children>, func: &mut dyn FnMut(Entity)) {
    while let Some(entity) = node {
        func(entity);
        node = if let Ok(children) = query.get(entity) {
            children.first().cloned()
        } else {
            None
        };
    }
}

fn top_entity(mut node: Entity, children: &Query<&Children>) -> Entity {
    while let Some(entity) = children.get(node).ok().map(|c| c.first()).flatten() {
        node = *entity
    }
    node
}
