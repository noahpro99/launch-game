use bevy::{prelude::*, sprite::collide_aabb::collide, math::I64Vec2};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

const BACKGROUND_SIZE: Vec2 = Vec2::new(400.0, 800.0);
const BACKGROUND_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, -10.0);

const INITIAL_BLOCKS_HEIGHT: i32 = 5;

const GRID_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const GRID_X: i32 = (BACKGROUND_SIZE.x / GRID_SIZE.x) as i32;
const GRID_Y: i32 = (BACKGROUND_SIZE.y / GRID_SIZE.y) as i32;

const DIRT_HEALTH: u8 = 1;
const GRASS_HEALTH: u8 = 2;
const BOARD_HEALTH: u8 = 3;

const CANNON_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const CANNONBALL_VELOCITY: f32 = 10.0;
const CANNONBALL_SIZE: Vec2 = Vec2::new(8.0, 8.0);
const CANNONBALL_COST: u32 = 30;


const STARTING_MONEY: u32 = 100;
const BOARD_COST: u32 = 50;
const CANNON_COST: u32 = 100;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn main() {
    App::new()
        // set window size to background size
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_event::<EndTurn>()
        .add_systems(
            Startup,
            (
                spawn_camera,
                spawn_initial_blocks,
                spawn_players,
                spawn_background,
                spawn_ui,
            ),
        )
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(
            FixedUpdate,
            (
                turn_done,
                change_turn,
                apply_velocity,
                cannonball_break_stuff,
                select_cannon,
                fire_selected_cannon,
                money_indicator,
                open_close_purchase_menu,
                button_color,
                open_close_purchase_menu_text,
                place_purchase,
                purchase.after(place_purchase),
            ),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn spawn_ui(mut commands: Commands) {
    // spawn text that says that next turn is D
    commands.spawn(
        TextBundle::from_section(
            "Next turn: D",
            TextStyle {
                font_size: 20.0,
                ..default()
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            right: Val::Px(15.0),
            ..default()
        }),
    );

    // money indicator for current player
    commands.spawn((
        TextBundle::from_sections([
            TextSection {
                value: "Money: ".to_string(),
                style: TextStyle {
                    font_size: 20.0,
                    ..default()
                },
            },
            TextSection {
                value: "100".to_string(),
                style: TextStyle {
                    font_size: 20.0,
                    ..default()
                },
            },
        ])
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(30.0),
            right: Val::Px(15.0),
            ..default()
        }),
        MoneyIndicator,
    ));

    // purchase button
    commands
        .spawn(NodeBundle {
            style: Style {
                // below money indicator
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                right: Val::Px(15.0),
                width: Val::Px(150.0),

                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    PurchaseMenuButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Purchase",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
}

fn spawn_players(mut commands: Commands) {
    commands.spawn(Player {
        side: PlayerSide::Top,
        money: STARTING_MONEY,
        state: PlayerState::WaitingForAction,
    });
    commands.spawn(Player {
        side: PlayerSide::Bottom,
        money: STARTING_MONEY,
        state: PlayerState::WaitingForTurn,
    });
    commands.spawn(Turn { player_side: PlayerSide::Bottom });
}

fn spawn_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background_texture = asset_server.load("sky.png");
    commands.spawn(SpriteBundle {
        texture: background_texture,
        transform: Transform {
            translation: BACKGROUND_STARTING_POSITION,
            ..default()
        },
        sprite: Sprite {
            custom_size: Some(BACKGROUND_SIZE),
            ..default()
        },
        ..default()
    });
}

enum SingleBlockType {
    Dirt,
    Grass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerSide {
    Top,
    Bottom,
}

impl PlayerSide {
    fn other(&self) -> PlayerSide {
        match self {
            PlayerSide::Top => PlayerSide::Bottom,
            PlayerSide::Bottom => PlayerSide::Top,
        }
    }

    fn flip_y(&self) -> bool {
        match self {
            PlayerSide::Top => true,
            PlayerSide::Bottom => false,
        }
    }

    fn iter() -> impl Iterator<Item = PlayerSide> {
        [PlayerSide::Bottom, PlayerSide::Top].iter().copied()
    }
}

impl SingleBlockType {
    fn image(&self, asset_server: &Res<AssetServer>) -> Handle<Image> {
        match self {
            SingleBlockType::Dirt => asset_server.load("dirt.png"),
            SingleBlockType::Grass => asset_server.load("dirt_grass.png"),
        }
    }

    fn health(&self) -> u8 {
        match self {
            SingleBlockType::Dirt => DIRT_HEALTH,
            SingleBlockType::Grass => GRASS_HEALTH,
        }
    }
}

fn spawn_block(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    block_type: SingleBlockType,
    grid_position: I64Vec2,
    player_side: PlayerSide,
) {
    let block_texture = block_type.image(asset_server);
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: from_grid_coords(grid_position).extend(0.0),
                ..default()
            },
            texture: block_texture.clone(),
            sprite: Sprite {
                custom_size: Some(GRID_SIZE),
                flip_y: player_side.flip_y(),
                ..default()
            },
            ..default()
        },
        Board {
            player_side: player_side,
        },
        Breakable {
            health: block_type.health(),
        },
        Grid {
            positions: vec![grid_position],
        },
    ));
}

