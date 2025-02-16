# ブロック崩し

[出典先](https://github.com/bevyengine/bevy/blob/latest/examples/games/breakout.rs)

## main.rs

## stepping.rs

### Bevy のスケジュール順序

```rust
use bevy::app::MainScheduleOrder;
```

- Bevy ではシステムの実行順序を **スケジュール (`Schedule`)** で管理します。
- `MainScheduleOrder` は、アプリケーションのメインスケジュール (`Startup`, `Update`, `PostUpdate`, など)の順序を制御するためのリソースです。
- `order.insert_after(Update, DebugSchedule);` のように使うことで、指定したスケジュール (`DebugSchedule`) を `Update` の後に挿入できます。

### スケジュール関連の機能

```rust
use bevy::ecs::schedule::*;
```

- `Schedule` は Bevy の **ECS (Entity-Component-System)** のシステム実行順序を管理するための仕組み です。
- Bevy では `Startup` (**初回のみ実行**), `Update` (**毎フレーム実行**), `PostUpdate` (**更新後に実行**) などのデフォルトスケジュールがあります。
- `ScheduleLabel` はカスタムスケジュールを作るためのラベルです。

```rust
/// 独立した [`Schedule`] を定義し、デバッグ用のステッピング処理を行う。
/// スケジュールを独立させることで、他のスケジュールを調査できるようにする。
#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
struct DebugSchedule;
```

- `DebugSchedule` という独自のスケジュールを作成。
- `derive(ScheduleLabel)` を使うことで Bevy のスケジュールとして利用可能になります。

```rust
app.init_schedule(DebugSchedule);
```

- `DebugSchedule` を Bevy のスケジュールに追加

### `Plugin` の仕組み

```rust
impl Plugin for SteppingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_stepping_hint);

```

- Bevy では `Plugin` を使って機能をモジュール化できます。
- `build()` 内で `app.add_systems()` を使うことで、特定のタイミング (`Startup`, `Update` など) にシステムを追加できます。

### `Resource`

```rust
#[derive(Resource, Debug)]
struct State {
    systems: Vec<(InternedScheduleLabel, NodeId, usize)>,
    ui_top: Val,
    ui_left: Val,
}
```

- `#[derive(Resource)]` をつけると Bevy の **リソース (ECS のグローバルデータ) として管理** できます。
- `systems: Vec<(InternedScheduleLabel, NodeId, usize)>`
  → デバッグ対象のスケジュール・システム情報を格納。
- `ui_top: Val, ui_left: Val`
  → UI の表示位置を管理

```rust
app.insert_resource(stepping);
```

- `Stepping` というリソースを Bevy に登録し、どのシステムからもアクセスできるようにする。

### `System`

```rust
fn handle_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut stepping: ResMut<Stepping>) {
```

- fn で定義された関数が **ECS のシステム** になります。
- `Res<ButtonInput<KeyCode>>`
  → キーボード入力を監視する `Resource` (`Res` は Bevy の `Resource` をシステムに渡すための型)。
- `ResMut<Stepping>`
  → `Stepping` リソースを **可変参照 (ResMut) として取得** し、処理を行う。

### UI コンポーネント

```rust
commands.spawn((
    Text::new(hint_text),
    TextFont {
        font_size: 15.0,
        ..default()
    },
    TextColor(FONT_COLOR),
    Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(5.0),
        left: Val::Px(5.0),
        ..default()
    },
));
```

- `commands.spawn(())` は新しいエンティティを作成する。
- `Text::new(hint_text)` で UI のテキスト要素を作成。
- `Node { position_type: PositionType::Absolute, bottom: Val::Px(5.0), left: Val::Px(5.0) }`
  → UI の配置 (`Absolute` で画面の特定位置に固定)。
- `TextColor(FONT_COLOR)` でフォントの色を設定。
