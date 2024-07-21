use bevy::{color::palettes::basic::WHITE, prelude::*, sprite::MaterialMesh2dBundle};
use avian2d::prelude::*;

struct KeyMap {
    up: KeyCode,
    down: KeyCode
}

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Velocity {
    speed: f32,
    direction: Vec2,
}

#[derive(Component)]
struct PlayerController {
    keymap: KeyMap
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
        mut materials: ResMut<Assets<ColorMaterial>>) {
    info!("Starting");

    commands.spawn(Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0., 0., 0.)),
            ..default()
        },
        ..default()
    });

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
        if offset_scale.x < 0. {
            CollisionSide::Left
        }
        else {
            CollisionSide::Right
        }
    }
    else if offset_scale.y == offset_distance.y {
        if offset_scale.y < 0. {
            CollisionSide:: Bottom
        }
        else {
            CollisionSide::Top
        }
    }
    else {
        warn!("offset: {} {}", offset.x, offset.y);
        warn!("effector size: {} {}", effector.size().x, effector.size().y);
        warn!("effected size: {} {}", effected.size().x, effected.size().y);
        warn!("offset scale: {} {}", offset_scale.x, offset_scale.y);
        panic!();
    };

    side
}

fn detect_collision(mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
                    collider_query: Query<&Transform, With<PlayerController>>) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();
    let ball_collider = ColliderAabb::new(ball_transform.translation.truncate(), ball_transform.scale.truncate() / 2.);

    debug!("Colliders: {}", collider_query.iter().len());

    for transform in &collider_query {
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
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, init)
        .add_systems(Update, (ball_move, paddles_move, detect_collision).chain())
        .run();
}