fn spawn_initial_blocks(mut commands: Commands, asset_server: Res<AssetServer>) {
    for player_side in PlayerSide::iter() {
        for x in 0..GRID_X {
            for y in 0..INITIAL_BLOCKS_HEIGHT {
                let block_type = if y == INITIAL_BLOCKS_HEIGHT - 1 {
                    SingleBlockType::Grass
                } else {
                    SingleBlockType::Dirt
                };

                let y = match player_side {
                    PlayerSide::Top => GRID_Y - y - 1,
                    PlayerSide::Bottom => y,
                };
                spawn_block(
                    &mut commands,
                    &asset_server,
                    block_type,
                    I64Vec2::new(x as i64, y as i64),
                    player_side,
                );
            }
        }
    }
}

fn spawn_cannon(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_side: PlayerSide,
    grid_position: I64Vec2,
) {
    let cannon_texture = asset_server.load("cannon.png");
    // takes up 2x2 grid spaces
    let translation_lower_left = from_grid_coords(grid_position);
    let translation = translation_lower_left + GRID_SIZE / 2.0;

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: translation.extend(0.0),
                ..default()
            },
            texture: cannon_texture,
            sprite: Sprite {
                custom_size: Some(CANNON_SIZE),
                flip_y: player_side.flip_y(),
                ..default()
            },
            ..default()
        },
        Cannon {
            player_side,
            is_selected: false,
        },
        Grid {
            positions: vec![grid_position, grid_position + I64Vec2::new(1, 0), grid_position + I64Vec2::new(0, 1), grid_position + I64Vec2::new(1, 1)],
        },
        Board {
            player_side,
        },
    ));
}

fn from_grid_coords(position: I64Vec2) -> Vec2 {
    Vec2::new(
        position.x as f32 * GRID_SIZE.x - BACKGROUND_SIZE.x / 2.0 + GRID_SIZE.x / 2.0,
        position.y as f32 * GRID_SIZE.y - BACKGROUND_SIZE.y / 2.0 + GRID_SIZE.y / 2.0,
    )
}

fn to_grid_coords(position: Vec2) -> I64Vec2 {
    I64Vec2::new(
        ((position.x + BACKGROUND_SIZE.x / 2.0) / GRID_SIZE.x).floor() as i64,
        ((position.y + BACKGROUND_SIZE.y / 2.0) / GRID_SIZE.y).floor() as i64,
    ) 
}

#[derive(Component)]
struct PurchaseMenuButton;
#[derive(Component)]
struct PurchaseButton {
    item: Purchasable,
}

#[derive(Component)]
struct Player {
    side: PlayerSide,
    money: u32,
    state: PlayerState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerState {
    WaitingForAction,
    WaitingForTurn,
    PurchaseMenu,
    Placing { item: Purchasable },
}

#[derive(Component)]
struct Turn {
    player_side: PlayerSide,
}

#[derive(Event)]
struct EndTurn;

#[derive(Component)]
struct Board {
    player_side: PlayerSide,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Breakable {
    health: u8,
}

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Menu;

#[derive(Component)]
struct Cannon {
    player_side: PlayerSide,
    is_selected: bool,
}

#[derive(Component)]
struct CannonBall {
    player_side: PlayerSide,
}

#[derive(Component)]
struct MoneyIndicator;

#[derive(Component)]
struct Grid {
    positions: Vec<I64Vec2>,
}

fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>) {
    query.for_each_mut(|(velocity, mut transform)| {
        transform.translation.x += velocity.0.x;
        transform.translation.y += velocity.0.y;
    });
}

fn fire_selected_cannon(
    mut commands: Commands,
    mut cannon_query: Query<(&mut Cannon, &Transform)>,
    windows: Query<&Window>,
    touch: Res<Input<MouseButton>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut player_query: Query<&mut Player>,
) {
    if !touch.just_released(MouseButton::Left) {
        return;
    }
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    let mut player = player_query
        .iter_mut()
        .find(|p| p.state != PlayerState::WaitingForTurn)
        .unwrap();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        for (mut cannon, transform) in cannon_query.iter_mut() {
            if cannon.is_selected {
                if player.money < CANNONBALL_COST {
                    continue;
                }
                let cannon_position = transform.translation.truncate();
                let direction = world_position - cannon_position;
                let power = (direction.length() / 100.0).min(1.0).max(0.1);
                // max it using min
                let velocity = -direction.normalize() * CANNONBALL_VELOCITY * power;
                cannon.is_selected = false;
                commands.spawn((
                    SpriteBundle {
                        transform: Transform {
                            translation: cannon_position.extend(0.0),
                            ..default()
                        },
                        sprite: Sprite {
                            custom_size: Some(CANNONBALL_SIZE),
                            ..default()
                        },
                        ..default()
                    },
                    Velocity(velocity),
                    CannonBall {
                        player_side: cannon.player_side,
                    },
                    Collider,
                ));
                player.money -= CANNONBALL_COST;
            }
        }
    }
}

