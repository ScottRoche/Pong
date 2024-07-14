use bevy::{prelude::*, sprite::MaterialMesh2dBundle, color::palettes::basic::WHITE};

struct KeyMap {
    up: KeyCode,
    down: KeyCode
}

#[derive(Component)]
struct Ball;

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
    commands.spawn((ball, Ball));
}

fn update(time: Res<Time>,
          keyboard_input: Res<ButtonInput<KeyCode>>,
          mut query: Query<(&PlayerController, &mut Transform)>) {
    for (controller, mut transform) in &mut query {
        controller.translate(time.delta_seconds(), &keyboard_input, &mut transform)
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, init)
        .add_systems(Update, update)
        .run();
}
