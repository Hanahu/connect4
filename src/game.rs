#![allow(clippy::too_many_arguments)]

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use serde::{Deserialize, Serialize};

use crate::{GameChange, GameState, MainMenuInfo, BACKGROUND_COLOR, WINDOW_HEIGHT, WINDOW_WIDTH};

const BOARD_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
const BOARD_SCALE: Vec2 = Vec2::new(1.0, 0.9);
const HOLE_PADDING: f32 = 0.9;
const DISK_PADDING: f32 = 0.95 * HOLE_PADDING;
const WINNER_LINE_HEIGHT: f32 = 0.5;

const RED_DISK_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);
const BLUE_DISK_COLOR: Color = Color::rgb(0.0, 0.0, 1.0);
pub const WINNER_COLOR: Color = Color::rgb(1.0, 1.0, 0.0);

// Holds all the materials used by the game
#[derive(Resource)]
struct MaterialHandles {
    background: Handle<ColorMaterial>,
    red_disk: Handle<ColorMaterial>,
    blue_disk: Handle<ColorMaterial>,
    red_ghost_disk: Handle<ColorMaterial>,
    blue_ghost_disk: Handle<ColorMaterial>,
}

impl MaterialHandles {
    fn get_disk_material(&self, disk: Disk) -> Handle<ColorMaterial> {
        match disk {
            Disk::Red => self.red_disk.clone(),
            Disk::Blue => self.blue_disk.clone(),
        }
    }
}

// Holds all the meshes used by the game
#[derive(Resource)]
struct MeshHandles {
    circle: Handle<Mesh>,
}

// Used to identify the ghost disks (used to show where the next disk will be placed)
#[derive(Component, PartialEq, Eq, Clone, Copy)]
enum GhostDisk {
    Red,
    Blue,
}

#[derive(Resource, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Red,
    Blue,
}

impl std::fmt::Display for Turn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Turn::Red => "Red",
                Turn::Blue => "Blue",
            }
        )
    }
}

// Some helpful functions for converting the Turn enum
impl Turn {
    fn next(&mut self) {
        *self = match self {
            Turn::Red => Turn::Blue,
            Turn::Blue => Turn::Red,
        }
    }

    fn to_disk(self) -> Disk {
        match self {
            Turn::Red => Disk::Red,
            Turn::Blue => Disk::Blue,
        }
    }

    fn to_ghost_disk(self) -> GhostDisk {
        match self {
            Turn::Red => GhostDisk::Red,
            Turn::Blue => GhostDisk::Blue,
        }
    }

    fn to_color(self) -> Color {
        match self {
            Turn::Red => RED_DISK_COLOR,
            Turn::Blue => BLUE_DISK_COLOR,
        }
    }
}

// Used to stop click from menu propagating to game
#[derive(Resource)]
struct SkipClick(bool);

// Used to identify which entities are in the game, so they can be removed when the game ends
#[derive(Component)]
struct InGame;

// To identify empty slots (not really used, but could be useful for any updates/other features)
#[derive(Component)]
struct EmptyDisk;

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
enum Disk {
    Red,
    Blue,
}

impl Disk {
    fn to_turn(self) -> Turn {
        match self {
            Disk::Red => Turn::Red,
            Disk::Blue => Turn::Blue,
        }
    }
}

// Dimensions of the board and screen to simplify logic
struct Dimensions {
    board_scale_y: f32,
    row_height: f32,
    col_width: f32,
    scale: f32,
}

// Gets the dimensions of the board and screen
fn get_dimensions(board: &Board, padding: f32) -> Dimensions {
    let board_scale_y = (1.0 - 1.0 / (board.rows as f32 + 1.0)) * BOARD_SCALE.y;

    // Height of each row in the board
    let row_height = WINDOW_HEIGHT * board_scale_y / board.rows as f32;

    // Width of each column in the board
    let col_width = WINDOW_WIDTH * BOARD_SCALE.x / board.cols as f32;

    // Scale of the disks in the board (padding may be for the hole or the disk)
    let scale = (col_width * padding).min(row_height * padding);

    Dimensions {
        board_scale_y,
        row_height,
        col_width,
        scale,
    }
}