fn select_cannon(
    mut cannon_query: Query<(&mut Cannon, &Transform)>,
    windows: Query<&Window>,
    click: Res<Input<MouseButton>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    turn: Query<&Turn>,
) {
    if !click.just_pressed(MouseButton::Left) {
        return;
    }
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        for (mut cannon, transform) in cannon_query.iter_mut() {
            // check for collision
            let cannon_position = transform.translation.truncate();
            let collision = collide(
                world_position.extend(0.0),
                Vec2::new(1.0, 1.0),
                cannon_position.extend(0.0),
                CANNON_SIZE,
            );
            if let Some(_) = collision {
                if cannon.player_side == turn.single().player_side {
                    cannon.is_selected = true;
                }
                println!("Cannon selected: {}", cannon.is_selected);
            } else {
                cannon.is_selected = false;
            }
        }
    }
}

fn cannonball_break_stuff(
    mut commands: Commands,
    mut cannonball_query: Query<(Entity, &Transform), With<CannonBall>>,
    mut breakable_query: Query<(Entity, &Transform, &Sprite, &mut Breakable)>,
) {
    for (cannonball_entity, cannonball_transform) in cannonball_query.iter_mut() {
        for (breakable_entity, breakable_transform, sprite, mut breakable) in breakable_query.iter_mut() {
            let collision = collide(
                cannonball_transform.translation,
                CANNONBALL_SIZE,
                breakable_transform.translation,
                breakable_transform.scale.truncate() * sprite.custom_size.unwrap(),
            );
            if let Some(_) = collision {
                breakable.health -= 1;
                commands.entity(cannonball_entity).despawn_recursive();
                if breakable.health == 0 {
                    commands.entity(breakable_entity).despawn_recursive();
                }
            }
        }
    }
}

fn change_turn(
    mut commands: Commands,
    mut events: EventReader<EndTurn>,
    mut turn_query: Query<&mut Turn>,
    mut player_query: Query<&mut Player>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut cannon_query: Query<&mut Cannon>,
    mut menu_query: Query<Entity, With<Menu>>,
) {
    let mut turn = turn_query.single_mut();
    let mut camera = camera_query.single_mut();
    events.iter().for_each(|_| {
        turn.player_side = turn.player_side.other();
        for mut player in player_query.iter_mut() {
            if player.side == turn.player_side {
                player.state = PlayerState::WaitingForAction;
                player.money += 100;
            } else {
                player.state = PlayerState::WaitingForTurn;
            }
        }
        camera.rotate(Quat::from_rotation_z(std::f32::consts::PI));
        for mut cannon in cannon_query.iter_mut() {
            cannon.is_selected = false;
        }
        for menu in menu_query.iter_mut() {
            commands.entity(menu).despawn_recursive();
        }
    });
}

fn money_indicator(
    mut query: Query<&Player, Changed<Player>>,
    mut money_indicator_query: Query<&mut Text, With<MoneyIndicator>>,
) {
    let mut money_indicator = money_indicator_query.single_mut();
    for player in query.iter_mut() {
        if player.state != PlayerState::WaitingForTurn {
            money_indicator.sections[1].value = player.money.to_string();
        }
    }
}

fn turn_done(input: Res<Input<KeyCode>>, mut event_writer: EventWriter<EndTurn>) {
    if input.just_pressed(KeyCode::D) {
        event_writer.send(EndTurn);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Purchasable {
    Cannon,
    Board,
}

impl Purchasable {
    fn cost(&self) -> u32 {
        match self {
            Purchasable::Cannon => CANNON_COST,
            Purchasable::Board => BOARD_COST,
        }
    }

    fn iter() -> impl Iterator<Item = Purchasable> {
        [Purchasable::Cannon, Purchasable::Board].iter().copied()
    }
}

impl ToString for Purchasable {
    fn to_string(&self) -> String {
        match self {
            Purchasable::Cannon => "Cannon".to_string(),
            Purchasable::Board => "Board".to_string(),
        }
    }
}

// show purchasing ui when a player state changes to purchasing which shows all the items that can be purchased
fn spawn_purchase_ui(commands: &mut Commands) {
    // spawn ui for purchasing
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    // put on right side below money indicator
                    position_type: PositionType::Absolute,
                    top: Val::Px(100.0),
                    right: Val::Px(15.0),
                    width: Val::Px(150.0),
                    ..default()
                },
                ..default()
            },
            Menu,
        ))
        .with_children(|parent| {
            for purchasable in Purchasable::iter() {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                border: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            border_color: BorderColor(Color::BLACK),
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        PurchaseButton { item: purchasable },
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            purchasable.to_string(),
                            TextStyle {
                                font_size: 20.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ));
                    });
            }
        });
}

