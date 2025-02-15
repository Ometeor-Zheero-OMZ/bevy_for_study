use  bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::sprite::{Wireframe2dConfig, Wireframe2dPlugin};

fn main() {
    let mut app = App::new();                               // Bevyアプリケーションを作成
    app.add_plugins((
        DefaultPlugins,                                          // デフォルトのプラグイン（レンダリングやイベント処理を含む）を追加
        #[cfg(not(target_arch = "wasm32"))]                      // WebAssembly環境ではWireframe2dPluginを無効化
        Wireframe2dPlugin
    ))
    .add_systems(Startup, setup);              // 起動時に setup システムを実行

    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, toggle_wireframe); // Updateフェーズでワイヤーフレームの切り替えを追加
    app.run();                                                   // アプリケーションを実行
}

const X_EXTENT: f32 = 900.;                                      // 形状を横に並べる際のX軸の幅

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d); // 2Dカメラを追加

    let shapes = [
        meshes.add(Circle::new(50.0)),                                         // 円
        meshes.add(CircularSector::new(50.0, 1.0)),                     // 扇形
        meshes.add(CircularSegment::new(50.0, 1.25)),                   // 円弧
        meshes.add(Ellipse::new(25.0, 50.0)),                 // 楕円
        meshes.add(Annulus::new(25.0, 50.0)),              // ドーナツ形状
        meshes.add(Capsule2d::new(25.0, 50.0)),                        // カプセル形状
        meshes.add(Rhombus::new(75.0, 100.0)), // ひし形
        meshes.add(Rectangle::new(50.0, 100.0)),                        // 長方形
        meshes.add(RegularPolygon::new(50.0, 6)),                 // 六角形
        meshes.add(Triangle2d::new(                                                   // 三角形
            Vec2::Y * 50.0,               // 上の頂点
            Vec2::new(-50.0, -50.0), // 左下の頂点
            Vec2::new(50.0, -50.0),  // 右下の頂点
        )),
    ];
    let num_shapes = shapes.len(); // 形状の数を取得

    for (i, shape) in shapes.into_iter().enumerate() {
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

        commands.spawn((
            Mesh2d(shape), // メッシュを2Dオブジェクトとしてスポーン
            MeshMaterial2d(materials.add(color)), // 色を指定
            Transform::from_xyz(
                // X座標を計算して形状を等間隔に配置
                -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                0.0,
                0.0,
            ),
        ));
    }

    #[cfg(not(target_arch = "wasm32"))]
    commands.spawn((
        Text::new("Press space to toggle wireframes"),  // テキストを作成
        Node {                                                // テキストを配置する座標を設定
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

#[cfg(not(target_arch = "wasm32"))]
fn  toggle_wireframe(
    mut wireframe_config: ResMut<Wireframe2dConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global; // スペースキーをワイヤーフレーム表示のON/OFF切り替え
    }
}