// Add a new disk to the board
fn draw_disk(
    commands: &mut Commands,
    mesh_handles: &MeshHandles,
    material_handles: &MaterialHandles,
    dims: &Dimensions,
    col: i32,
    row: i32,
    disk: Disk,
) {
    let mut transform = get_disk_transform(dims, row, col);

    // Keep it infront of the holes
    transform.translation.z = 0.2;

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh_handles.circle.clone().into(),
            material: material_handles.get_disk_material(disk),
            transform,
            ..default()
        },
        InGame,
    ));
}

// Add a new hole to the board (the holes are drawn as circles
//      with the same color as the background, and the board is just a rectangle)
fn draw_hole(
    commands: &mut Commands,
    mesh_handles: &MeshHandles,
    material_handles: &MaterialHandles,
    dims: &Dimensions,
    col: i32,
    row: i32,
) {
    commands.spawn((
        EmptyDisk,
        MaterialMesh2dBundle {
            mesh: mesh_handles.circle.clone().into(),
            material: material_handles.background.clone(),
            transform: get_disk_transform(dims, row, col),
            ..default()
        },
        InGame,
    ));
}

// Get the location of a disk based on its row and column
fn get_disk_transform(dims: &Dimensions, row: i32, col: i32) -> Transform {
    Transform {
        translation: Vec3::new(
            -WINDOW_WIDTH / 2.0 + (dims.col_width * col as f32 + dims.col_width / 2.0),
            WINDOW_HEIGHT / 2.0 - dims.row_height * (row as f32 + 1.5),
            0.1,
        ),
        scale: Vec3::new(dims.scale, dims.scale, 0.0),
        ..default()
    }
}

// Contains all the data of the current game
#[derive(Resource, Serialize, Deserialize, Clone)]
struct Board {
    rows: i32,
    cols: i32,
    disks: Vec<Vec<Option<Disk>>>,
}

impl Board {
    fn new(rows: i32, cols: i32) -> Self {
        let disks = vec![vec![None; rows as usize]; cols as usize];
        Self { rows, cols, disks }
    }

    // Add a disk to the board, checks there is space for it and returns the row it was added to
    fn drop_disk(&mut self, col: i32, disk: Disk) -> Option<i32> {
        if (0..self.cols).contains(&col) {
            let row = &mut self.disks[col as usize];
            if let Some(index) = row.iter().rev().position(|disk| disk.is_none()) {
                let index = row.len() - index - 1;
                row[index] = Some(disk);
                return Some(index as i32);
            }
        }
        None
    }

    // Check if the game has been won, starting from a certain disk
    fn check_for_win(&self, row: i32, col: i32, disk: Disk) -> Option<(i32, i32)> {
        // Iterate through all directions
        for &(row_delta, col_delta) in &[
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
        ] {
            let mut row = row;
            let mut col = col;
            let mut count = 1;

            for _ in 1..4 {
                row += row_delta;
                col += col_delta;
                if (0..self.rows).contains(&row) && (0..self.cols).contains(&col) {
                    match self.disks[col as usize][row as usize] {
                        Some(disk2) if disk2 == disk => count += 1,
                        _ => break,
                    }
                }
            }
            if count >= 4 {
                return Some((row, col));
            }
        }
        None
    }

    // Checks whole board for a win
    #[allow(clippy::type_complexity)]
    fn check_for_wins(&self) -> Option<(Turn, (i32, i32), (i32, i32))> {
        for row in 0..self.rows {
            for col in 0..self.cols {
                if let Some(disk) = self.disks[col as usize][row as usize] {
                    if let Some(delta) = self.check_for_win(row, col, disk) {
                        return Some((disk.to_turn(), (row, col), delta));
                    }
                }
            }
        }
        None
    }
}

#[derive(Resource, Serialize, Deserialize, Clone)]
struct MoveHistory {
    moves: Vec<(i32, Turn)>,
}

impl MoveHistory {
    fn new() -> MoveHistory {
        MoveHistory { moves: Vec::new() }
    }
}

// Used to identify the move history numbers
#[derive(Component)]
struct Move;

// For saving/loading the game
#[derive(Serialize, Deserialize)]
struct GameData {
    board: Board,
    turn: Turn,
    history: MoveHistory,
}

pub struct GamePlugin;

// Creating the plugin
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SkipClick(false))
            .insert_resource(Turn::Red)
            .insert_resource(Board::new(6, 7))
            .insert_resource(MoveHistory::new())
            .add_startup_system(setup)
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(skip_click))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(check_for_game_change)
                    .with_system(check_for_pause)
                    .with_system(check_for_click.after(check_for_game_change))
                    .with_system(check_for_mouse_movement.after(check_for_game_change))
                    .with_system(check_for_wins.after(check_for_game_change)),
            );
    }
}

