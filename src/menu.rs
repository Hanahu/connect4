use bevy::{app::AppExit, prelude::*};

use crate::{game::WINNER_COLOR, GameChange, GameState, MainMenuInfo, WINDOW_HEIGHT, WINDOW_WIDTH};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const TITLE_COLOR: Color = Color::WHITE;
const FONT_COLOR: Color = Color::WHITE;
const TINT: Color = Color::rgba(0.0, 0.0, 0.0, 0.7);

// Used to label each button with a unique component
#[derive(Component)]
enum ButtonType {
    Resume,
    NewGame,
    IncreaseRows,
    DecreaseRows,
    IncreaseCols,
    DecreaseCols,
    Save,
    Load,
    Exit,
}

// Used to store the current board size that is displayed in the main menu
#[derive(Resource)]
struct BoardSize {
    rows: i32,
    cols: i32,
}

// To identify the text that displays the current board size
#[derive(Component)]
struct BoardSizeText;

// To identify all entities inside the menu, so they can be easily fetched and removed
#[derive(Component)]
struct InMenu;

pub struct MenuPlugin;

// Setup the main menu plugin, adding all the systems and resources (all only running when the state is GameState::Menu)
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoardSize { rows: 6, cols: 7 })
            .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(setup))
            .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(cleanup))
            .add_system_set(
                SystemSet::on_update(GameState::Menu)
                    .with_system(button_system)
                    .with_system(update_text),
            );
    }
}

// Add all entities to the screen
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    main_menu_info: Res<MainMenuInfo>,
) {
    // Reused data for the buttons -------------------
    let box_size = Size::new(Val::Px(200.0), Val::Px(65.0));

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: FONT_COLOR,
    };

    let button_style = Style {
        size: box_size,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        margin: UiRect {
            top: Val::Px(10.0),
            bottom: Val::Px(10.0),
            ..default()
        },
        ..default()
    };
    let button_bundle = ButtonBundle {
        style: button_style,
        background_color: NORMAL_BUTTON.into(),
        ..default()
    };
    // ----------------------------------------------

    // Tint, so the game is not too visible behind the menu
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.5),
                scale: Vec3::new(WINDOW_WIDTH, WINDOW_HEIGHT, 0.0),
                ..default()
            },
            sprite: Sprite {
                color: TINT,
                ..default()
            },
            ..default()
        },
        InMenu,
    ));

    // Main menu entity, used to center all the buttons
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            InMenu,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn(TextBundle::from_section(
                "Connect 4",
                TextStyle {
                    font: font.clone(),
                    font_size: 50.0,
                    color: TITLE_COLOR,
                },
            ));

            // Winner (if there is one)
            if let Some(winner) = &main_menu_info.winner {
                parent.spawn(TextBundle::from_section(
                    format!("{} wins!", winner),
                    TextStyle {
                        font,
                        font_size: 40.0,
                        color: WINNER_COLOR,
                    },
                ));
            }

            // Resume button
            if main_menu_info.allow_resume {
                parent
                    .spawn((button_bundle.clone(), ButtonType::Resume))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section("Resume", text_style.clone()));
                    });
            }

            // New Game button
            parent
                .spawn((button_bundle.clone(), ButtonType::NewGame))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("New Game", text_style.clone()));
                });

            // Board size label
            parent.spawn(TextBundle::from_section("Board Size:", text_style.clone()));

            // Board size buttons and text, inside new entities for easy formatting
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: box_size,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Rows increase/decrease
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(20.0), Val::Percent(100.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            // Increase row button
                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            size: Size::new(
                                                Val::Percent(100.0),
                                                Val::Percent(50.0),
                                            ),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        background_color: NORMAL_BUTTON.into(),
                                        ..default()
                                    },
                                    ButtonType::IncreaseRows,
                                ))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section("+", text_style.clone()));
                                });

                            // Decrease row button
                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            size: Size::new(
                                                Val::Percent(100.0),
                                                Val::Percent(50.0),
                                            ),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        background_color: NORMAL_BUTTON.into(),
                                        ..default()
                                    },
                                    ButtonType::DecreaseRows,
                                ))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section("-", text_style.clone()));
                                });
                        });

                    // Size text
                    parent.spawn((
                        TextBundle::from_section("6x7", text_style.clone()),
                        BoardSizeText,
                    ));

                    // Cols increase/decrease
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(20.0), Val::Percent(100.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            // Increase col button
                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            size: Size::new(
                                                Val::Percent(100.0),
                                                Val::Percent(50.0),
                                            ),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        background_color: NORMAL_BUTTON.into(),
                                        ..default()
                                    },
                                    ButtonType::IncreaseCols,
                                ))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section("+", text_style.clone()));
                                });

                            // Decrease col button
                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            size: Size::new(
                                                Val::Percent(100.0),
                                                Val::Percent(50.0),
                                            ),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        background_color: NORMAL_BUTTON.into(),
                                        ..default()
                                    },
                                    ButtonType::DecreaseCols,
                                ))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section("-", text_style.clone()));
                                });
                        });
                });

            // Save button
            if main_menu_info.allow_resume {
                parent
                    .spawn((button_bundle.clone(), ButtonType::Save))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section("Save Game", text_style.clone()));
                    });
            }

            // Load button
            parent
                .spawn((button_bundle.clone(), ButtonType::Load))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Load Game", text_style.clone()));
                });

            // Exit button
            parent
                .spawn((button_bundle.clone(), ButtonType::Exit))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Exit", text_style.clone()));
                });
        });
}

