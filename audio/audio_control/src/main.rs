use bevy::{math::ops, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (update_speed, pause, volume))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // オーディオプレイヤーを生成し、音楽を再生する
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/Windless Slopes.ogg")),
        MyMusic,
    ));
}

/// 音楽を管理するためのカスタムコンポーネント
#[derive(Component)]
struct MyMusic;

/// 音楽の再生速度を変更するシステム
fn update_speed(music_controller: Query<&AudioSink, With<MyMusic>>, time: Res<Time>) {
    if let Ok(sink) = music_controller.get_single() {
        // 再生速度を `sin` の値に基づいて変化させる (5秒周期で速度が変動)
        sink.set_speed((ops::sin(time.elapsed_secs() / 5.0) + 1.0).max(0.1));
    }
}

/// スペースキーを押すと音楽を一時停止・再開するシステム
fn pause(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    music_controller: Query<&AudioSink, With<MyMusic>>, // 音楽の制御コンポーネントを取得
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(sink) = music_controller.get_single() {
            sink.toggle(); // 一時停止/再開を切り替え
        }
    }
}

/// 音量を調整するシステム
fn volume(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    music_controller: Query<&AudioSink, With<MyMusic>>,
) {
    if let Ok(sink) = music_controller.get_single() {
        if keyboard_input.just_pressed(KeyCode::Equal) {
            sink.set_volume(sink.volume() + 0.1); // 音量を0.1増加
        } else if keyboard_input.just_pressed(KeyCode::Minus) {
            sink.set_volume(sink.volume() - 0.1); // 音量を0.1減少
        }
    }
}