// To prevent click in menu from propagating to game, called on enter
fn skip_click(mut skip_click: ResMut<SkipClick>) {
    skip_click.0 = true;
}

// Creates a completely new game
fn new_game(
    commands: &mut Commands,
    mesh_handles: &MeshHandles,
    material_handles: &MaterialHandles,
    board: &mut Board,
    turn: &mut Turn,
    move_history: &mut MoveHistory,
    rows: i32,
    cols: i32,
) {
    *board = Board::new(rows, cols);
    *turn = Turn::Red;
    *move_history = MoveHistory::new();

    let hole_dims = get_dimensions(board, HOLE_PADDING);
    let disk_dims = get_dimensions(board, DISK_PADDING);

    // Add the ghost disks (but invisible)
    let mut red_ghost_disk_color = RED_DISK_COLOR;
    red_ghost_disk_color.set_a(0.3);
    commands.spawn((
        GhostDisk::Red,
        InGame,
        MaterialMesh2dBundle {
            mesh: mesh_handles.circle.clone().into(),
            material: material_handles.red_ghost_disk.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(disk_dims.scale, disk_dims.scale, 0.0),
                ..default()
            },
            visibility: Visibility { is_visible: false },
            ..default()
        },
    ));

    let mut blue_ghost_disk_color = BLUE_DISK_COLOR;
    blue_ghost_disk_color.set_a(0.3);
    commands.spawn((
        GhostDisk::Blue,
        InGame,
        MaterialMesh2dBundle {
            mesh: mesh_handles.circle.clone().into(),
            material: material_handles.blue_ghost_disk.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(disk_dims.scale, disk_dims.scale, 0.0),
                ..default()
            },
            visibility: Visibility { is_visible: false },
            ..default()
        },
    ));

    // Board
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(
                    -WINDOW_WIDTH * ((1.0 - BOARD_SCALE.x) / 2.0),
                    WINDOW_HEIGHT * (1.0 - hole_dims.board_scale_y) / 2.0 - hole_dims.row_height,
                    0.0,
                ),
                scale: Vec3::new(
                    WINDOW_WIDTH * BOARD_SCALE.x,
                    WINDOW_HEIGHT * hole_dims.board_scale_y,
                    0.0,
                ),
                ..default()
            },
            sprite: Sprite {
                color: BOARD_COLOR,
                ..default()
            },
            ..default()
        },
        InGame,
    ));

    // Draw all the holes
    for row in 0..rows {
        for col in 0..cols {
            draw_hole(
                commands,
                mesh_handles,
                material_handles,
                &hole_dims,
                col,
                row,
            );
        }
    }
}

// Initial setup, to load all the materials and meshes
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(MaterialHandles {
        background: materials.add(ColorMaterial::from(BACKGROUND_COLOR)),
        red_disk: materials.add(ColorMaterial::from(RED_DISK_COLOR)),
        blue_disk: materials.add(ColorMaterial::from(BLUE_DISK_COLOR)),
        red_ghost_disk: materials.add(ColorMaterial::from({
            let mut ghost_red = RED_DISK_COLOR;
            ghost_red.set_a(0.3);
            ghost_red
        })),
        blue_ghost_disk: materials.add(ColorMaterial::from({
            let mut ghost_blue = BLUE_DISK_COLOR;
            ghost_blue.set_a(0.3);
            ghost_blue
        })),
    });

    commands.insert_resource(MeshHandles {
        circle: meshes.add(shape::Circle::default().into()),
    });
}

