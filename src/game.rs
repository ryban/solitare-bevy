use std::collections::HashMap;
use std::time::Duration;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::render::view::Visibility;
use bevy::ui::Display;
use bevy_easings::*;
// use bevy_easings::*;
use rand::prelude::*;

use crate::mouse_input::{Droppable, Draggable, Clickable, MouseInteraction};
use crate::menus::{ResetMenuRoot, ResetButton};

#[allow(dead_code)] pub const BACK_GREEN: usize = 5 * 13;
pub const BACK_BLUE: usize = 6 * 13;
#[allow(dead_code)] pub const BACK_RED: usize = 7 * 13;
pub const FINAL_STACKS: usize = 7 * 13 + 9;
pub const EMPTY_SPACE: usize = 6 * 13 + 12;

pub const CARD_WIDTH: f32 = 140.0;
pub const CARD_HEIGHT: f32 = 190.0;
pub const CARD_STACK_SPACE: f32 = 35.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    Spades,
    Clubs,
    Hearts,
    Diamonds,
}

impl Suit {
    pub fn can_stack(&self, other: &Self) -> bool {
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
    pub fn row(&self) -> usize {
        match self {
            Suit::Spades => 0,
            Suit::Clubs => 1,
            Suit::Diamonds => 2,
            Suit::Hearts => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardKind {
    Ace,
    Number(usize),
    Jack,
    Queen,
    King,
    // Joker,
}

impl CardKind {
    pub fn column(&self) -> usize {
        match self {
            CardKind::Ace => 0,
            // 2:10 => 1:9
            CardKind::Number(n) => n - 1,
            CardKind::Jack => 10,
            CardKind::Queen => 11,
            CardKind::King => 12,
        }
    }

    pub fn can_stack(&self, other: &Self) -> bool {
        self != &CardKind::Ace && other != &CardKind::Ace && self.column() < other.column() && (other.column() - self.column() == 1)
    }

    pub fn next(&self) -> Option<CardKind> {
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

#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct Card {
    pub suit: Suit,
    pub kind: CardKind,
}

impl Card {
    pub fn texture_index(&self) -> usize {
        (self.suit.row() * 13) + self.kind.column()
    }

    /// Return true if other can be stacked below self
    pub fn can_stack(&self, other: &Card) -> bool {
        self.suit.can_stack(&other.suit) && self.kind.can_stack(&other.kind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StackKind {
    Ordered(Suit),
    Stack,
}

#[derive(Debug, Component)]
pub struct Deck {
    pub cards: Vec<Card>,
}

// XXX: Possibly change this to be a Card field instead of an individual component?
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum CardFace {
    Down,
    Up,
}

#[derive(Debug)]
pub struct Area {
    pub pos: Vec2,
    pub size: Vec2,
}

impl Area {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            size: Vec2::new(width, height),
        }
    }

    pub fn contains(&self, pos: Vec2) -> bool {
        // FIXME: Change it so the position is relative to the center of the entity? Or should it always be global like it is now?
        pos.x >= self.pos.x &&
        pos.x <= (self.pos.x + self.size.x) &&
        pos.y >= self.pos.y &&
        pos.y <= (self.pos.y + self.size.y)
    }
}

#[derive(Debug, PartialEq, Resource)]
pub enum DrawMode {
    Draw1,
    Draw3,
}

impl DrawMode {
    pub fn num(&self) -> usize {
        match self {
            DrawMode::Draw1 => 1,
            DrawMode::Draw3 => 3,
        }
    }
}

#[derive(Debug, Resource)]
pub struct SolveTimer(pub Timer);

#[derive(Debug, Default, Component)]
pub struct DiscardPile;

#[derive(Debug, Component, Clone)]
pub struct Stack {
    pub kind: StackKind,
}

impl Stack {
    pub fn new(kind: StackKind) -> Self {
        Self {
            kind: kind,
        }
    }

    pub fn can_stack(&self, stack_card: Option<&Card>, target: Card, has_children: bool) -> bool {
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
#[derive(Resource)]
pub struct CardsTextureHandle(pub Handle<TextureAtlas>);
#[derive(Resource)]
pub struct FontHandle(pub Handle<Font>);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    AutoSolving,
    Shuffle,
    Won,
}

#[derive(Debug)]
pub enum Action {
    /// Move a card from one stack to another
    MoveCard {
        /// The card that was moved
        card: Card,
        /// The stack entity this card was on
        from: Entity,
        /// The stack entity this card was moved to
        to: Entity,
        /// The cards old Y offset
        y_offset: f32,
        // Whether or not this cards old parent was face down
        parent_face_down: bool,
    },
    /// The deck was reset
    ResetDeck,
    /// Flip n cards from the top of the deck
    Draw(usize),
}

#[derive(Default, Resource)]
pub struct Actions(pub Vec<Action>);


pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut camera_bundle = Camera2dBundle::default();
    // camera_bundle.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    camera_bundle.projection.scaling_mode = ScalingMode::WindowSize(1.0);
    commands.spawn(camera_bundle);

    // Pre-load this now
    let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.insert_resource(FontHandle(font_handle.clone()));
    let card_texture_handle = asset_server.load("playingCards.png");
    let card_atlas = TextureAtlas::from_grid(card_texture_handle, Vec2::new(CARD_WIDTH, CARD_HEIGHT), 13, 8, None, None);
    let card_atlas_handle = texture_atlases.add(card_atlas);
    commands.insert_resource(CardsTextureHandle(card_atlas_handle.clone()));

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::None,
                margin: UiRect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(5.0),
                    top: Val::Auto,
                    right: Val::Auto,
                },
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(ResetMenuRoot)
        .with_children(|parent| {
            parent.spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(20.0),
                        margin: UiRect {
                            right: Val::Px(5.0),
                            ..Default::default()
                        },
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::FlexEnd,
                        ..Default::default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..Default::default()
                })
                .insert(ResetButton::Draw1)
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "New Single Draw",
                            TextStyle {
                                font: font_handle.clone(),
                                font_size: 20.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ),
                        ..Default::default()
                    });
                });

            parent.spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(20.0),
                        margin: UiRect {
                            left: Val::Px(5.0),
                            ..Default::default()
                        },
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::FlexEnd,
                        ..Default::default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..Default::default()
                })
                .insert(ResetButton::Draw3)
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "New Triple Draw",
                            TextStyle {
                                font: font_handle.clone(),
                                font_size: 20.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            }
                        ),
                        ..Default::default()
                    });
                });
        });
}

