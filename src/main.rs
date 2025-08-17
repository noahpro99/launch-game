use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::{math::I64Vec2, prelude::*};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

const BACKGROUND_SIZE: Vec2 = Vec2::new(400.0, 800.0);
const BACKGROUND_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, -10.0);

const INITIAL_BLOCKS_HEIGHT: i32 = 5;

const GRID_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const GRID_X: i32 = (BACKGROUND_SIZE.x / GRID_SIZE.x) as i32;
const GRID_Y: i32 = (BACKGROUND_SIZE.y / GRID_SIZE.y) as i32;

const DIRT_HEALTH: u8 = 1;
const GRASS_HEALTH: u8 = 2;

const CANNON_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const CANNONBALL_VELOCITY: f32 = 10.0;
const CANNONBALL_SIZE: Vec2 = Vec2::new(8.0, 8.0);
const CANNONBALL_COST: u32 = 30;

const STARTING_MONEY: u32 = 100;
const BOARD_COST: u32 = 50;
const CANNON_COST: u32 = 100;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

const TURN_INCOME: u32 = 100;

fn main() {
    App::new()
        // set window size to background size
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default()) // ⬅ add this BEFORE the inspector
        .add_plugins(WorldInspectorPlugin::new())
        .add_event::<EndTurn>()
        .add_systems(
            Startup,
            (
                spawn_camera,
                spawn_initial_blocks,
                spawn_players,
                spawn_background,
            ),
        )
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
    commands.spawn((Camera2d, MainCamera));
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
    commands.spawn(Turn {
        player_side: PlayerSide::Bottom,
    });
}

fn spawn_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background_texture = asset_server.load("sky.png");
    commands.spawn((
        Sprite {
            image: background_texture,
            ..default()
        },
        Transform {
            translation: BACKGROUND_STARTING_POSITION,
            ..default()
        },
    ));
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
        Sprite {
            image: block_texture,
            flip_y: player_side.flip_y(),
            custom_size: Some(GRID_SIZE),
            ..default()
        },
        Transform::from_translation(from_grid_coords(grid_position).extend(0.0)),
        GlobalTransform::default(),
        Visibility::default(),
        Board { player_side },
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
        Sprite {
            image: cannon_texture,
            flip_y: player_side.flip_y(),
            custom_size: Some(CANNON_SIZE),
            ..default()
        },
        Transform::from_translation(translation.extend(0.0)),
        GlobalTransform::default(),
        Visibility::default(),
        Cannon {
            player_side,
            is_selected: false,
        },
        Grid {
            positions: vec![
                grid_position,
                grid_position + I64Vec2::new(1, 0),
                grid_position + I64Vec2::new(0, 1),
                grid_position + I64Vec2::new(1, 1),
            ],
        },
        Board { player_side },
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
    query.iter_mut().for_each(|(velocity, mut transform)| {
        transform.translation.x += velocity.0.x;
        transform.translation.y += velocity.0.y;
    });
}

