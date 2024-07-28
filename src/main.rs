use bevy::{color::palettes::basic::WHITE, prelude::*, sprite::MaterialMesh2dBundle};
use avian2d::prelude::*;
use rand::Rng;

struct KeyMap {
    up: KeyCode,
    down: KeyCode
}

#[derive(Component)]
struct ArenaWall;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Velocity {
    speed: f32,
    direction: Vec2,
}

#[derive(Component)]
struct PlayerController {
    id: i8,
    keymap: KeyMap
}

#[derive(Component)]
struct GameMode {
    player1_score: u8,
    player2_score: u8
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    RoundStart,
    Playing,
    RoundFinished,
    GameFinished
}

impl PlayerController {
    fn translate(&self,
                 delta_time: f32,
                 input: &Res<ButtonInput<KeyCode>>,
                 transform: &mut Transform) {
        if input.pressed(self.keymap.up) {
            transform.translation.y += 150. * delta_time;
        }

        if input.pressed(self.keymap.down) {
            transform.translation.y -= 150. * delta_time;
        }
    }
}

fn init(mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        mut next_state: ResMut<NextState<GameState>>) {
    info!("Starting");

    commands.spawn(Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0., 0., 0.)),
            ..default()
        },
        ..default()
    });

    commands.spawn(GameMode { player1_score: 0, player2_score: 0 });

    let player1_position: Vec3 = Vec3::new(64., 0., 0.);
    let player1_paddle = MaterialMesh2dBundle {
        mesh: meshes.add(Rectangle::default()).into(),
        transform: Transform::default()
            .with_translation(player1_position)
            .with_scale(Vec3::new(16., 128., 0.)),
        material: materials.add(Color::from(WHITE)),
        ..default()
    };
    commands.spawn((player1_paddle,
                    PlayerController {
                        id: 0,
                        keymap: KeyMap {
                            up: KeyCode::ArrowUp,
                            down: KeyCode::ArrowDown,
                        },
                    }));

    let player2_position: Vec3 = Vec3::new(-64., 0., 0.);
    let player2_paddle = MaterialMesh2dBundle {
        mesh: meshes.add(Rectangle::default()).into(),
        transform: Transform::default()
            .with_translation(player2_position)
            .with_scale(Vec3::new(16., 128., 0.)),
        material: materials.add(Color::from(WHITE)),
        ..default()
    };

    commands.spawn((player2_paddle,
                    PlayerController {
                        id: 1,
                        keymap: KeyMap {
                            up: KeyCode::KeyW,
                            down: KeyCode::KeyS,
                        },
                    }));

    let ball = MaterialMesh2dBundle {
        mesh: meshes.add(Rectangle::default()).into(),
        transform: Transform::default()
            .with_scale(Vec3::splat(16.)),
        material: materials.add(Color::from(WHITE)),
        ..default()
    };

    commands.spawn((ball,
                    Velocity {
                        speed: 100.,
                        direction: Vec2::new(1., 1.)
                    },
                    Ball));

    // Arena
    commands.spawn((ColliderAabb::new(Vec2::new(0., 250.), Vec2::new(250., 8.)), ArenaWall));
    commands.spawn((ColliderAabb::new(Vec2::new(0., -250.), Vec2::new(250., 8.)), ArenaWall));

    commands.spawn((ColliderAabb::new(Vec2::new(250., 0.), Vec2::new(8., 250.)), ArenaWall));
    commands.spawn((ColliderAabb::new(Vec2::new(-250., 0.), Vec2::new(8., 250.)), ArenaWall));

    next_state.set(GameState::Playing);
}


fn paddles_move(time: Res<Time>,
          keyboard_input: Res<ButtonInput<KeyCode>>,
          mut query: Query<(&PlayerController, &mut Transform)>) {
    for (controller, mut transform) in &mut query {
        controller.translate(time.delta_seconds(), &keyboard_input, &mut transform)
    }
}

fn ball_move(time: Res<Time>, mut ball_query: Query<(&Velocity, &mut Transform), With<Ball>>) {
    let (velocity, mut transform) = ball_query.single_mut();
    transform.translation.x += time.delta_seconds() * velocity.speed * velocity.direction.x;
    transform.translation.y += time.delta_seconds() * velocity.speed * velocity.direction.y;
}

enum CollisionSide {
    Top,
    Bottom,
    Left,
    Right
}

fn calculate_collision_side(effector: ColliderAabb, effected: ColliderAabb) -> CollisionSide
{
    let offset = effector.center() - effected.center(); /* I know with this how far away the
                                                         * centres of the colliders are. */
    let offset_distance = offset.abs().ceil();

    // we need to know where on the effected collider that the effector hit.
    // i.e. if the x scale of the effected and effector = the x value of the offset we are on a
    // side.

    let offset_scale = (effector.size() + effected.size()) / 2.;

    let side = if offset_scale.x == offset_distance.x {
        if offset.x < 0. {
            CollisionSide::Left
        }
        else {
            CollisionSide::Right
        }
    } else if offset_scale.y == offset_distance.y {
        if offset.y < 0. {
            CollisionSide:: Bottom
        }
        else {
            CollisionSide::Top
        }
    } else {
        // FIXME: When the player's paddle is moving towards the colliding ball
        // this will get triggered because the paddle moves over the ball's
        // collider.
        warn!("effected pos: {} {}", effected.center().x, effected.center().y);
        warn!("offset: {} {}", offset.x, offset.y);
        warn!("effector size: {} {}", effector.size().x, effector.size().y);
        warn!("effected size: {} {}", effected.size().x, effected.size().y);
        warn!("offset scale: {} {}", offset_scale.x, offset_scale.y);
        panic!();
    };

    side
}

