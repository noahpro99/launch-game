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

fn main() {
    App::new()
        // set window size to background size
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_event::<EndTurn>()
        .add_systems(Startup, startup)
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
            ),
        )
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 100.0),
                ..default()
            },
            ..default()
        },
        MainCamera,
    ));

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
    commands.spawn(Player { player_number: 1 });
    commands.spawn(Player { player_number: 2 });
    commands.spawn(Turn { player_number: 1 });

    // boards
    let board_texture = asset_server.load("board.png");
    for player in 0..2 {
        for x in 0..BOARD_ROWS {
            for y in 0..BOARD_COLS {
                let x = x as f32 * BOARD_SIZE.x - BACKGROUND_SIZE.x / 2.0 + BOARD_SIZE.x / 2.0;
                let y = match player {
                    0 => y as f32 * BOARD_SIZE.y - BACKGROUND_SIZE.y / 2.0 - BOARD_SIZE.y / 2.0,
                    _ => {
                        y as f32 * BOARD_SIZE.y + BACKGROUND_SIZE.y / 2.0 + BOARD_SIZE.y / 2.0
                            - BOARD_ROWS as f32 * BOARD_SIZE.y
                    }
                };
                commands.spawn((
                    SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(x, y, 0.0),
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
                        player_number: player + 1,
                    },
                    Breakable { health: 3 },
                ));
            }
        }
    }

    // cannon
    let cannon_texture = asset_server.load("cannon.png");
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -BACKGROUND_SIZE.y / 2.0 + CANNON_SIZE.y / 2.0 + BOARD_ROWS as f32 * BOARD_SIZE.y, 0.0),
                ..default()
            },
            texture: cannon_texture,
            sprite: Sprite {
                custom_size: Some(CANNON_SIZE),
                ..default()
            },
            ..default()
        },
        Cannon {
            player_number: 1,
            is_selected: false,
        },
    ));
}

#[derive(Component)]
struct Player {
    player_number: u8,
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
struct Cannon {
    player_number: u8,
    is_selected: bool,
}

#[derive(Component)]
struct CannonBall {
    player_number: u8,
}

fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>) {
    query.for_each_mut(|(velocity, mut transform)| {
        transform.translation.x += velocity.0.x;
        transform.translation.y += velocity.0.y;
    });
}

fn fire_selected_cannon(
    mut commands: Commands,
    mut cannon_query: Query<(&Cannon, &Transform)>,
    windows: Query<&Window>,
    click: Res<Input<MouseButton>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if !click.just_released(MouseButton::Left) {
        return;
    }
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        for (cannon, transform) in cannon_query.iter_mut() {
            if cannon.is_selected {
                let cannon_position = transform.translation.truncate();
                let direction = world_position - cannon_position;
                let power = (direction.length() / 100.0).min(1.0).max(0.1);
                // max it using min
                let velocity = - direction.normalize() * CANNONBALL_VELOCITY * power;
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
            }
        }
    }
}

fn select_cannon(
    mut cannon_query: Query<(&mut Cannon, &Transform)>,
    windows: Query<&Window>,
    click: Res<Input<MouseButton>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
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
                cannon.is_selected = true;
            } else {
                cannon.is_selected = false;
            }
            println!("Cannon selected: {}", cannon.is_selected);
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

fn change_turn(mut events: EventReader<EndTurn>, mut turn_query: Query<&mut Turn>) {
    let mut turn = turn_query.single_mut();
    events.iter().for_each(|_| {
        turn.player_number = if turn.player_number == 1 { 2 } else { 1 };
        println!("Turn: {}", turn.player_number);
    });
}

fn turn_done(
    mut commands: Commands,
    turn: Query<&Turn>,
    input: Res<Input<KeyCode>>,
    mut event_writer: EventWriter<EndTurn>,
) {
    if input.just_pressed(KeyCode::D) {
        let turn = turn.single();
        commands.spawn(Player {
            player_number: turn.player_number,
        });
        event_writer.send(EndTurn);
    }
}