// Remove all entities that are in the menu
fn cleanup(mut commands: Commands, query: Query<Entity, With<InMenu>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

type OnButtonChanged = (Changed<Interaction>, With<Button>);

// Button system, handles all button interactions
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonType),
        OnButtonChanged,
    >,
    mut game_state: ResMut<State<GameState>>,
    mut exit: EventWriter<AppExit>,
    mut game_change: EventWriter<GameChange>,
    mut board_size: ResMut<BoardSize>,
) {
    for (interaction, mut background_color, button_type) in &mut interaction_query {
        // Check each interaction, and color the button accordingly
        match *interaction {
            Interaction::Clicked => {
                *background_color = PRESSED_BUTTON.into();

                // Handle button presses
                match button_type {
                    // Just return to the game
                    ButtonType::Resume => game_state.set(GameState::Playing).unwrap(),
                    // Send the event to create a new game, and then return to the game
                    ButtonType::NewGame => {
                        game_change.send(GameChange::New {
                            rows: board_size.rows,
                            cols: board_size.cols,
                        });
                        game_state.set(GameState::Playing).unwrap();
                    }
                    ButtonType::IncreaseRows => {
                        if (board_size.cols - (board_size.rows + 1)).abs() <= 2 {
                            board_size.rows += 1;
                        }
                    }
                    ButtonType::DecreaseRows => {
                        if board_size.rows > 6
                            && (board_size.cols - (board_size.rows - 1)).abs() <= 2
                        {
                            board_size.rows -= 1;
                        }
                    }
                    ButtonType::IncreaseCols => {
                        if ((board_size.cols + 1) - board_size.rows).abs() <= 2 {
                            board_size.cols += 1;
                        }
                    }
                    ButtonType::DecreaseCols => {
                        if board_size.cols > 7
                            && ((board_size.cols - 1) - board_size.rows).abs() <= 2
                        {
                            board_size.cols -= 1;
                        }
                    }
                    // Tell the game to save, and then return to the game
                    ButtonType::Save => {
                        game_change.send(GameChange::Save);
                        game_state.set(GameState::Playing).unwrap();
                    }
                    // Tell the game to load, and then return to the game
                    ButtonType::Load => {
                        game_change.send(GameChange::Load);
                        game_state.set(GameState::Playing).unwrap();
                    }
                    // Exit the whole app
                    ButtonType::Exit => exit.send_default(),
                }
            }
            Interaction::Hovered => {
                *background_color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *background_color = NORMAL_BUTTON.into();
            }
        }
    }
}

// Keeps the BoardSize struct and displayed text in sync
fn update_text(mut query: Query<&mut Text, With<BoardSizeText>>, board_size: Res<BoardSize>) {
    for mut text in &mut query {
        text.sections[0].value = format!("{}x{}", board_size.rows, board_size.cols);
    }
}