// Removes all entities in the game
fn cleanup(commands: &mut Commands, query: Query<Entity, With<InGame>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

// Checks for the game change event
fn check_for_game_change(
    mut commands: Commands,
    query: Query<Entity, With<InGame>>,
    mesh_handles: Res<MeshHandles>,
    material_handles: Res<MaterialHandles>,
    mut board: ResMut<Board>,
    mut turn: ResMut<Turn>,
    mut move_history: ResMut<MoveHistory>,
    mut game_change_events: EventReader<GameChange>,
    asset_server: Res<AssetServer>,
) {
    if let Some(game_change) = game_change_events.iter().next() {
        match game_change {
            &GameChange::New { rows, cols } => {
                // Remove all the entities in the game, then create a new one
                cleanup(&mut commands, query);
                new_game(
                    &mut commands,
                    &mesh_handles,
                    &material_handles,
                    &mut board,
                    &mut turn,
                    &mut move_history,
                    rows,
                    cols,
                );
            }
            GameChange::Save => {
                let data = GameData {
                    board: board.clone(),
                    turn: *turn,
                    history: move_history.clone(),
                };

                let Ok(file) = std::fs::File::create("save.json") else {
                    println!("Failed to create save file");
                    return;
                };
                if let Err(err) = serde_json::to_writer(file, &data) {
                    println!("Failed to write save file: {}", err);
                }
            }
            GameChange::Load => {
                let Ok(file) = std::fs::File::open("save.json") else {
                    println!("Failed to open save file");
                    return;
                };
                let Ok(data) = serde_json::from_reader::<_, GameData>(file) else {
                    println!("Failed to read save file");
                    return;
                };

                cleanup(&mut commands, query);
                new_game(
                    &mut commands,
                    &mesh_handles,
                    &material_handles,
                    &mut board,
                    &mut turn,
                    &mut move_history,
                    data.board.rows,
                    data.board.cols,
                );
                *board = data.board;
                *turn = data.turn;
                *move_history = data.history;

                let dims = get_dimensions(&board, DISK_PADDING);

                // All the data is now loaded, but the visuals need to sync
                // Add the disks and history

                // Draw all the disks
                for row in 0..board.rows {
                    for col in 0..board.rows {
                        if let Some(disk) = board.disks[col as usize][row as usize] {
                            draw_disk(
                                &mut commands,
                                &mesh_handles,
                                &material_handles,
                                &dims,
                                col,
                                row,
                                disk,
                            );
                        }
                    }
                }

                // Add history
                for (i, (col, disk)) in move_history.moves.iter().rev().enumerate() {
                    commands
                        .spawn((
                            Move,
                            NodeBundle {
                                style: Style {
                                    size: Size::new(
                                        Val::Percent(10.0),
                                        Val::Percent((1.0 - BOARD_SCALE.y) * 100.0),
                                    ),
                                    position_type: PositionType::Absolute,
                                    position: UiRect {
                                        left: Val::Percent(10.0 * i as f32),
                                        bottom: Val::Percent(0.0),
                                        ..default()
                                    },
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    align_content: AlignContent::Center,
                                    ..default()
                                },
                                ..default()
                            },
                            InGame,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    format!("{}", col + 1),
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        font_size: 40.0,
                                        color: disk.to_color(),
                                    },
                                ),
                                style: Style {
                                    align_content: AlignContent::Center,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                ..default()
                            });
                        });
                }
            }
        }
    }

    game_change_events.clear();
}

// Check for player pressing escape to go back to the main menu
fn check_for_pause(
    keyboard_input: Res<Input<KeyCode>>,
    mut game_state: ResMut<State<GameState>>,
    mut main_menu_info: ResMut<MainMenuInfo>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        main_menu_info.allow_resume = true;
        main_menu_info.winner = None;
        game_state.set(GameState::Menu).unwrap();
    }
}

// Convert the mouse position to the column in the board
fn mouse_pos_to_col(mouse_pos: Vec2, board: &Board) -> Option<i32> {
    if mouse_pos.x < WINDOW_WIDTH * BOARD_SCALE.x {
        let col = mouse_pos.x / (WINDOW_WIDTH * BOARD_SCALE.x / board.cols as f32);
        Some(col.floor() as i32)
    } else {
        None
    }
}

// Add a disk to the board and screen
fn drop_disk(
    mut commands: Commands,
    mesh_handles: &MeshHandles,
    material_handles: &MaterialHandles,
    board: &mut Board,
    turn: &mut Turn,
    history: &mut MoveHistory,
    mut query: Query<&mut Style, With<Move>>,
    asset_server: Res<AssetServer>,
    col: i32,
) {
    if let Some(row) = board.drop_disk(col, turn.to_disk()) {
        draw_disk(
            &mut commands,
            mesh_handles,
            material_handles,
            &get_dimensions(board, DISK_PADDING),
            col,
            row,
            turn.to_disk(),
        );

        // Add to history
        history.moves.push((col, *turn));

        // Shift all other history moves to the right
        for mut style in &mut query {
            style.position.left = style.position.left.try_add(Val::Percent(10.0)).unwrap();
        }

        commands
            .spawn((
                Move,
                NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Percent(10.0),
                            Val::Percent((1.0 - BOARD_SCALE.y) * 100.0),
                        ),
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            left: Val::Percent(0.0),
                            bottom: Val::Percent(0.0),
                            ..default()
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        align_content: AlignContent::Center,
                        ..default()
                    },
                    ..default()
                },
                InGame,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        format!("{}", col + 1),
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: turn.to_color(),
                        },
                    ),
                    style: Style {
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                });
            });

        turn.next();
    }
}

