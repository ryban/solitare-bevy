use bevy::prelude::*;
use bevy_easings::*;
use bevy::window::PresentMode;

mod menus;
mod mouse_input;
mod game;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Solitaire".to_string(),
            width: 1280.,
            height: 960.,
            present_mode: PresentMode::Mailbox,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EasingsPlugin)
        .insert_resource(ClearColor(Color::rgb(0.3, 0.7, 0.1)))
        .insert_resource(game::DrawMode::Draw1)
        .insert_resource(game::Actions::default())
        .add_state(game::GameState::Menu)
        .add_event::<mouse_input::Released>()
        .add_event::<mouse_input::Dropped>()
        .add_startup_system(game::setup)
        .add_startup_system(menus::setup_menu)
        .add_system_set(
            SystemSet::on_enter(game::GameState::Menu)
                .with_system(menus::show_menu)
        )
        .add_system_set(
            SystemSet::on_update(game::GameState::Menu)
                .with_system(menus::main_menu)
        )
        .add_system_set(
            SystemSet::on_exit(game::GameState::Menu)
                .with_system(menus::hide_menu)
        )
        .add_system_set(
            SystemSet::on_enter(game::GameState::Shuffle)
                .with_system(game::clean_cards)
                .with_system(game::reset_cards)
        )
        .add_system_set(
            SystemSet::on_update(game::GameState::Playing)
                .with_system(mouse_input::update_click_timers)
                .with_system(game::undo.before("mouse"))
                // This is done before the mouse because the mouse can modify the deck to move cards from the deck to the discard
                // If there is only 1 or 3 cards in the deck and no discard pile the win system will see it as a win this frame
                // because the deck is empty but the new discard entities have not spawned yet.
                .with_system(game::win_check_system.before("mouse"))
                .with_system(mouse_input::mouse_interaction_system.label("mouse"))
                .with_system(mouse_input::click_system.after("mouse"))
                .with_system(mouse_input::drop_system.after("mouse"))
                // Run this before the mouse system so the double click sets the translation to the
                // completed pile rather than the discard pile resetting it
                // It would nice to make this event based so its not running constantly anyways
                .with_system(game::discard_update_system)
                .with_system(game::card_texture_update_system)
                .with_system(game::deck_update_system)
                .with_system(menus::reset_game_button)
        )
        .add_system_set(
            SystemSet::on_update(game::GameState::AutoSolving)
                .with_system(game::auto_solver)
        )
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(mouse_input::clickable_bounds_update_system)
        )
        .add_system_set(
            SystemSet::on_enter(game::GameState::Won)
                .with_system(menus::spawn_win_screen)
        )
        .add_system_set(
            SystemSet::on_update(game::GameState::Won)
                .with_system(menus::win_screen)
        )
        .add_system_set(
            SystemSet::on_exit(game::GameState::Won)
                .with_system(game::clean_cards)
        )
        .run();
}