pub fn clean_cards(mut commands: Commands, cleanup: Query<Entity, Or<(With<Card>, With<Stack>, With<Deck>, With<DiscardPile>)>>) {
    for entity in cleanup.iter() {
        commands.entity(entity).despawn();
    }

}

pub fn reset_cards(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    card_texture: Res<CardsTextureHandle>,
    windows: Query<&Window>,
    mut actions: ResMut<Actions>,
    mut reset_menu: Query<&mut Style, With<ResetMenuRoot>>
) {
    actions.0.clear();
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

    let window = if let Ok(w) = windows.get_single() {w} else {return};
    let window_half_width = window.width() / 2.0;
    let window_half_height = window.height() / 2.0;
    let stacks_y = window.height() * 0.1;

    let stack_x = |stack: usize| {
        50.0 + (175.0 * (stack as f32)) + (CARD_WIDTH / 2.0) - window_half_width
    };

    let mut stacks: Vec<Vec<Entity>> = (0..7).map(|stack| {
        let pos = Vec2::new(stack_x(stack), stacks_y);
        vec![
            commands.spawn(SpriteSheetBundle {
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
                .insert(Droppable {zone: Area::new(pos.x - CARD_WIDTH / 2.0, -window_half_height, CARD_WIDTH, pos.y + window_half_height + CARD_HEIGHT)})
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
        let new = commands.spawn(SpriteSheetBundle {
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
            let new = commands.spawn(SpriteSheetBundle {
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

    let deck_pos = Vec2::new((CARD_WIDTH / 2.0) + 50.0 - window_half_width, window_half_height - 25.0 - (CARD_HEIGHT / 2.0));
    commands.spawn(SpriteSheetBundle {
            texture_atlas: card_texture.0.clone(),
            transform: Transform::from_xyz((CARD_WIDTH / 2.0) + 50.0 - window_half_width, window_half_height - 25.0 - (CARD_HEIGHT / 2.0), 1.0),
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
            parent.spawn(SpriteSheetBundle {
                texture_atlas: card_texture.0.clone(),
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                sprite: TextureAtlasSprite {
                    index: EMPTY_SPACE,
                    ..Default::default()
                },
                ..Default::default()
            });
        });

    commands.spawn(SpriteSheetBundle {
            transform: Transform::from_xyz((CARD_WIDTH / 2.0) + 50.0 + 175.0 - window_half_width , window_half_height - 25.0 - (CARD_HEIGHT / 2.0), 1.0),
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
        commands.spawn(SpriteSheetBundle {
            texture_atlas: card_texture.0.clone(),
            sprite: TextureAtlasSprite {
                index: FINAL_STACKS + suit.row(),
                ..Default::default()
            },
            transform: Transform::from_xyz(stack_pos.x, stack_pos.y, 0.0),
            ..Default::default()
        })
        .insert(Stack::new(StackKind::Ordered(suit)))
        .insert(Droppable {zone: Area::new(stack_pos.x - CARD_WIDTH / 2.0 - window_half_width, stack_pos.y - CARD_HEIGHT / 2.0, CARD_WIDTH, CARD_HEIGHT)});
    }

    for mut style in reset_menu.iter_mut() {
        style.display = Display::Flex;
    }

    game_state.set(GameState::Playing);
}

pub fn deck_update_system(mut decks: Query<(&Deck, &mut Visibility), Changed<Deck>>) {
    for (deck, mut visible) in decks.iter_mut() {
        *visible = if deck.cards.len() > 0 {Visibility::Visible} else {Visibility::Hidden};
    }
}

pub fn discard_update_system(
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
            // When we undo a draw, some entities at the top are no longer guaranteed to be there
            // TODO: Fix this system order or somehow otherwise ensure there are not stale entities
            if let Ok(mut transform) = q_transform.get_mut(*child) {
                transform.translation = Vec3::new(CARD_STACK_SPACE, 0.0, 1.0);
            }
        } else {
            if let Ok(mut transform) = q_transform.get_mut(*child) {
                transform.translation = Vec3::new(0.0, 0.0, 1.0);
            }
        }
    }
}

pub fn win_check_system(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    actions: Res<Actions>,
    q_stacks: Query<(Entity, &Stack)>,
    q_card: Query<(&Card, &CardFace)>,
    q_children: Query<&Children>,
    q_deck: Query<&Deck>,
    q_discard: Query<Entity, With<DiscardPile>>,
) {
    let mut completed = 0;
    for (stack_entity, _) in q_stacks.iter().filter(|(_entity, stack)| match stack.kind {StackKind::Ordered(_) => true, _ => false}) {
        let top = top_entity(stack_entity, &q_children);
        if let Ok((card, _)) = q_card.get(top) {
            if card.kind == CardKind::King {
                completed += 1;
            }
        }
    }
    if completed == 4 {
        info!("Game Won in {} moves!", actions.0.len());
        game_state.set(GameState::Won);
    } else {
        // The only place facedown cards will exist is on the board
        if q_deck.single().cards.len() == 0 && q_card.iter().all(|(_, face)| face == &CardFace::Up) {
            let discard = q_discard.single();
            let top_discard = top_entity(discard, &q_children);
            if discard == top_discard {
                info!("Attempting to auto-solve");
                commands.insert_resource(SolveTimer(Timer::from_seconds(0.15, TimerMode::Repeating)));
                game_state.set(GameState::AutoSolving);
            }
        }
    }
}

pub fn auto_solver(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut solve_timer: ResMut<SolveTimer>,
    mut actions: ResMut<Actions>,
    time: Res<Time>,
    q_stacks: Query<(Entity, &Stack)>,
    q_card: Query<&Card>,
    q_card_face: Query<&CardFace>,
    q_children: Query<&Children>,
    q_parent: Query<&Parent>,
    q_gtransform: Query<&GlobalTransform>,
    mut q_transform: Query<&mut Transform>,
) {
    if !solve_timer.0.tick(time.delta()).finished() {
        return
    }
    let mut stacks = HashMap::new();
    let mut to_solve = Vec::new();
    for (stack_entity, stack) in q_stacks.iter() {
        let top = top_entity(stack_entity, &q_children);
        match stack.kind {
            StackKind::Ordered(suit) => {
                let card = q_card.get(top).ok();
                assert!(stacks.insert(suit, (stack_entity, top, card)).is_none());
            },
            StackKind::Stack => {
                if top != stack_entity {
                    let card = q_card.get(top).unwrap();
                    to_solve.push((top, card))
                }
            }
        }
    }
    if to_solve.len() == 0 {
        // Just go back to playing and let the normal check logic set it for now to double check that is working
        // This is will get stuck in a loop if its wrong, but it worked from the very beginning so...
        game_state.set(GameState::Playing);
    }
    // Always solve them lowest to highest
    to_solve.sort_by_key(|(_, card)| card.kind.column());

    for (candidate, card) in to_solve {
        if let Some((stack_entity, top, maybe_top_card)) = stacks.get(&card.suit) {
            let stack = Stack::new(StackKind::Ordered(card.suit));
            if stack.can_stack(*maybe_top_card, *card, false) {
                actions.0.push(Action::MoveCard {
                    card: card.clone(),
                    from: bottom_entity(candidate, &q_parent),
                    to: *stack_entity,
                    y_offset: CARD_STACK_SPACE,
                    parent_face_down: false,
                });
                move_card(&mut commands, &q_parent, &q_gtransform, &mut q_transform, &q_card, &q_card_face, candidate, *top, 0.0, 100);
                break
            }
        }
    }
}

pub fn card_texture_update_system(mut cards: Query<(&Card, &CardFace, &mut TextureAtlasSprite), Changed<CardFace>>) {
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

pub fn move_card(
    commands: &mut Commands,
    q_parent: &Query<&Parent>,
    q_gtransform: &Query<&GlobalTransform>,
    q_transform: &mut Query<&mut Transform>,
    q_card: &Query<&Card>,
    q_card_face: &Query<&CardFace>,
    to_move: Entity,
    new_parent: Entity,
    final_y_offset: f32,
    animation_time: u64,
) {
    if let Ok(parent) = q_parent.get(to_move) {
        commands.entity(parent.get()).remove_children(&[to_move]);
        if q_card.contains(parent.get()) {
            // Make the new stack top draggable
            commands.entity(parent.get()).insert(Draggable);
            if let Ok(face) = q_card_face.get(parent.get()) {
                if face == &CardFace::Down {
                    commands.entity(parent.get()).insert(CardFace::Up);
                }
            }
        }
    }
    commands.entity(new_parent).add_child(to_move);
    let target_pos = q_gtransform.get(new_parent).unwrap().translation();
    let cur_pos = q_gtransform.get(to_move).unwrap().translation();
    let mut start_pos = cur_pos - target_pos;
    start_pos.z = 250.0;
    let end_pos = Transform::from_xyz(0.0, final_y_offset, 250.0);
    let mut cur_transform = q_transform.get_mut(to_move).unwrap();
    // immediately move to the parent relative start position to prevent occasional glitches
    cur_transform.translation = start_pos;
    commands
        .entity(to_move)
            .insert(
                Transform::from_translation(start_pos)
                    .ease_to(
                        end_pos,
                        EaseFunction::QuadraticIn,
                        EasingType::Once {duration: Duration::from_millis(animation_time)}
                    )
                    // this will keep the card above all of the others until it reaches the final position
                    .ease_to(
                        Transform::from_xyz(0.0, final_y_offset, 1.0),
                        EaseFunction::QuadraticIn,
                        EasingType::Once {duration: Duration::from_millis(1)}
                    )
            );

}

pub fn undo(
    mut commands: Commands,
    card_texture: Res<CardsTextureHandle>,
    mut actions: ResMut<Actions>,
    mut q_deck: Query<&mut Deck>,
    keys: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
    q_interaction: Query<&MouseInteraction>,
    q_card: Query<&Card>,
    q_card_face: Query<&CardFace>,
    q_children: Query<&Children>,
    q_parent: Query<&Parent>,
    q_discard: Query<Entity, With<DiscardPile>>,
    q_gtransform: Query<&GlobalTransform>,
    mut q_transform: Query<&mut Transform>,
) {
    // If we are currently dragging a card, don't attempt to undo anything
    if q_interaction.iter().any(|interaction| interaction.is_dragging()) {
        return
    }

    // ctrl+Z or right click to undo
    if actions.0.len() > 0 && (keys.just_pressed(KeyCode::Z) && keys.pressed(KeyCode::ControlLeft)) || mouse.just_released(MouseButton::Right) {
        if let Some(action) = actions.0.pop() {
            debug!("undo {:?}", action);
            let mut deck = q_deck.single_mut();
            let discard_pile = q_discard.single();
            match action {
                Action::MoveCard {card, from, to, y_offset, parent_face_down} => {
                    let mut target = None;
                    walk_children(Some(to), &q_children, &mut |e| {
                        if q_card.get(e).ok() == Some(&card) {
                            target = Some(e)
                        }
                    });
                    if let Some(target) = target {
                        let top = top_entity(from, &q_children);
                        let discard_top = top_entity(discard_pile, &q_children);
                        move_card(&mut commands, &q_parent, &q_gtransform, &mut q_transform, &q_card, &q_card_face, target, top, y_offset, 100);
                        if top == discard_top {
                            commands.entity(discard_top).remove::<Draggable>();

                            let discard_positon = q_gtransform.get(discard_pile).unwrap();
                            let click_position = Vec2::new(discard_positon.translation().x, discard_positon.translation().y);
                            commands.entity(discard_top).insert(Clickable::at(click_position)).insert(Draggable);
                            commands.entity(target).insert(Draggable);
                        } else {
                            if parent_face_down {
                                commands.entity(top).insert(CardFace::Down).remove::<Draggable>();
                            }
                        }
                    }
                },
                Action::ResetDeck => {
                    let mut discard_top = top_entity(discard_pile, &q_children);
                    while let Some(card) = deck.cards.pop() {

                        let new = commands.spawn(
                            SpriteSheetBundle {
                                texture_atlas: card_texture.0.clone(),
                                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                                ..Default::default()
                            })
                            .insert(card)
                            .insert(CardFace::Up)
                            .insert(Clickable::at(Vec2::new(0.0, 0.0)))
                            .id();
                        commands.entity(discard_top).add_child(new);
                        discard_top = new;
                    }
                    if discard_top != discard_pile {
                        let discard_positon = q_gtransform.get(discard_pile).unwrap();
                        let click_position = Vec2::new(discard_positon.translation().x, discard_positon.translation().y);
                        commands.entity(discard_top)
                                    .insert(Clickable::at(click_position))
                                    .insert(Draggable);
                    }
                },
                Action::Draw(n) => {
                    debug!("discard pile: {:?}", discard_pile);
                    walk_children(Some(discard_pile), &q_children, &mut |e| debug!("  {:?}", e));
                    let mut discard_top = top_entity(discard_pile, &q_children);
                    // This cannot happen. If the last action was to draw there _must_ be cards on the discard pile
                    assert!(discard_top != discard_pile);
                    commands.entity(discard_top).remove::<Draggable>();
                    for _ in 0..n {
                        // This shouldn't fail unless something modified the deck without updating "moves"
                        let card = q_card.get(discard_top).unwrap();
                        deck.cards.push(*card);

                        // Must be done in this order
                        let old_top = discard_top;
                        discard_top = q_parent.get(discard_top).unwrap().get();
                        commands.entity(discard_top).remove_children(&[old_top]);
                        debug!("undo despawning {:?}", old_top);
                        commands.entity(old_top).despawn();
                    }
                    debug!("new discard top: {:?}", discard_top);
                    if discard_top != discard_pile {
                        let discard_positon = q_gtransform.get(discard_pile).unwrap();
                        let click_position = Vec2::new(discard_positon.translation().x, discard_positon.translation().y);
                        commands.entity(discard_top).insert(Clickable::at(click_position)).insert(Draggable);
                    }
                },
            }
        }
    }
}

pub fn walk(mut node: Option<Entity>, query: &Query<&Parent>, func: &mut dyn FnMut(Entity)) {
    while let Some(entity) = node {
        func(entity);
        node = if let Ok(parent) = query.get(entity) {
            Some(parent.get())
        } else {
            None
        }
    }
}

pub fn walk_children(mut node: Option<Entity>, query: &Query<&Children>, func: &mut dyn FnMut(Entity)) {
    while let Some(entity) = node {
        func(entity);
        node = if let Ok(children) = query.get(entity) {
            children.first().cloned()
        } else {
            None
        };
    }
}

pub fn top_entity(mut node: Entity, children: &Query<&Children>) -> Entity {
    while let Some(entity) = children.get(node).ok().map(|c| c.first()).flatten() {
        node = *entity
    }
    node
}

pub fn bottom_entity(mut node: Entity, parent: &Query<&Parent>) -> Entity {
    while let Some(entity) = parent.get(node).ok().map(|p| p.get()) {
        node = entity
    }
    node
}