fn purchase(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &PurchaseButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut player_query: Query<&mut Player>,
    mut menu_query: Query<Entity, With<Menu>>,
) {
    let mut player = player_query
        .iter_mut()
        .find(|p| p.state != PlayerState::WaitingForTurn)
        .unwrap();

    for (interaction, purchase) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if player.money > purchase.item.cost() {
                    player.state = PlayerState::Placing {
                        item: purchase.item,
                    };
                    let menu = menu_query.single_mut();
                    commands.entity(menu).despawn_recursive();
                }
            }
            _ => {}
        }
    }
}

fn button_color(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                bg_color.0 = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                bg_color.0 = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                bg_color.0 = NORMAL_BUTTON.into();
            }
        }
    }
}

fn open_close_purchase_menu(
    mut commands: Commands,
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<PurchaseMenuButton>),
    >,
    mut player_query: Query<&mut Player>,
    mut menu_query: Query<Entity, With<Menu>>,
) {
    let mut player = player_query
        .iter_mut()
        .find(|p| p.state != PlayerState::WaitingForTurn)
        .unwrap();

    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => match player.state {
                PlayerState::WaitingForAction => {
                    player.state = PlayerState::PurchaseMenu;
                    spawn_purchase_ui(&mut commands);
                }
                PlayerState::PurchaseMenu => {
                    player.state = PlayerState::WaitingForAction;
                    let menu = menu_query.single_mut();
                    commands.entity(menu).despawn_recursive();
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn open_close_purchase_menu_text(
    mut interaction_query: Query<&Children, (With<Button>, With<PurchaseMenuButton>)>,
    mut player_query: Query<&Player>,
    mut text_query: Query<&mut Text>,
) {
    let player = player_query
        .iter_mut()
        .find(|p| p.state != PlayerState::WaitingForTurn)
        .unwrap();

    for children in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        text.sections[0].value = match player.state {
            PlayerState::WaitingForAction => "Purchase".to_string(),
            PlayerState::PurchaseMenu => "Cancel".to_string(),
            _ => "Purchase".to_string(),
        }
    }
}

fn place_purchase(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    grid_query: Query<&Grid>,
    mut player_query: Query<&mut Player>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let mut player = player_query
        .iter_mut()
        .find(|p| p.state != PlayerState::WaitingForTurn)
        .unwrap();

    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        if let PlayerState::Placing { item } = player.state {
            match item {
                Purchasable::Cannon => {
                    // check if there is enough space for a cannon which takes up 2x2 grid spaces
                    let grid_position = to_grid_coords(world_position);
                    if is_valid_place(&grid_query, grid_position, vec![2, 2], player.side) {
                        spawn_cannon(
                            &mut commands,
                            &asset_server,
                            player.side,
                            grid_position,
                        );
                        player.money -= CANNON_COST;
                        player.state = PlayerState::WaitingForAction;
                    }
                }
                // Purchasable::Board => spawn_board(
                //     &mut commands,
                //     &asset_server,
                //     world_position,
                //     player.player_number,
                // ),
                _ => {}
            }
        }
    }
}


/// loop and check there isn't anything else and also only on that players side
fn is_valid_place(
    grid_query: &Query<&Grid>,
    grid_position: I64Vec2,
    size: Vec<u8>,
    player_side: PlayerSide,
) -> bool {
    let mut valid = true;
    for x in 0..size[0] {
        for y in 0..size[1] {
            let position = grid_position + I64Vec2::new(x as i64, y as i64);
            let grid = grid_query
                .iter()
                .find(|grid| grid.positions.contains(&position));
            if grid.is_some() {
                valid = false;
            }
        }
    }

    // check if on the correct side
    match player_side {
        PlayerSide::Top => {
            if grid_position.y < (GRID_Y / 2) as i64 {
                valid = false;
            }
        }
        PlayerSide::Bottom => {
            if grid_position.y + size[1] as i64 > (GRID_Y / 2) as i64 {
                valid = false;
            }
        }
    }
    valid
}