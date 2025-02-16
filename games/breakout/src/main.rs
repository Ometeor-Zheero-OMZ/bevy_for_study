use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
};

mod stepping;

// 定数はすべて `Transform` ユニットで定義されています。
// デフォルトの2Dカメラで1:1で画面ピクセルに対応します。

// パドルのサイズ（横幅、縦幅）
const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
// パドルと床の間のギャップ
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
// パドルの移動速度
const PADDLE_SPEED: f32 = 500.0;
// パドルが壁にどれだけ近づけるか
const PADDLE_PADDING: f32 = 10.0;

// ボールの開始位置（z値は上に重ねて描画するために設定）
const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
// ボールの直径
const BALL_DIAMETER: f32 = 30.;
// ボールの初期速度
const BALL_SPEED: f32 = 400.0;
// ボールの初期方向（x, y方向の速度）
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

// 壁の厚さ
const WALL_THICKNESS: f32 = 10.0;
// 左の壁のx座標
const LEFT_WALL: f32 = -450.;
// 右の壁のx座標
const RIGHT_WALL: f32 = 450.;
// 下の壁のy座標
const BOTTOM_WALL: f32 = -300.;
// 上の壁のy座標
const TOP_WALL: f32 = 300.;

// ブロックのサイズ（幅、高さ）
const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);
// パドルとブロックの間のギャップ
const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
// ブロック間のギャップ
const GAP_BETWEEN_BRICKS: f32 = 5.0;
// 天井とブロックの間の最低限のギャップ
const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
// ブロックと画面の両端のギャップ
const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

// スコアボードのフォントサイズ
const SCOREBOARD_FONT_SIZE: f32 = 33.0;
// スコアボードテキストの周囲のパディング
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

// 背景色
const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
// パドルの色
const PADDLE_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);
// ボールの色
const BALL_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
// ブロックの色
const BRICK_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
// 壁の色
const WALL_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
// テキストの色
const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
// スコアの色
const SCORE_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .at(Val::Percent(35.0), Val::Percent(50.0)),
        )
        // ゲームのスコアリソースを初期化 (初期スコアは0)
        .insert_resource(Score(0))
        // 背景色を設定
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        // 衝突イベントを追加 (ゲーム中で発生するイベント)
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        // 固定更新（64Hzで更新される）スケジュールにゲームシミュレーションシステムを追加
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,    // 速度の適用
                move_paddle,       // パドルの移動
                check_for_collisions, // 衝突チェック
                play_collision_sound, // 衝突時の音再生
            )
            // システムのチェーン実行（順番に処理）
                .chain()
        )
        // 更新スケジュールでスコアボードを更新するシステムを追加
        .add_systems(Update, update_scoreboard)
        .run();
}

// パドルを示すコンポーネント
#[derive(Component)]
struct Paddle;

// ボールを示すコンポーネント
#[derive(Component)]
struct Ball;

// 速度を示すコンポーネント（Vec2型でX軸とY軸の速度）
#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

// 衝突判定用コンポーネント（ゲーム内で衝突判定を持つオブジェクト）
#[derive(Component)]
struct Collider;

// 衝突イベント（ゲーム内で発生した衝突を追跡）
#[derive(Event, Default)]
struct CollisionEvent;

// ブロックを示すコンポーネント
#[derive(Component)]
struct Brick;

// 衝突音のリソース（音源のハンドル）
#[derive(Resource, Deref)]
struct CollisionSound(Handle<AudioSource>);

// ゲーム内の「壁」を構成するコンポーネントのバンドル
// 複数のコンポーネントを一つにまとめることで、壁のオブジェクトを効率よく作成
#[derive(Bundle)]
struct WallBundle {
    // 壁のスプライト（見た目）
    sprite: Sprite,
    // 壁の位置と回転を定義する変換（Transform）
    transform: Transform,
    // 壁の衝突判定を持つコンポーネント
    collider: Collider,
}

/// アリーナのどの側に壁が位置しているかを表す列挙型
enum WallLocation {
    Left,   // 左側
    Right,  // 右側
    Bottom, // 下側
    Top,    // 上側
}

impl WallLocation {
    /// 壁の*中心*の位置を返す。`transform.translation()`で使用される
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),   // 左壁の中心位置
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),  // 右壁の中心位置
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL), // 下壁の中心位置
            WallLocation::Top => Vec2::new(0., TOP_WALL),      // 上壁の中心位置
        }
    }

    /// 壁の(x, y)サイズを返す。`transform.scale()`で使用される
    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL; // アリーナの高さ
        let arena_width = RIGHT_WALL - LEFT_WALL;  // アリーナの幅
        // 定数が正しいか確認するためのアサーション
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                // 左右の壁のサイズ：幅はWALL_THICKNESS、高さはアリーナの高さ＋壁の厚さ
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                // 上下の壁のサイズ：幅はアリーナの幅＋壁の厚さ、高さはWALL_THICKNESS
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}