fn detect_collision(mut game_query: Query<&mut GameMode>,
                    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
                    paddle_query: Query<&Transform, With<PlayerController>>,
                    arena_query: Query<&ColliderAabb, With<ArenaWall>>,
                    mut next_state: ResMut<NextState<GameState>>) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();
    let ball_collider = ColliderAabb::new(ball_transform.translation.truncate(), ball_transform.scale.truncate() / 2.);

    let mut game_mode = game_query.single_mut();

    debug!("Colliders: {}", paddle_query.iter().len());

    for transform in &paddle_query {
        debug!("Checking collision");
        let collider = ColliderAabb::new(transform.translation.truncate(), transform.scale.truncate() / 2.);
        if !collider.intersects(&ball_collider) {
            debug!("No collision detected");
            continue;
        }

        debug!("Collision detected");

        // update balls velocity after the collision
        let impact_side: CollisionSide = calculate_collision_side(ball_collider, collider);

        // reflect the velocity based on the collision side
        match impact_side {
            CollisionSide::Top | CollisionSide::Bottom => ball_velocity.direction.y = -ball_velocity.direction.y,
            CollisionSide::Right | CollisionSide::Left => ball_velocity.direction.x = -ball_velocity.direction.x,
        }
    }

    for &wall_collider in &arena_query {
        debug!("Checking collision");
        if !wall_collider.intersects(&ball_collider) {
            debug!("No collision detected");
            continue;
        }

        debug!("Collision detected");

        // update balls velocity after the collision
        let impact_side: CollisionSide = calculate_collision_side(ball_collider, wall_collider);

        // reflect the velocity based on the collision side
        match impact_side {
            CollisionSide::Top | CollisionSide::Bottom => ball_velocity.direction.y = -ball_velocity.direction.y,
            CollisionSide::Right => {
                ball_velocity.direction.x = -ball_velocity.direction.x;
                game_mode.player1_score += 1;
                info!("Player 1 scored: {}", game_mode.player1_score);
                next_state.set(GameState::RoundFinished);
            },
            CollisionSide::Left => {
                ball_velocity.direction.x = -ball_velocity.direction.x;
                game_mode.player2_score += 1;
                info!("Player 2 scored: {}", game_mode.player2_score);
                next_state.set(GameState::RoundFinished);
            }
        }
    }
}

fn paddles_reset(mut paddle_query: Query<(&mut Transform, &PlayerController)>) {
    for (mut transform, controller) in &mut paddle_query {
        if controller.id == 0 {
            transform.translation = Vec3::new(64., 0., 0.);
        } else {
            transform.translation = Vec3::new(-64., 0., 0.);
        }
    }
}

fn ball_reset(mut ball_query: Query<(&mut Velocity, &mut Transform), With<Ball>>) {
    let (mut velocity, mut transform) = ball_query.single_mut();

    let mut rng = rand::thread_rng();

    velocity.direction = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize();
    transform.translation = Vec3::splat(0.);
}

fn start_round(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}

fn check_score(game_query: Query<&mut GameMode>,
               mut next_state: ResMut<NextState<GameState>>) {
    let game_mode = game_query.single();

    if game_mode.player1_score == 5 {
        info!("Player 1 Wins!");
        info!("Final score: {} - {}", game_mode.player1_score, game_mode.player2_score);
        next_state.set(GameState::GameFinished);
    } else if game_mode.player2_score == 5 {
        info!("Player 2 Wins!");
        info!("Final score: {} - {}", game_mode.player1_score, game_mode.player2_score);
        next_state.set(GameState::GameFinished);
    } else {
        next_state.set(GameState::RoundStart);
    }
}

fn end_game(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit::Success);
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct RoundStartSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct PlayingSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct RoundFinishedSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct GameFinishedSet;

fn main() {
    let mut app = App::new();

    app.configure_sets(Update, (
            RoundStartSet
                .run_if(in_state(GameState::RoundStart)),
            PlayingSet
                .run_if(in_state(GameState::Playing)),
            RoundFinishedSet
                .run_if(in_state(GameState::RoundFinished)),
            GameFinishedSet
                .run_if(in_state(GameState::GameFinished))
    ));

    app.add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, init)
        .add_systems(Update, (
            (
                (
                    (
                        ball_reset,
                        paddles_reset
                    ),
                    start_round
                ).chain()
            ).in_set(RoundStartSet),
            (
                ball_move,
                paddles_move,
                detect_collision,
            ).chain().in_set(PlayingSet),
            (
                check_score
            ).in_set(RoundFinishedSet),
            (
                end_game
            ).in_set(GameFinishedSet)
        ))
        .insert_state(GameState::RoundStart)
        .run();
}
