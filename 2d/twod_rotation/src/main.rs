use bevy::{math::ops, prelude::*};

// ゲームの境界を定義
const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Time::<Fixed>::from_hz(60.0)) // 固定時間ステップを60Hzに設定
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                player_movement_system, // プレイヤーの移動システム
                snap_to_player_system,  // 敵がプレイヤーに即座を向くシステム
                rotate_to_player_system // 敵が徐々にプレイヤーに向くシステム
            )
        )
        .run();
}

/// プレイヤーコンポーネント
#[derive(Component)]
struct Player {
    movement_speed: f32, // 移動速度 (メートル/秒)
    rotation_speed: f32, // 回転速度 (ラジアン/秒)
}

/// プレイヤーの方向へ即座に向く敵のコンポーネント
#[derive(Component)]
struct SnapToPlayer;

/// プレイヤーの方向へ徐々に回転する敵のコンポーネント
#[derive(Component)]
struct RotateToPlayer {
    rotation_speed: f32, //  回転速度 (rad/s)
}

/// ゲームのエンティティを追加し、2Dレンダリング用の直交カメラを作成する。
/// 
/// Bevy の座標系は 2D と 3D で共通で、2D では以下のようになる：
/// 
/// * `X` 軸は左から右へ (`+X` は右方向)
/// * `Y` 軸は下から上へ (`+Y` は上方向)
/// * `Z` 軸は奥から手前へ (`+Z` は画面外から手前方向)
/// 
/// 原点は画面の中心
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ship_handle = asset_server.load("textures/simplespace/ship_C.png");
    let enemy_a_handle = asset_server.load("textures/simplespace/enemy_A.png");
    let enemy_b_handle = asset_server.load("textures/simplespace/enemy_B.png");

    // 2D直交カメラの作成
    commands.spawn(Camera2d);

    let horizontal_margin = BOUNDS.x / 4.0;
    let vertical_margin = BOUNDS.y / 4.0;

    // プレイヤーの宇宙船
    commands.spawn((
        Sprite::from_image(ship_handle),
        Player {
            movement_speed: 500.0,                        // メートル/秒
            rotation_speed: f32::to_radians(360.0), // 度/秒
        },
    ));

    // SnapToPlayerの敵を作成 (即座にプレイヤーを向く)
    commands.spawn((
        Sprite::from_image(enemy_a_handle.clone()),
        Transform::from_xyz(0.0 - horizontal_margin, 0.0, 0.0),
        SnapToPlayer,
    ));
    commands.spawn((
        Sprite::from_image(enemy_a_handle),
        Transform::from_xyz(0.0, 0.0 - vertical_margin, 0.0),
        SnapToPlayer,
    ));

    // RotateToPlayerの敵を生成 (徐々に回転する)
    commands.spawn((
        Sprite::from_image(enemy_b_handle.clone()),
        Transform::from_xyz(0.0 + horizontal_margin, 0.0, 0.0),
        RotateToPlayer {
            rotation_speed: f32::to_radians(45.0), // 度/秒
        },
    ));
    commands.spawn((
        Sprite::from_image(enemy_b_handle),
        Transform::from_xyz(0.0, 0.0 + vertical_margin, 0.0),
        RotateToPlayer {
            rotation_speed: f32::to_radians(90.0), // 度/秒
        },
    ));
}

/// キーボード入力に基づいて回転と移動を適用する
fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Single<(&Player, &mut Transform)>,
) {
    let (ship, mut transform) = query.into_inner();
    let mut rotation_factor = 0.0;
    let mut movement_factor = 0.0;

    // キー入力による回転と移動の制御
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        rotation_factor += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        rotation_factor -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        movement_factor += 1.0;
    }

    // Z軸回転
    transform.rotate_z(rotation_factor * ship.rotation_speed * time.delta_secs());

    // 現在の向きに基づいて移動
    let movement_direction = transform.rotation * Vec3::Y;
    let movement_distance = movement_factor * ship.movement_speed * time.delta_secs();
    let translation_delta = movement_direction * movement_distance;
    transform.translation += translation_delta;

    // 画面の境界内に収める
    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
}

/// 敵が即座にプレイヤーを向くシステム
fn snap_to_player_system(
    mut query: Query<&mut Transform, (With<SnapToPlayer>, Without<Player>)>,
    player_transform: Single<&Transform, With<Player>>,
) {
    let player_translation = player_transform.translation.xy();

    for mut enemy_transform in &mut query {
        let to_player = (player_translation - enemy_transform.translation.xy()).normalize();
        let rotate_to_player = Quat::from_rotation_arc(Vec3::Y, to_player.extend(0.));
        enemy_transform.rotation = rotate_to_player;
    }
}

/// 敵が徐々にプレイヤーを向くシステム
/// 
/// `acos` は -1.0 から 1.0 の間で動作するため
/// 浮動小数点誤差による NaN を防ぐために `clamp` する。
fn rotate_to_player_system(
    time: Res<Time>,
    mut query: Query<(&RotateToPlayer, &mut Transform), Without<Player>>,
    player_transform: Single<&Transform, With<Player>>,
) {
    let player_translation = player_transform.translation.xy();

    for (config, mut enemy_transform) in &mut query {
        let enemy_forward = (enemy_transform.rotation * Vec3::Y).xy();
        let to_player = (player_translation - enemy_transform.translation.xy()).normalize();
        let forward_dot_player = enemy_forward.dot(to_player);

        if (forward_dot_player - 1.0).abs() < f32::EPSILON {
            continue;
        }

        let enemy_right = (enemy_transform.rotation * Vec3::X).xy();
        let right_dot_player = enemy_right.dot(to_player);
        let rotation_sign = -f32::copysign(1.0, right_dot_player);
        let max_angle = ops::acos(forward_dot_player.clamp(-1.0, 1.0));

        let rotation_angle =
            rotation_sign * (config.rotation_speed * time.delta_secs()).min(max_angle);

        enemy_transform.rotate_z(rotation_angle);
    }
}