impl WallBundle {
    // この「ビルダーメソッド」は壁エンティティ間でロジックを再利用できるようにし、
    // ロジックを変更したときにコードの可読性を向上させ、バグを減らします
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite: Sprite::from_color(WALL_COLOR, Vec2::ONE), // 壁の色を設定したスプライトを作成
            transform: Transform {
                // Vec2からVec3に変換し、z座標を0.0に設定してスプライトの順序を決定
                // これによりスプライトが描画される順序が決まります
                translation: location.position().extend(0.0),
                // 2Dオブジェクトのzスケールは常に1.0に設定しないと
                // 順序が予期しない方法で影響を受ける
                // 詳細は https://github.com/bevyengine/bevy/issues/4149 を参照
                scale: location.size().extend(1.0),
                ..default() // その他のデフォルト値を使用
            },
            collider: Collider, // 衝突判定用のコンポーネントを追加
        }
    }
}

/// ゲームのスコアを追跡するリソース
#[derive(Resource, Deref, DerefMut)]
struct Score(usize);

#[derive(Component)]
struct ScoreboardUi; // スコアボード用のUIコンポーネント

// ゲームのエンティティをワールドに追加するセットアップ関数
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // カメラの追加
    commands.spawn(Camera2d);

    // サウンドの追加
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // パドルの追加
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

    commands.spawn((
        Sprite::from_color(PADDLE_COLOR, Vec2::ONE),
        Transform {
            translation: Vec3::new(0.0, paddle_y, 0.0), // パドルの初期位置
            scale: PADDLE_SIZE.extend(1.0), // パドルのサイズ
            ..default()
        },
        Paddle,  // パドルコンポーネント
        Collider, // 衝突判定用コンポーネント
    ));

    // ボールの追加
    commands.spawn((
        Mesh2d(meshes.add(Circle::default())), // ボールの形状
        MeshMaterial2d(materials.add(BALL_COLOR)), // ボールの色
        Transform::from_translation(BALL_STARTING_POSITION)
            .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.)), // ボールの位置とサイズ
        Ball, // ボールコンポーネント
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED), // ボールの速度
    ));

    // スコアボードの追加
    commands
        .spawn((
            Text::new("Score: "), // スコアのラベル
            TextFont {
                font_size: SCOREBOARD_FONT_SIZE, // フォントサイズ
                ..default()
            },
            TextColor(TEXT_COLOR), // フォントカラー
            ScoreboardUi, // スコアボードUIコンポーネント
            Node {
                position_type: PositionType::Absolute, // 絶対位置指定
                top: SCOREBOARD_TEXT_PADDING, // 上の余白
                left: SCOREBOARD_TEXT_PADDING, // 左の余白
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(), // 子要素としてスコア数値を表示
            TextFont {
                font_size: SCOREBOARD_FONT_SIZE,
                ..default()
            },
            TextColor(SCORE_COLOR), // スコアの色
        ));

    // 壁の追加
    commands.spawn(WallBundle::new(WallLocation::Left));   // 左の壁
    commands.spawn(WallBundle::new(WallLocation::Right));  // 右の壁
    commands.spawn(WallBundle::new(WallLocation::Bottom)); // 下の壁
    commands.spawn(WallBundle::new(WallLocation::Top));    // 上の壁

    // ブロックの追加
    let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES; // ブロックの幅
    let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS; // ブロックの下端位置
    let total_height_of_bricks = TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING; // ブロックの高さ

    assert!(total_width_of_bricks > 0.0); // 幅が0以下でないことを確認
    assert!(total_height_of_bricks > 0.0); // 高さが0以下でないことを確認

    // 利用可能なスペースに基づいて、ブロックを配置できる行数と列数を計算
    let n_columns = (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize; // 列数
    let n_rows = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize; // 行数
    let n_vertical_gaps = n_columns - 1; // 縦の隙間の数

    // 列数を丸めたため、ブロックの上下や左右に配置されるスペースは下限値を表す
    let center_of_bricks = (LEFT_WALL + RIGHT_WALL) / 2.0; // ブロックの中心位置
    let left_edge_of_bricks = center_of_bricks
        // ブロックの幅
        - (n_columns as f32 / 2.0 * BRICK_SIZE.x)
        // ギャップの幅
        - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

    // Bevyではエンティティの`translation`は左下の位置ではなく中心位置を表す
    let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.; // ブロックのx軸方向のオフセット
    let offset_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.; // ブロックのy軸方向のオフセット

    // ブロックを行列に配置する
    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new(
                offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS), // x位置
                offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),    // y位置
            );

            // 各ブロックのエンティティを生成
            commands.spawn((
                Sprite {
                    color: BRICK_COLOR, // ブロックの色
                    ..default()
                },
                Transform {
                    translation: brick_position.extend(0.0), // 位置
                    scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0), // サイズ
                    ..default()
                },
                Brick, // ブロックコンポーネント
                Collider, // 衝突判定用コンポーネント
            ));
        }
    }
}

