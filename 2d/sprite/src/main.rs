use bevy::prelude::*; // Bevy の基本的な機能をインポート

fn main() {
    App::new() // 新しい Bevy アプリケーションを作成
        .add_plugins(DefaultPlugins) // デフォルトのプラグインを追加 (レンダリングやアセット管理など)
        .add_systems(Startup, setup) // スタートアップ時に `setup` システムを実行
        .run(); // アプリケーションを作成
}

// 初期セットアップ (カメラとスプライトを追加)
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 2D カメラをスポーン (画面に何かを描画するために必要)
    commands.spawn(Camera2d);

    // スプライトをスポーン
    commands.spawn(Sprite::from_image(
        asset_server.load("branding/bevy_bird_dark.png") // assets ディレクトリに入れることでAssetServerが読み込んでくれる
    ));
}