use std::time::Duration;
use bevy::prelude::*;
use bevy::input::{Input, mouse::MouseMotion};
use bevy_easings::*;


use crate::game::{
    CARD_WIDTH,
    CARD_HEIGHT,
    CARD_STACK_SPACE,
    Deck,
    Card,
    CardFace,
    DiscardPile,
    Stack,
    StackKind,
    CardsTextureHandle,
    DrawMode,
    Area,
    move_card,
    top_entity,
    walk,
};

#[derive(Debug, Component)]
pub struct Clickable {
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
    pub fn at(pos: Vec2) -> Self {
        let size = Vec2::new(CARD_WIDTH, CARD_HEIGHT);
        Self {
            zone: Area {pos: pos - (size / 2.0), size: size},
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Component, PartialEq)]
pub enum MouseInteraction {
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
    pub fn is_hovered(&self) -> bool {
        match self {
            MouseInteraction::Hovered => true,
            _ => false,
        }
    }

    pub fn is_clicked(&self) -> bool {
        match self {
            MouseInteraction::Clicked(_) => true,
            _ => false,
        }
    }

    pub fn is_dragging(&self) -> bool {
        match self {
            MouseInteraction::Dragging {..} => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Clicked(Entity, Vec2);

#[derive(Debug, Clone)]
pub struct Released(Entity, Vec2);

#[derive(Debug, Clone, Component)]
pub struct LeftClicked(Vec2);

#[derive(Debug, Component)]
pub struct WasClicked(Timer);

#[derive(Debug, Component, Default)]
pub struct Dragging(Vec2);

#[derive(Debug, Component)]
pub struct Draggable;

#[derive(Debug, Component)]
pub struct Droppable {
    pub zone: Area,
}

#[derive(Debug)]
pub struct Dropped(Entity, Vec3, Vec2);

pub fn clickable_bounds_update_system(
    mut clickables: Query<(&mut Clickable, &GlobalTransform), Changed<GlobalTransform>>,
) {
    for (mut clickable, global_transform) in clickables.iter_mut() {
        clickable.zone.pos = Vec2::new(global_transform.translation.x, global_transform.translation.y) - (clickable.zone.size / 2.0);
    }
}

pub fn mouse_interaction_system(
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

pub fn click_system(
    mut commands: Commands,
    draw_mode: ResMut<DrawMode>,
    mut ev_released: EventReader<Released>,
    card_texture: Res<CardsTextureHandle>,
    q_card: Query<&Card>,
    q_was_clicked: Query<&WasClicked>,
    q_card_face: Query<&CardFace>,
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
                    if let Ok(card) = q_card.get(entity) {
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

        if let Ok(card) = q_card.get(*entity) {
            let face = q_card_face.get(*entity).unwrap();
            match face {
                CardFace::Up => {
                    let has_children = q_children.get(*entity).ok().map(|c| !c.is_empty()).unwrap_or(false);
                    if q_was_clicked.get(*entity).is_ok() && !has_children {
                        // double click
                        commands.entity(*entity).remove::<WasClicked>();
                        if let Some((stack_entity, stack)) = q_stacks.iter().filter(|(_, stack)| stack.kind == StackKind::Ordered(card.suit)).nth(0) {
                            let target = top_entity(stack_entity, &q_children);
                            let target_card = q_card.get(target).ok();
                            if stack.can_stack(target_card, *card, false) {
                                move_card(&mut commands, &q_parent, &q_gtransform, &mut q_transform, &q_card, &q_card_face, *entity, target, 0.0, 100);
                                break
                            }
                        }
                    } else {
                        commands.entity(*entity).insert(WasClicked(Timer::from_seconds(0.5, false)));
                    }
                },
                CardFace::Down => {
                    let has_children = q_children.get(*entity).ok().map(|c| !c.is_empty()).unwrap_or(false);
                    if !has_children {
                        // This shouldn't ever happen since its done autoamtically
                        commands.entity(*entity)
                                    .insert(CardFace::Up)
                                    .insert(Draggable);
                    }
                }
            }
        }
    }
}

pub fn drop_system(
    mut commands: Commands,
    mut ev_dropped: EventReader<Dropped>,
    q_droppable: Query<(Entity, &Droppable)>,
    q_children: Query<&Children>,
    q_parent: Query<&Parent>,
    q_card: Query<&Card>,
    q_card_face: Query<&CardFace>,
    q_stack: Query<&Stack>,
    mut q_transform: Query<&mut Transform>,
    q_global_transform: Query<&GlobalTransform>,
) {
    for Dropped(dropped, local_start_pos, _mouse_position) in ev_dropped.iter() {
        let pos3 = q_global_transform.get(*dropped).unwrap().translation;
        let pos = Vec2::new(pos3.x, pos3.y);
        let mut was_dropped = false;
        for (droppable_entity, droppable) in q_droppable.iter() {
            if droppable.zone.contains(pos) {
                // Find the bottom of the drop stack
                let top = top_entity(droppable_entity, &q_children);
                // If the entity is not a card we will get None
                let top_card = q_card.get(top).ok();
                let stack = q_stack.get(droppable_entity).unwrap();
                // If we are not dragging a card this will fail
                let dropped_card = q_card.get(*dropped).unwrap();
                let has_children = q_children.get(*dropped).ok().map(|c| !c.is_empty()).unwrap_or(false);
                if stack.can_stack(top_card, *dropped_card, has_children) {
                    let end_y = match stack.kind {
                        StackKind::Stack => {
                            if top_card.is_some() {
                                -CARD_STACK_SPACE
                            } else {
                                0.0
                            }
                        },
                        StackKind::Ordered(_) => {
                            0.0
                        }
                    };
                    move_card(&mut commands, &q_parent, &q_global_transform, &mut q_transform, &q_card, &q_card_face, *dropped, top, end_y, 50);
                    was_dropped = true;
                    break
                }
            }
        }
        if !was_dropped {
            // Move back to the old position
            let transform = q_transform.get_mut(*dropped).unwrap();
            commands
                .entity(*dropped)
                    .insert(
                        transform
                            .ease_to(
                                Transform::from_translation(*local_start_pos),
                                EaseFunction::QuadraticIn,
                                EasingType::Once {duration: Duration::from_millis(50)}
                            )
                    );
        }
    }
}

pub fn update_click_timers(mut commands: Commands, time: Res<Time>, mut timers: Query<(Entity, &mut WasClicked)>) {
    for (entity, mut timer) in timers.iter_mut() {
        if timer.0.tick(time.delta()).finished() {
            commands.entity(entity).remove::<WasClicked>();
        }
    }
}