// Checking for placing a disk
fn check_for_click(
    commands: Commands,
    windows: Res<Windows>,
    buttons: Res<Input<MouseButton>>,
    mut board: ResMut<Board>,
    mut turn: ResMut<Turn>,
    mut history: ResMut<MoveHistory>,
    mut skip_click: ResMut<SkipClick>,
    mesh_handles: Res<MeshHandles>,
    material_handles: Res<MaterialHandles>,
    query: Query<&mut Style, With<Move>>,
    asset_server: Res<AssetServer>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if skip_click.0 {
            skip_click.0 = false;
            return;
        }
        if let Some(window) = windows.get_primary() {
            let position = window.cursor_position();
            if let Some(position) = position {
                if let Some(col) = mouse_pos_to_col(position, &board) {
                    drop_disk(
                        commands,
                        &mesh_handles,
                        &material_handles,
                        &mut board,
                        &mut turn,
                        &mut history,
                        query,
                        asset_server,
                        col,
                    );
                }
            }
        }
    }
}

// Used to display the ghost disks in the correct place
fn check_for_mouse_movement(
    windows: Res<Windows>,
    board: Res<Board>,
    turn: Res<Turn>,
    mut ghost_disk_query: Query<(&mut Transform, &mut Visibility, &GhostDisk)>,
) {
    if let Some(mouse_pos) = windows.get_primary().unwrap().cursor_position() {
        for (mut ghost_disk_transform, mut ghost_disk_visibility, &ghost_disk_type) in
            &mut ghost_disk_query
        {
            if ghost_disk_type == turn.to_ghost_disk() {
                if let Some(col) = mouse_pos_to_col(mouse_pos, &board) {
                    let Dimensions {
                        row_height,
                        col_width,
                        ..
                    } = get_dimensions(&board, 0.0);

                    // Set correct ghost disk to visible and the right loaction
                    ghost_disk_visibility.is_visible = true;
                    ghost_disk_transform.translation = Vec3::new(
                        -WINDOW_WIDTH / 2.0 + (col_width * col as f32 + col_width / 2.0),
                        WINDOW_HEIGHT / 2.0 - row_height / 2.0,
                        0.0,
                    );
                    continue;
                }
            }

            // Set all other ghost disks to invisible
            ghost_disk_visibility.is_visible = false;
        }
    }
}

// Checking for a win every frame
fn check_for_wins(
    mut commands: Commands,
    board: Res<Board>,
    mut game_state: ResMut<State<GameState>>,
    mut ghost_disks: Query<&mut Visibility, With<GhostDisk>>,
    mut main_menu_info: ResMut<MainMenuInfo>,
) {
    if let Some((winner, from, to)) = board.check_for_wins() {
        let dims = get_dimensions(&board, 0.0);

        // Drawing the win line
        let mut from = get_disk_transform(&dims, from.0, from.1).translation;
        from.z = 0.4;

        let mut to = get_disk_transform(&dims, to.0, to.1).translation;
        to.z = 0.4;

        // Winning line
        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: from + (to - from) / 2.0,
                    scale: Vec3::new(
                        (to - from).length()
                            + (dims.col_width.powf(2.0) + dims.row_height.powf(2.0)).sqrt() / 2.5,
                        (dims.col_width * WINNER_LINE_HEIGHT)
                            .min(dims.row_height * WINNER_LINE_HEIGHT),
                        0.0,
                    ),
                    rotation: Quat::from_rotation_z(
                        (from - to).angle_between(Vec3::new(1.0, 0.0, 0.0)),
                    ),
                },
                sprite: Sprite {
                    color: WINNER_COLOR,
                    ..default()
                },
                ..default()
            },
            InGame,
        ));

        // Hide all ghost disks
        for mut ghost_disk_visibility in &mut ghost_disks {
            ghost_disk_visibility.is_visible = false;
        }

        // Send to game menu with a winner
        main_menu_info.allow_resume = false;
        main_menu_info.winner = Some(winner);
        game_state.set(GameState::Menu).unwrap();
    }
}