fn fire_selected_cannon(
    mut commands: Commands,
    mut cannon_query: Query<(&mut Cannon, &Transform)>,
    window: Single<&Window>,
    touch: Res<ButtonInput<MouseButton>>,
    camera_q: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut player_query: Query<&mut Player>,
) {
    if !touch.just_released(MouseButton::Left) {
        return;
    }
    let (camera, camera_transform) = camera_q.into_inner();
    let mut player = player_query
        .iter_mut()
        .find(|p| p.state != PlayerState::WaitingForTurn)
        .unwrap();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        for (mut cannon, transform) in cannon_query.iter_mut() {
            if cannon.is_selected {
                if player.money < CANNONBALL_COST {
                    continue;
                }
                let cannon_position = transform.translation.truncate();
                let direction = world_position - cannon_position;
                let power = (direction.length() / 100.0).clamp(0.1, 1.0);
                // max it using min
                let velocity = -direction.normalize() * CANNONBALL_VELOCITY * power;
                cannon.is_selected = false;
                commands.spawn((
                    Sprite {
                        custom_size: Some(CANNONBALL_SIZE),
                        ..default()
                    },
                    Transform::from_translation(cannon_position.extend(0.0)),
                    GlobalTransform::default(),
                    Visibility::default(),
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
    window: Single<&Window>,
    click: Res<ButtonInput<MouseButton>>,
    camera_q: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    turn: Single<&Turn>,
) {
    if !click.just_pressed(MouseButton::Left) {
        return;
    }
    let (camera, camera_transform) = camera_q.into_inner();
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        let probe_half = Vec2::new(0.5, 0.5); // half-size of probe rectangle
        let cannon_half = CANNON_SIZE / 2.0;

        for (mut cannon, transform) in cannon_query.iter_mut() {
            let cannon_center = transform.translation.truncate();
            let collided = aabb_collision(world_position, probe_half, cannon_center, cannon_half);

            cannon.is_selected = collided && (cannon.player_side == turn.player_side);
            println!("Cannon selected: {}", cannon.is_selected);
        }
    }
}

fn aabb_collision(point: Vec2, half_size: Vec2, entity_center: Vec2, entity_half: Vec2) -> bool {
    let a = Aabb2d::new(point, half_size);
    let b = Aabb2d::new(entity_center, entity_half);
    a.intersects(&b)
}

fn cannonball_break_stuff(
    mut commands: Commands,
    mut cannonball_q: Query<(Entity, &Transform), With<CannonBall>>,
    mut breakable_q: Query<(Entity, &Transform, &Sprite, &mut Breakable)>,
) {
    for (cannonball_e, cannonball_tf) in cannonball_q.iter_mut() {
        let ball_center = cannonball_tf.translation.truncate();
        let ball_half = CANNONBALL_SIZE * 0.5; // CANNONBALL_SIZE is full size -> make half

        for (breakable_e, breakable_tf, sprite, mut breakable) in breakable_q.iter_mut() {
            // If you know custom_size is always set, keep unwrap().
            // Otherwise consider `unwrap_or(Vec2::ZERO)` or similar.
            let full_breakable_size = breakable_tf.scale.truncate() * sprite.custom_size.unwrap();
            let breakable_center = breakable_tf.translation.truncate();
            let breakable_half = full_breakable_size * 0.5;

            if aabb_collision(ball_center, ball_half, breakable_center, breakable_half) {
                breakable.health -= 1;
                commands.entity(cannonball_e).despawn();
                if breakable.health == 0 {
                    commands.entity(breakable_e).despawn();
                }
            }
        }
    }
}

fn change_turn(
    mut commands: Commands,
    mut events: EventReader<EndTurn>,
    mut turn: Single<&mut Turn>,
    mut players: Query<&mut Player>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
    mut cannons: Query<&mut Cannon>,
    menus: Query<Entity, With<Menu>>,
) {
    // Consume all EndTurn events this frame
    let flips = events.read().count();
    if flips % 2 == 0 {
        // Even number of events cancels out; nothing to do.
        return;
    }

    // Flip turn once
    turn.player_side = turn.player_side.other();

    // Update players
    for mut p in &mut players {
        if p.side == turn.player_side {
            p.state = PlayerState::WaitingForAction;
            p.money += TURN_INCOME;
        } else {
            p.state = PlayerState::WaitingForTurn;
        }
    }

    // Rotate camera 180° (PI radians)
    camera.rotate(Quat::from_rotation_z(std::f32::consts::PI));

    // Clear selection
    for mut cannon in &mut cannons {
        cannon.is_selected = false;
    }

    // Close any open menus
    for e in &menus {
        commands.entity(e).despawn();
    }
}

fn money_indicator(
    player: Single<&Player, Changed<Player>>,
    mut span_q: Single<&mut TextSpan, With<MoneyIndicator>>,
) {
    span_q.0 = player.money.to_string();
}

fn turn_done(input: Res<ButtonInput<KeyCode>>, mut event_writer: EventWriter<EndTurn>) {
    if input.just_pressed(KeyCode::KeyD) {
        event_writer.write(EndTurn);
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
fn spawn_purchase_ui(commands: &mut Commands, assets: &AssetServer) {
    // Root panel (right side, below money indicator)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                right: Val::Px(15.0),
                width: Val::Px(150.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            Menu,
        ))
        .with_children(|parent| {
            for purchasable in Purchasable::iter() {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(36.0),
                            border: UiRect::all(Val::Px(5.0)),
                            justify_content: JustifyContent::Center, // center the label
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BackgroundColor(NORMAL_BUTTON),
                        BorderRadius::MAX, // optional: rounded corners
                        PurchaseButton { item: purchasable },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(purchasable.to_string()),
                            TextFont {
                                font: assets.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            TextShadow::default(),
                        ));
                    });
            }
        });
}

fn purchase(
    mut commands: Commands,
    interactions: Query<(&Interaction, &PurchaseButton), (Changed<Interaction>, With<Button>)>,
    mut players: Query<&mut Player>,
    menu: Single<Entity, With<Menu>>,
) {
    // Find the non-waiting player (active side). Bail if none.
    let Some(mut player) = players
        .iter_mut()
        .find(|p| p.state != PlayerState::WaitingForTurn)
    else {
        return;
    };

    // Find the first button that was pressed AND is affordable; grab the item.
    if let Some(item) = interactions.iter().find_map(|(i, p)| {
        (*i == Interaction::Pressed && player.money >= p.item.cost()).then_some(p.item)
    }) {
        player.state = PlayerState::Placing { item };
        commands.entity(*menu).despawn();
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
                bg_color.0 = PRESSED_BUTTON;
            }
            Interaction::Hovered => {
                bg_color.0 = HOVERED_BUTTON;
            }
            Interaction::None => {
                bg_color.0 = NORMAL_BUTTON;
            }
        }
    }
}

