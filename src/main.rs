use std::collections::HashMap;
use bevy::prelude::*;
use bevy::log::{LogSettings, Level};
use bevy::render::camera::{WindowOrigin, ScalingMode};
use bevy::render::view::Visibility;
use bevy::input::{keyboard::KeyCode, Input, mouse::MouseMotion};
use bevy::ui::Display;
// use bevy_easings::*;
use rand::prelude::*;

const BACK_GREEN: usize = 5 * 13;
const BACK_BLUE: usize = 6 * 13;
const BACK_RED: usize = 7 * 13;
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
#[derive(Debug, Clone, Copy, Component)]
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
    enabled: bool,
}

impl Default for Clickable {
    fn default() -> Self {
        Self {
            zone: Area {pos: Vec2::new(0.0, 0.0), size: Vec2::new(CARD_WIDTH, CARD_HEIGHT)},
            enabled: true,
        }
    }
}

impl Clickable {
    fn at(pos: Vec2) -> Self {
        Self {
            zone: Area {pos: pos, size: Vec2::new(CARD_WIDTH, CARD_HEIGHT)},
            ..Default::default()
        }
    }

    fn contains(&self, pos: Vec2) -> bool {
        self.zone.contains(pos)
    }
}


#[derive(Component)]
struct WinText;

#[derive(Component)]
struct ResetButton;

// #[derive(Debug)]
// enum DrawMode {
//     Draw1,
//     Draw3,
// }

#[derive(Debug)]
struct MoveCard {
    target: Entity,
    to: Entity,
}

impl MoveCard {
    fn new(target: Entity, to: Entity) -> Self {
        Self {
            target,
            to
        }
    }
}

/// State machine
///   Hovered -> Clicked -> Released
///                 |
///                 +-> Dragging -> Dropped
#[derive(Debug, Clone, Component)]
enum MouseInteraction {
    /// The mouse is over this entity
    Hovered,
    /// The left mouse is held down on this entity
    Clicked(Vec2),
    /// The left mouse was released while still hovered
    Released(Vec2),
    /// Entity is being dragged
    Dragging(Vec2),
    /// The mouse was released while the entity was being dragged.
    /// The Interaction component will be removed the frame following this one
    Dropped(Vec2),
}

#[derive(Debug, Clone)]
struct Clicked(Entity, Vec2);

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
struct Dropped(Entity, Vec2);

/// This exists because the entity inside PreviousParent is not accessible
#[derive(Debug, Component)]
struct PrevParent(Entity, Transform);

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
    // let mut log_settings = LogSettings::default();
    // log_settings.level = Level::DEBUG;
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
        // .insert_resource(log_settings)
        .add_state(GameState::Menu)
        .add_event::<Clicked>()
        .add_event::<Dropped>()
        .add_event::<MoveCard>()
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::on_enter(GameState::Menu)
                .with_system(setup_menu)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Menu)
                .with_system(main_menu)
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Shuffle)
                .with_system(reset_cards)
        )
        .add_system_set_to_stage(
            // Run these before update so we are absolutely sure commands are completed early
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(card_move_system)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(click_system.label("click"))
                .with_system(update_click_timers.before("click"))
                .with_system(clicked_system.after("click"))
                .with_system(drag_system.label("drag").after("click"))
                .with_system(drop_system.label("drop").before("click"))
                .with_system(win_check_system.after("click"))
                .with_system(card_texture_update_system)
                .with_system(deck_update_system.after("click"))
                .with_system(debug_duplicate_children)
                .with_system(fixup_children_system)
                .with_system(reset_game_button)
        )
        .add_system_set_to_stage(
            // Run the stack cleanup code in postupdate, otherwise there will be a 1 frame
            // delay causing cards to jump around as they are dragged/dropped/double clicked
            // Using system labels and .after() etc it is clear that there are multiple
            // systems updating the state of dragged cards
            CoreStage::PostUpdate,
            // Change these to just run off of some events or changes
            // so that it doesn't run all the time
            SystemSet::new()
                .with_system(discard_pile_update_system.system())
                .with_system(card_stacks_update_system.system())
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Won)
                .with_system(spawn_win_text.system())
        )
        .add_system_set(
            SystemSet::on_update(GameState::Won)
                .with_system(won_screen.system())
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
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(50.0), Val::Px(20.0)),
                display: Display::None,
                margin: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(5.0),
                    top: Val::Auto,
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
                    "Reset",
                    TextStyle {
                        font: font_handle.clone(),
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            })
            .insert(ResetButton);
        })
        .insert(ResetButton);
}

