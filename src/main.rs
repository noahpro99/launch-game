use bevy::{prelude::*, sprite::collide_aabb::collide};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

const BACKGROUND_SIZE: Vec2 = Vec2::new(400.0, 700.0);
const BACKGROUND_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, -10.0);
const BOARD_SIZE: Vec2 = Vec2::new(40.0, 10.0);

const BOARD_ROWS: u8 = 10;
const BOARD_COLS: u8 = 8;

const CANNON_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const CANNONBALL_VELOCITY: f32 = 10.0;
const CANNONBALL_SIZE: Vec2 = Vec2::new(8.0, 8.0);
const CANNONBALL_COST: u32 = 30;

const BOARD_HEALTH: u8 = 3;

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
                spawn_boards,
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
        player_number: 1,
        money: STARTING_MONEY,
        state: PlayerState::WaitingForAction,
    });
    commands.spawn(Player {
        player_number: 2,
        money: STARTING_MONEY,
        state: PlayerState::WaitingForTurn,
    });
    commands.spawn(Turn { player_number: 1 });
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

fn spawn_board(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec2,
    player_num: u8,
) {
    let board_texture = asset_server.load("board.png");
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: position.extend(0.0),
                ..default()
            },
            texture: board_texture.clone(),
            sprite: Sprite {
                custom_size: Some(BOARD_SIZE),
                ..default()
            },
            ..default()
        },
        Board {
            player_number: player_num,
        },
        Breakable {
            health: BOARD_HEALTH,
        },
    ));
}

fn spawn_boards(mut commands: Commands, asset_server: Res<AssetServer>) {
    for player in 0..2 {
        for x in 0..BOARD_ROWS {
            for y in 0..BOARD_COLS {
                let x = x as f32 * BOARD_SIZE.x - BACKGROUND_SIZE.x / 2.0 + BOARD_SIZE.x / 2.0;
                let y = match player {
                    0 => y as f32 * BOARD_SIZE.y - BACKGROUND_SIZE.y / 2.0 + BOARD_SIZE.y / 2.0,
                    _ => y as f32 * -BOARD_SIZE.y + BACKGROUND_SIZE.y / 2.0 - BOARD_SIZE.y / 2.0,
                };
                spawn_board(&mut commands, &asset_server, Vec2::new(x, y), player + 1);
            }
        }
    }
}

fn spawn_cannon(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_num: u8,
    position: Vec2,
) {
    let cannon_texture = asset_server.load("cannon.png");
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: position.extend(0.0),
                ..default()
            },
            texture: cannon_texture,
            sprite: Sprite {
                custom_size: Some(CANNON_SIZE),
                flip_y: if player_num == 1 { false } else { true },
                ..default()
            },
            ..default()
        },
        Cannon {
            player_number: player_num,
            is_selected: false,
        },
    ));
}

#[derive(Component)]
struct PurchaseMenuButton;
#[derive(Component)]
struct PurchaseButton {
    item: Purchasable,
}

#[derive(Component)]
struct Player {
    player_number: u8,
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
    player_number: u8,
}

#[derive(Event)]
struct EndTurn;

#[derive(Component)]
struct Board {
    player_number: u8,
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
    player_number: u8,
    is_selected: bool,
}

#[derive(Component)]
struct CannonBall {
    player_number: u8,
}

#[derive(Component)]
struct MoneyIndicator;

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
                        player_number: cannon.player_number,
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
                if cannon.player_number == turn.single().player_number {
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
    mut breakable_query: Query<(Entity, &Transform, &mut Breakable)>,
) {
    for (cannonball_entity, cannonball_transform) in cannonball_query.iter_mut() {
        for (breakable_entity, breakable_transform, mut breakable) in breakable_query.iter_mut() {
            let collision = collide(
                cannonball_transform.translation,
                CANNONBALL_SIZE,
                breakable_transform.translation,
                BOARD_SIZE,
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
        turn.player_number = if turn.player_number == 1 { 2 } else { 1 };
        for mut player in player_query.iter_mut() {
            if player.player_number == turn.player_number {
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
                println!("player state: {:?}", player.state);
                if player.money > purchase.item.cost() {
                    player.money -= purchase.item.cost();
                    player.state = PlayerState::Placing {
                        item: purchase.item,
                    };
                    let menu = menu_query.single_mut();
                    commands.entity(menu).despawn_recursive();
                    println!("player state: {:?}", player.state);
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
                    spawn_cannon(commands, asset_server, player.player_number, world_position)
                }
                Purchasable::Board => spawn_board(
                    &mut commands,
                    &asset_server,
                    world_position,
                    player.player_number,
                ),
            }
            player.state = PlayerState::WaitingForAction;
        }
    }
}
