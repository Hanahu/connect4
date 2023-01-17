use bevy::prelude::*;

mod game;
mod menu;

use game::Turn;

const BACKGROUND_COLOR: Color = Color::rgb(0.0, 0.0, 0.0);
const FPS: f32 = 60.0;
pub const TIME_STEP: f32 = 1.0 / FPS;

pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 800.0;

// Used by main menu and game to determine if the game can be resumed or saved, and if there is a winner
#[derive(Resource)]
struct MainMenuInfo {
    pub allow_resume: bool,
    pub winner: Option<Turn>,
}

// Event type used to communicate between the main menu and game
pub enum GameChange {
    New { rows: i32, cols: i32 },
    Save,
    Load,
}

// Setup the bevy app, adding the main menu and game plugins
fn main() {
    App::new()
        .add_event::<GameChange>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: WINDOW_WIDTH,
                height: WINDOW_HEIGHT,
                title: "Connect 4".to_string(),
                resizable: false,
                ..default()
            },
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(MainMenuInfo {
            allow_resume: false,
            winner: None,
        })
        .add_startup_system(setup)
        .add_state(GameState::Menu)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .run();
}

// Used to determine which plugin to run (game or main menu)
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Playing,
    Menu,
}

// Setup the camera
fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