fn reset_cards(
    mut commands: Commands,
    mut game_state: ResMut<State<GameState>>,
    card_texture: Res<CardsTextureHandle>,
    window: Res<WindowDescriptor>,
    cleanup: Query<Entity, Or<(With<Card>, With<Stack>, With<Deck>, With<DiscardPile>)>>,
    mut reset_button: Query<&mut Style, With<ResetButton>>
) {
    for entity in cleanup.iter() {
        commands.entity(entity).despawn_recursive();
    }

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

    // FIXME: Math to make this better
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
                // .insert(Clickable::at(pos - Vec2::new(CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0)))
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
        .insert(Clickable::at(deck_pos - Vec2::new(CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0)))
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

    for mut style in reset_button.iter_mut() {
        style.display = Display::Flex;
    }

    game_state.set(GameState::Playing).unwrap();
}

fn setup_menu(mut commands: Commands, font: Res<FontHandle>, mut reset_button: Query<&mut Style, With<ResetButton>>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
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
                    "Play",
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

    for mut style in reset_button.iter_mut() {
        style.display = Display::None;
    }
}

fn main_menu(
    mut commands: Commands,
    mut game_state: ResMut<State<GameState>>,
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>
) {
    for (entity, interaction) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => {
                game_state.set(GameState::Shuffle).unwrap();
                commands.entity(entity).despawn_recursive();
            },
            _ => {},
        }
    }
}