fn open_close_purchase_menu(
    mut commands: Commands,
    interactions: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<PurchaseMenuButton>),
    >,
    mut players: Query<&mut Player>,
    menu: Single<Entity, With<Menu>>,
    assets: Res<AssetServer>, // pass through to your spawner
) {
    // Only act if at least one relevant button was *pressed* this frame.
    let pressed = interactions.iter().any(|i| *i == Interaction::Pressed);
    if !pressed {
        return;
    }

    // Find the active player (not waiting for their turn).
    let Some(mut player) = players
        .iter_mut()
        .find(|p| p.state != PlayerState::WaitingForTurn)
    else {
        return;
    };

    match player.state {
        PlayerState::WaitingForAction => {
            player.state = PlayerState::PurchaseMenu;
            // Your spawner signature likely: fn spawn_purchase_ui(commands: &mut Commands, assets: &AssetServer)
            spawn_purchase_ui(&mut commands, &assets);
        }
        PlayerState::PurchaseMenu => {
            player.state = PlayerState::WaitingForAction;
            commands.entity(menu.into_inner()).despawn();
        }
        _ => {}
    }
}

fn open_close_purchase_menu_text(
    buttons: Query<&Children, (With<Button>, With<PurchaseMenuButton>)>,
    players: Query<&Player>,
    children_q: Query<&Children>,
    mut spans: Query<&mut TextSpan>,
) {
    // Find active player
    let Some(player) = players
        .iter()
        .find(|p| p.state != PlayerState::WaitingForTurn)
    else {
        return;
    };

    let label = match player.state {
        PlayerState::WaitingForAction => "Purchase",
        PlayerState::PurchaseMenu => "Cancel",
        _ => "Purchase",
    };

    // Iterate Entities directly (no &)
    for btn_children in &buttons {
        for child in btn_children.iter() {
            // child: Entity
            if let Ok(text_children) = children_q.get(child) {
                for span_ent in text_children.iter() {
                    // span_ent: Entity
                    if let Ok(mut span) = spans.get_mut(span_ent) {
                        span.0 = label.to_string();
                        return;
                    }
                }
            }
            // Fallback if a TextSpan is on the button entity itself
            if let Ok(mut span) = spans.get_mut(child) {
                span.0 = label.to_string();
                return;
            }
        }
    }
}

fn place_purchase(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
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

    let (camera, camera_transform) = camera.into_inner();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        if let PlayerState::Placing { item } = player.state {
            if item == Purchasable::Cannon {
                // check if there is enough space for a cannon which takes up 2x2 grid spaces
                let grid_position = to_grid_coords(world_position);
                if is_valid_place(&grid_query, grid_position, vec![2, 2], player.side) {
                    spawn_cannon(&mut commands, &asset_server, player.side, grid_position);
                    player.money -= CANNON_COST;
                    player.state = PlayerState::WaitingForAction;
                }
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