/// パドルの移動を処理する関数
fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>, // キー入力をリソースとして取得
    mut paddle_transform: Single<&mut Transform, With<Paddle>>, // パドルの変換情報
    time: Res<Time>, // 時間の経過をリソースとして取得
) {
    let mut direction = 0.0; // パドルの移動方向を初期化

    // 左矢印キーが押されていれば、左方向に移動
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }

    // 右矢印キーが押されていれば、右方向に移動
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    // プレイヤー入力に基づき新しいパドルの位置を計算
    let new_paddle_position =
        paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_secs();

    // パドルがアリーナから外れないように位置を制限
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    // 新しいパドル位置を制限内に収める
    paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
}

/// ボールの速度を適用し、位置を更新する関数
fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        // 速度に基づき、ボールの位置を更新
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

/// スコアボードを更新する関数
fn update_scoreboard(
    score: Res<Score>, // 現在のスコアを取得
    score_root: Single<Entity, (With<ScoreboardUi>, With<Text>)>, // スコアボードのUIエンティティ
    mut writer: TextUiWriter, // テキストを書き込むためのライター
) {
    // スコアボードにスコアを表示
    *writer.text(*score_root, 1) = score.to_string();
}

/// 衝突を検出し、必要な処理を行う関数
fn check_for_collisions(
    mut commands: Commands, // コマンドを送信してエンティティを操作
    mut score: ResMut<Score>, // スコアの変更
    ball_query: Single<(&mut Velocity, &Transform), With<Ball>>, // ボールのクエリ
    collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>, // 衝突する可能性のあるエンティティ
    mut collision_events: EventWriter<CollisionEvent>, // 衝突イベントを発行
) {
    let (mut ball_velocity, ball_transform) = ball_query.into_inner();

    // 衝突可能なすべてのエンティティと衝突をチェック
    for (collider_entity, collider_transform, maybe_brick) in &collider_query {
        // ボールとコライダーの衝突判定
        let collision = ball_collision(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
            Aabb2d::new(
                collider_transform.translation.truncate(),
                collider_transform.scale.truncate() / 2.,
            ),
        );

        // 衝突があった場合
        if let Some(collision) = collision {
            // 衝突イベントを発行
            collision_events.send_default();

            // ブロックに衝突した場合、ブロックを消去してスコアを更新
            if maybe_brick.is_some() {
                commands.entity(collider_entity).despawn(); // ブロックを消去
                **score += 1; // スコアを増加
            }

            // ボールの速度を反転させる（衝突の反射）
            let mut reflect_x = false;
            let mut reflect_y = false;

            // 反射処理（衝突した方向によってボールの速度を反転）
            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
            }

            // x軸での反射
            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            // y軸での反射
            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

/// 衝突音を再生する関数
fn play_collision_sound(
    mut commands: Commands, // コマンドを送信してエンティティを操作
    mut collision_events: EventReader<CollisionEvent>, // 衝突イベントを読み取る
    sound: Res<CollisionSound>, // 衝突音リソース
) {
    // 衝突イベントが発生している場合に音を再生
    if !collision_events.is_empty() {
        collision_events.clear(); // イベントをクリアして次フレームに引き継がないようにする
        commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN)); // 音声再生のためにエンティティを生成
    }
}

/// 衝突の種類を表す列挙型
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,   // 左側の衝突
    Right,  // 右側の衝突
    Top,    // 上側の衝突
    Bottom, // 下側の衝突
}

// ボールとコライダーの衝突を判定し、衝突した側を返す
fn ball_collision(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&bounding_box) {
        return None; // 衝突していない場合はNoneを返す
    }

    // 衝突した最寄の点を計算
    let closest = bounding_box.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        // x軸方向の衝突判定
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        // y軸方向の衝突判定（上）
        Collision::Top
    } else {
        // y軸方向の衝突判定（下）
        Collision::Bottom
    };

    Some(side) // 衝突した側を返す
}