fn won_screen(
    mut commands: Commands,
    mut game_state: ResMut<State<GameState>>,
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
    win_text: Query<Entity, With<WinText>>,

) {
    for (entity, interaction) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => {
                game_state.set(GameState::Shuffle).unwrap();
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
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<ResetButton>)>,
) {
    for interaction in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => {
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

fn spawn_win_text(
    mut commands: Commands,
    windows: Res<Windows>,
    font: Res<FontHandle>,
    mut reset_button: Query<&mut Style, With<ResetButton>>,
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

    for mut style in reset_button.iter_mut() {
        style.display = Display::None;
    }
}

fn deck_update_system(mut decks: Query<(&Deck, &mut Visibility), Changed<Deck>>) {
    for (deck, mut visible) in decks.iter_mut() {
        visible.is_visible = deck.cards.len() > 0;
    }
}

fn discard_pile_update_system(
    discard_pile: Query<Entity, With<DiscardPile>>,
    children: Query<&Children>,
    mut card: Query<(&mut Visibility, &mut Clickable, &mut Transform, &GlobalTransform)>
) {
    let discard = match discard_pile.get_single() {
        Ok(e) => e,
        Err(_) => return
    };
    let mut top = None;
    let first_child = children.get(discard).ok().map(|c| c.first()).flatten().cloned();

    walk_children(first_child, &children, &mut |child| {
        if let Ok((mut visible, mut clickable, mut transform, global_transform)) = card.get_mut(child) {
            visible.is_visible = false;
            transform.translation = Vec3::new(0.0, 0.0, 1.0);
            clickable.enabled = false;
            clickable.zone.pos = Vec2::new(global_transform.translation.x, global_transform.translation.y) - Vec2::new(CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0);
        }
        top = Some(child);
    });

    if let Some(top) = top {
        if let Ok((mut visible, mut clickable, _, _)) = card.get_mut(top) {
            visible.is_visible = true;
            clickable.enabled = true;
        }
    }
}

fn fixup_children_system(mut commands: Commands, mut q_children: Query<(Entity, &mut Children)>, q_parent: Query<&Parent>) {
    for (parent, children) in q_children.iter_mut() {
        let mut to_remove = Vec::new();
        for child in children.iter() {
            if let Ok(actual_parent) = q_parent.get(*child) {
                if actual_parent.0 != parent {
                    to_remove.push(*child);
                }
            } else {
                // No longer has a parent
                to_remove.push(*child);
            }
        }
        if to_remove.len() > 0 {
            error!("{:?} has incorrect children = {:?}; invalid = {:?}", parent, children, to_remove);
            commands.entity(parent).remove_children(&to_remove);
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

// This could be a system that only runs when something changes via a Changed<CardFace> event
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

fn card_stacks_update_system(
    stacks: Query<(Entity, &Stack)>,
    mut clickable: Query<(Option<&Children>, &mut Clickable, &mut Transform, &GlobalTransform), With<Card>>,
    children: Query<&Children>,
) {
    for (entity, stack) in stacks.iter() {
        let mut depth = 0;
        let stack_transform = match stack.kind {
            StackKind::Ordered(_) => Vec3::new(0.0, 0.0, 1.0),
            StackKind::Stack => Vec3::new(0.0, -CARD_STACK_SPACE, 1.0),
        };
        let first_child = children.get(entity).ok().map(|c| c.first()).flatten().cloned();

        walk_children(first_child, &children, &mut |child| {
            if let Ok((maybe_children, mut clickable, mut transform, global_transform)) = clickable.get_mut(child) {
                if depth == 0 {
                    *transform.translation = *Vec3::new(0.0, 0.0, 1.0);
                } else {
                    *transform.translation = *stack_transform.clone();
                }
                clickable.zone.pos = Vec2::new(global_transform.translation.x, global_transform.translation.y) - Vec2::new(CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0);
                let has_children = maybe_children.map(|c| !c.is_empty()).unwrap_or(false);
                if has_children {
                    if stack.kind == StackKind::Stack {
                        clickable.zone.size = Vec2::new(CARD_WIDTH, CARD_STACK_SPACE);
                        // origin is bottom left so we need to move the box up when we reduce the size of it
                        clickable.zone.pos.y += CARD_HEIGHT - CARD_STACK_SPACE;
                        clickable.enabled = true;
                    } else {
                        clickable.zone.size = Vec2::new(CARD_WIDTH, CARD_HEIGHT);
                        clickable.enabled = false;
                    }
                } else {
                    clickable.zone.size = Vec2::new(CARD_WIDTH, CARD_HEIGHT);
                    clickable.enabled = true;
                }
            }
            depth += 1;
        });
    }
}

fn click_system(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    q_clickable: Query<(Entity, &GlobalTransform, &Clickable)>,
    q_clicked: Query<(Entity, &LeftClicked)>,
    q_dragging: Query<Entity, With<Dragging>>,
    mut clicked_events: EventWriter<Clicked>,
    mut dropped_events: EventWriter<Dropped>,
) {
    // Clicking very quickly can cause use to be dragging a card and also click on the card below it
    // flipping that card while still holding the card. The card doesn't even get dropped either.
    // No idea whats happening and the best way to fix it. Probably use resources instead
    // of components becuase the resources will be updated immediately rather than at the end like components.
    let window = windows.get_primary().unwrap();
    if let Some(mouse_position) = window.cursor_position() {
        if mouse.just_released(MouseButton::Left) {
            // FIXME: releasing the left mouse button outside the window will leave it clicked
            if let Ok((entity, clicked)) = q_clicked.get_single() {
                if let Ok((_, _, bounds)) = q_clickable.get(entity) {
                    if bounds.enabled && bounds.contains(mouse_position) {
                        info!("Clicked {:?} {:?} {:?}", entity, clicked.0, bounds);
                        clicked_events.send(Clicked(entity, clicked.0));
                    }
                }
                commands.entity(entity).remove::<LeftClicked>();
            } else {
                for entity in q_dragging.iter() {
                    info!("Dropped {:?} at {:?}", entity, mouse_position);
                    commands.entity(entity).remove::<Dragging>();
                    dropped_events.send(Dropped(entity, mouse_position));
                }
            }
        } else if mouse.just_pressed(MouseButton::Left) {
            info!("mouse pos = {:?}", mouse_position);
            for (entity, transform, bounds) in q_clickable.iter() {
                if bounds.contains(mouse_position) {
                    info!("Pressed {:?} {:?} {:?}", entity, mouse_position, bounds);
                    if bounds.enabled {
                        info!("  Enabled!");
                        let pos = Vec2::new(transform.translation.x, transform.translation.y);
                        // Change this from resource to a component on the clicked entity
                        // commands.insert_resource(Clicked(entity, mouse_position - pos));
                        commands.entity(entity).insert(LeftClicked(mouse_position - pos));
                        // Ensure we don't click on multiple items
                        break
                    } else {
                        info!("  Not Enabled!");
                    }
                }
            }
        } else if !mouse.pressed(MouseButton::Left) {
            for entity in q_dragging.iter() {
                    error!("{:?} was being dragged despite the mouse not being pressed. Dropping at {:?}", entity, mouse_position);
                    commands.entity(entity).remove::<Dragging>();
                    dropped_events.send(Dropped(entity, mouse_position));
                }
        }
    }
}

fn drag_system(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    q_clicked: Query<(Entity, &LeftClicked)>,
    windows: Res<Windows>,
    mut q_dragging: Query<(&Dragging, &mut Transform)>,
    q_draggable: Query<Entity, With<Draggable>>,
    q_parent: Query<&Parent>,
    mut mouse_movements: EventReader<MouseMotion>,
    mut q_transforms: Query<(&mut Transform, Option<&GlobalTransform>), Without<Dragging>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        // This is needed to prevent a click while moving in the same frame causing
        // an entity to be in the children list of multiple entities
        // Maybe? Needs confirmation that this fixes it...
        // The bug is related to double clicking so why does this matter?
        // This is not enough likely becuase its more related to double clicking
        // that weird race conditions between the click and drag system...
        return
    }
    let window = windows.get_primary().unwrap();
    if let Some(mouse_position) = window.cursor_position() {
        let mut delta = Vec2::new(0.0, 0.0);
        for motion in mouse_movements.iter() {
            delta += Vec2::new(motion.delta.x, motion.delta.y);
            for (dragging, mut transform) in q_dragging.iter_mut() {
                let position = mouse_position - dragging.0;
                transform.translation.x = position.x;
                transform.translation.y = position.y;
            }
        }
        if delta.length() > 0.0 {
            if let Ok((clicked, LeftClicked(click_offset))) = q_clicked.get_single() {
                if q_draggable.get(clicked).is_ok() {
                    info!("Started dragging {:?} {:?}", clicked, delta.length());
                    commands.entity(clicked).insert(Dragging(*click_offset));
                    commands.entity(clicked).remove::<LeftClicked>();

                    // This can fail without Option<&GlobalTransform> and I have no idea why
                    // Its long after everything has been rendered. Its possibly because
                    // the global transform is in the middle of being changed by another system?
                    if let Ok((mut transform, global_transform)) = q_transforms.get_mut(clicked) {
                        if let Ok(parent) = q_parent.get(clicked) {
                            info!("  Removing parent {:?} from {:?}", parent.0, clicked);
                            commands.entity(parent.0).remove_children(&[clicked]);
                            // commands.entity(clicked).remove::<Parent>();
                            // commands.entity(parent.0).remove::<Children>();
                            // We cannot access the entity from the automatically added PreviousParent component
                            // Might consider adding the "previous parent" as an Option<Entity> to the Dragging component
                            commands.entity(clicked).insert(PrevParent(parent.0, transform.clone()));
                        }

                        if let Some(gtransform) = global_transform {
                            *transform.translation = *gtransform.translation;
                            transform.translation.z = 500.0;
                        } else {
                            info!("{:?} doesn't have a global transform?", clicked);
                        }
                    } else {
                        error!("{:?} was missing its transforms????", clicked);
                    }
                }
            }
        }
    }
}

fn drop_system(
    mut commands: Commands,
    q_card: Query<&Card>,
    windows: Res<Windows>,
    mut dropped_entities: EventReader<Dropped>,
    _move_cards: EventWriter<MoveCard>,
    q_droppable: Query<(Entity, &Droppable)>,
    q_stack: Query<&Stack>,
    q_previous_parent: Query<&PrevParent>,
    q_children: Query<&Children>,
    mut q_transform: Query<&mut Transform>,
) {
    let window = windows.get_primary().unwrap();
    if let Some(mouse_position) = window.cursor_position() {
        for dropped in dropped_entities.iter() {
            let mut was_dropped = false;
            for (droppable_entity, droppable) in q_droppable.iter() {
                if droppable.zone.contains(mouse_position) {
                    // Find the bottom of the drop stack
                    let top_entity = top_entity(droppable_entity, &q_children);
                    // If the entity is not a card we will get None
                    let top_card = q_card.get(top_entity).ok();
                    let stack = q_stack.get(droppable_entity).unwrap();
                    let dropped_card = q_card.get(dropped.0);
                    // Weren't even dragging a card...
                    // if dropped_card.is_err() {
                    //     break
                    // }
                    let has_children = q_children.get(dropped.0).ok().map(|c| !c.is_empty()).unwrap_or(false);
                    info!("stack {:?} {:?}; top card = {:?} {:?}; dropped card {:?}; has children? {:?}", droppable_entity, stack, top_entity, top_card, dropped_card, has_children);
                    if stack.can_stack(top_card, *dropped_card.unwrap(), has_children) {
                        commands.entity(top_entity).add_child(dropped.0);
                        // Cannot use MoveCard because I have not make it recursive yet
                        // move_cards.send(MoveCard::new(dropped.0, top_entity));
                        // commands.entity(top_entity).insert(Children::with(&[dropped.0]));
                        // commands.entity(dropped.0).insert(Parent(top_entity));
                        // let mut transform = q_transform.get_mut(dropped.0).unwrap();
                        // *transform = stack.transform();
                        was_dropped = true;
                        break
                    }
                }
            }
            info!("was_dropped? {:?}", was_dropped);
            if !was_dropped {
                if let Ok(PrevParent(parent, prev_transform)) = q_previous_parent.get(dropped.0) {
                    // Cannot use PreviousParent, need to add a different component
                    // Reset the translation to the stack offset

                    // This can fail sometimes because the parent is apparently deleted?
                    commands.entity(*parent).add_child(dropped.0);
                    // commands.entity(*parent).insert(Children::with(&[dropped.0]));
                    // commands.entity(dropped.0).insert(Parent(*parent));

                    let mut transform = q_transform.get_mut(dropped.0).unwrap();
                    *transform.translation = *prev_transform.translation;
                }
            }
            commands.entity(dropped.0).remove::<PrevParent>();
        }
    } else {
        // Released but the mouse was not on something droppable
        for dropped in dropped_entities.iter() {
            if let Ok(PrevParent(parent, _transform)) = q_previous_parent.get(dropped.0) {
                // Cannot use PreviousParent, need to add a different component
                // Reset the translation to the stack offset
                commands.entity(*parent).add_child(dropped.0);
                // commands.entity(*parent).insert(Children::with(&[dropped.0]));
                // commands.entity(dropped.0).insert(Parent(*parent));
            }
            commands.entity(dropped.0).remove::<PrevParent>();
        }
    }
}

fn clicked_system(
    mut commands: Commands,
    card_texture: Res<CardsTextureHandle>,
    mut ev_clicked: EventReader<Clicked>,
    mut move_cards: EventWriter<MoveCard>,
    q_card: Query<(&Card, &CardFace, Option<&WasClicked>)>,
    mut q_deck: Query<&mut Deck, With<Deck>>,
    q_parent: Query<&Parent>,
    q_children: Query<&Children>,
    q_stacks: Query<(Entity, &Stack)>,
    q_discard: Query<Entity, With<DiscardPile>>,
) {
    for clicked in ev_clicked.iter() {
        if let Ok(mut deck) = q_deck.get_mut(clicked.0) {
            match deck.cards.pop() {
                Some(card) => {
                    let discard = q_discard.single();
                    let top = top_entity(discard, &q_children);
                    let new = commands.spawn_bundle(SpriteSheetBundle {
                                            texture_atlas: card_texture.0.clone(),
                                            transform: Transform::from_xyz(0.0, 0.0, 1.0),
                                            ..Default::default()
                                        })
                                        .insert(card)
                                        .insert(CardFace::Up)
                                        .insert(Clickable::default())
                                        .insert(Draggable)
                                        .id();
                    info!("Attempting to add {:?} to {:?} (discard = {:?})", new, top, discard);
                    commands.entity(top).add_child(new);
                },
                None => {
                    info!("Reset deck");
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
                            // Is despawn_recursive too aggressive?
                            // Could it be causing some of the issues?
                            // I don't see how but I don't understand the issues in the first place so...
                            commands.entity(*child).despawn_recursive();
                        }
                    }
                }
            }
            // Clicked on the deck, no reason to check anything else
            continue
        }

        if let Ok((card, face, clicked_at)) = q_card.get(clicked.0) {
            match face {
                CardFace::Up => {
                    // Only allow double clicking things that are on the top of their stack
                    if q_children.get(clicked.0).ok().map(|c| !c.is_empty()).unwrap_or(false) {
                        return
                    }
                    // As long as the WasClicked component exists the timer is still active
                    if let Some(_was_clicked) = clicked_at {
                        commands.entity(clicked.0).remove::<WasClicked>();
                        if let Some((stack_entity, stack)) = q_stacks.iter().filter(|(_, stack)| stack.kind == StackKind::Ordered(card.suit)).nth(0) {
                            // FIXME: Something is broken here. Sometimes the stack isn't moved properly
                            // and the card below the one that was double clicked becomes unclickable
                            // Sometimes cards can just move around seemingly randomly
                            // Very possibly due to 1 frame delay on dragging and clicking?
                            // Parent/Child handling might be broken?
                            // Possibly related to double clicking a card while its on the completed stack
                            // I really think its just the parent/child stuff not being updated properly
                            // Need to go over my systems first then if they look fine just make up our own
                            // system to fixup the parents/children so the transform
                            // system works properly
                            // Also possibly related to the "clickable" not being disabled properly?
                            // Also possibly just get rid of this whole thing and instead have the stacks be a single
                            // component containing a list of cards
                            // Dragging from a completed stack to normal stack unsuccessfully caused the card to go
                            // back to the completed stack as a non-completed stack, so a later double click fails the
                            // assertion here
                            //
                            // Best idea: If its an issue with remove_children/add_child, change this
                            // to create a "MoveCard(entity, to)" event and run remove_children.
                            // Then we order it so the event processing is _before_ anything that generates
                            // the events so we have delinked the parent. Drag and drop doesn't seem
                            // to cause have issues, so by ensuring the old parent is removed before
                            // a new parent is added it should continue to work.
                            // However, it does seem to happen _more_ often with the discard pile...
                            //
                            // The MoveCard event didn't fix anything, but it does seema bit harder to cause?
                            //
                            // Theory: Double-click -> card ends up on the discard and completed pile _somehow_
                            // but its in the "children" list of both of its parents and only the "parent"
                            // of one of them. Which one is unknown because it will visually move only
                            // to move back after some action is done. Cycling the deck then deletes all
                            // of the entities but because the completed stack still has it has its child
                            // it will
                            //
                            // I think the next thing to try is to change clickable/draggable
                            // to work the same way that the bevy_ui Interaction does
                            // so we can just query for Query<&Interaction, Change<Interaction>>
                            // and the interactions can be Hovered, Clicked, Released, StartDragging, Dragging, Dropped
                            let target = top_entity(stack_entity, &q_children);
                            let target_card = q_card.get(target).ok().map(|(c, _, _)| c);
                            // let target_stack = q_stacks.get(target).unwrap().1;
                            // This assertion catches some cases where things are just off
                            // assert_eq!(target_stack.kind, stack.kind);
                            if stack.can_stack(target_card, *card, false) {
                                if let Ok(parent) = q_parent.get(clicked.0) {
                                    commands.entity(parent.0).remove_children(&[clicked.0]);
                                    // commands.entity(parent.0).remove::<Children>();
                                }
                                move_cards.send(MoveCard::new(clicked.0, target));
                                // commands.entity(target).add_child(clicked.0);
                                // commands.entity(target).insert(Children::with(&[clicked.0]));
                                // // This right here can somehow be the wrong stack...
                                // commands.entity(clicked.0).insert(Parent(target));
                                break
                            }
                        }
                    } else {
                        commands.entity(clicked.0).insert(WasClicked(Timer::from_seconds(0.5, false)));
                    }
                },
                CardFace::Down => {
                    // TODO: Make this automatic
                    // Clicked on a facedown card that has no children
                    info!("Clicked face down card");
                    if q_children.get(clicked.0).ok().map(|c| c.is_empty()).unwrap_or(true) {
                        info!("  Flipping");
                        // Flip the card
                        commands.entity(clicked.0).insert(CardFace::Up).insert(Draggable);
                    } else {
                        info!("  But it had children: {:?} ({:?})", q_children.get(clicked.0), q_stacks.get(clicked.0));
                    }
                }
            }
        }
    }
}

fn card_move_system(mut commands: Commands, card_texture: Res<CardsTextureHandle>, mut move_cards: EventReader<MoveCard>, card: Query<&Card>) {
    for event in move_cards.iter() {
        commands.entity(event.to)
            .with_children(|parent| {
                parent.spawn_bundle(SpriteSheetBundle {
                    texture_atlas: card_texture.0.clone(),
                    transform: Transform::from_xyz(0.0, -CARD_STACK_SPACE, 1.0),
                    ..Default::default()
                })
                .insert(card.get(event.target).unwrap().clone())
                .insert(CardFace::Up)
                .insert(Clickable::default())
                .insert(Draggable);
        });
        commands.entity(event.target).despawn();
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
