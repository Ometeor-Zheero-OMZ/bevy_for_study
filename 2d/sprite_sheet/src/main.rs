use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // スプライトのぼやけを防ぐ
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sprite)
        .run();
}


/// スプライトのアニメーション範囲を定義するコンポーネント
#[derive(Component)]
struct AnimationIndices {
    first: usize, // アニメーションの最初のフレーム
    last: usize, // アニメーションの最後のフレーム
}

/// アニメーションのタイマーを管理するコンポーネント
#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

/// スプライトのアニメーションを制御するシステム
fn animate_sprite(
    time: Res<Time>, // 時間のリソース (delta time などを取得)
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta()); // タイマーを進める

        if timer.just_finished() {
            // アニメーションのフレームを更新する
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = if atlas.index == indices.last {
                    indices.first // 最後のフレームなら最初に戻る
                } else {
                    atlas.index + 1 // 次のフレームへ進む
                }
            }
        }
    }
}

/// 初期セットアップ (カメラとスプライトを追加)
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("textures/rpg/chars/gabe/gabe-idle-run.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None); // スプライトシートの設定
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    
    // アニメーションの対象となるフレーム範囲
    let animation_indices = AnimationIndices { first: 1, last: 6 };

    // 2D カメラを取得
    commands.spawn(Camera2d);

    // スプライトのエンティティをスポーン
    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first, // 最初のフレーム
            },
        ),
        Transform::from_scale(Vec3::splat(6.0)), // スプライトのサイズを 6 倍に拡大
        animation_indices, // アニメーションの範囲情報
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)), // 0.1 秒ごとにフレームを更新
    ));
}