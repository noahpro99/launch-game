use bevy::{prelude::*, sprite::collide_aabb::collide};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

const BACKGROUND_SIZE: Vec2 = Vec2::new(400.0, 800.0);
const BACKGROUND_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, -10.0);
const BOARD_SIZE: Vec2 = Vec2::new(50.0, 50.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_event::<EndTurn>()
        .add_systems(Startup, startup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(FixedUpdate, (turn_done, change_turn, remove_boards_by_click))
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
    for i in 0..8 {
        let x = i as f32 * BOARD_SIZE.x;
        commands.spawn((SpriteBundle {
            transform: Transform {
                translation: Vec3::new(x, 0.0, 0.0),
                ..default()
            },
            sprite: Sprite {
                custom_size: Some(BOARD_SIZE),
                ..default()
            },
            ..default()
        }, Board));
    }
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
struct Board;

#[derive(Component)]
struct MainCamera;

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

fn remove_boards_by_click(
    mut commands: Commands,
    mut board_query: Query<(Entity, &Transform), With<Board>>,
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
        for (entity, transform) in board_query.iter_mut() {
            // check for collision
            let board_position = transform.translation.truncate();
            let collision = collide(
                world_position.extend(0.0),
                Vec2::new(1.0, 1.0),
                board_position.extend(0.0),
                BOARD_SIZE,
            );
            if let Some(_) = collision {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
