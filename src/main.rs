use bevy::prelude::*;
use bevy_easings::*;
use bevy::window::PresentMode;

mod menus;
mod mouse_input;
mod game;

fn main() {
    App::new()
        .add_plugins(
            (
                DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Solitaire".to_string(),
                        resolution: (1280., 960.).into(),
                        present_mode: PresentMode::AutoVsync,
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                EasingsPlugin,
            )
        )
        .insert_resource(ClearColor(Color::rgb(0.3, 0.7, 0.1)))
        .insert_resource(game::DrawMode::Draw1)
        .insert_resource(game::Actions::default())
        .add_state::<game::GameState>()
        .add_event::<mouse_input::Released>()
        .add_event::<mouse_input::Dropped>()
        .add_systems(Startup, (game::setup, menus::setup_menu))
        .add_systems(OnEnter(game::GameState::Menu), menus::show_menu)
        .add_systems(Update, menus::main_menu.run_if(in_state(game::GameState::Menu)))
        .add_systems(OnExit(game::GameState::Menu), menus::hide_menu)
        .add_systems(OnEnter(game::GameState::Shuffle), (game::clean_cards, game::reset_cards))
        .add_systems(
            Update,
            (
                mouse_input::update_click_timers,
                game::undo,
                // This is done before the mouse because the mouse can modify the deck to move cards from the deck to the discard
                // If there is only 1 or 3 cards in the deck and no discard pile the win system will see it as a win this frame
                // because the deck is empty but the new discard entities have not spawned yet.
                apply_deferred,
                game::win_check_system,
                mouse_input::mouse_interaction_system,
                mouse_input::click_system,
                mouse_input::drop_system,
                // Run this before the mouse system so the double click sets the translation to the
                // completed pile rather than the discard pile resetting it
                // It would nice to make this event based so its not running constantly anyways
                apply_deferred,
                game::discard_update_system,
                game::card_texture_update_system,
                game::deck_update_system,
                menus::reset_game_button,
            ).chain().run_if(in_state(game::GameState::Playing))
        )
        .add_systems(Update, game::auto_solver.run_if(in_state(game::GameState::AutoSolving)))
        .add_systems(
            PreUpdate,
            mouse_input::clickable_bounds_update_system
        )
        .add_systems(OnEnter(game::GameState::Won), menus::spawn_win_screen)
        .add_systems(Update, menus::win_screen.run_if(in_state(game::GameState::Won)))
        .add_systems(OnExit(game::GameState::Won), game::clean_cards)
        .run();
}
