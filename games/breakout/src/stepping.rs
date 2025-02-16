use bevy::{app::MainScheduleOrder, ecs::schedule::*, prelude::*};

/// 独立した [`Schedule`] を定義し、デバッグ用のステッピング処理を行う。
/// スケジュールを独立させることで、他のスケジュールを調査できるようにする。
#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
struct DebugSchedule;

/// ステッピング UI を追加するためのプラグイン
#[derive(Default)]
pub struct SteppingPlugin {
    schedule_labels: Vec<InternedScheduleLabel>,
    top: Val,
    left: Val,
}

impl SteppingPlugin {
    /// ステッピング対象のスケジュールを追加する
    pub fn add_schedule(mut self, label: impl ScheduleLabel) -> SteppingPlugin {
        self.schedule_labels.push(label.intern());
        self
    }

    /// ステッピング UI の位置を設定する
    pub fn at(self, left: Val, top: Val) -> SteppingPlugin {
        SteppingPlugin { top, left, ..self }
    }
}

impl Plugin for SteppingPlugin {
    fn build(&self, app: &mut App) {
        // アプリの起動時に UI を構築する
        app.add_systems(Startup, build_stepping_hint);
        if cfg!(not(feature = "bevy_debug_stepping")) {
            return;
        }

        // デバッグ用の独立したスケジュールを作成し、メインスケジュールの実行順序に追加
        app.init_schedule(DebugSchedule);
        let mut order = app.world_mut().resource_mut::<MainScheduleOrder>();
        order.insert_after(Update, DebugSchedule);

        // ステッピングリソースを作成し、追加されたスケジュールを登録
        let mut stepping = Stepping::new();
        for label in &self.schedule_labels {
            stepping.add_schedule(*label);
        }
        app.insert_resource(stepping);

        // UI の状態管理用リソースを挿入
        app.insert_resource(State {
            ui_top: self.top,
            ui_left: self.left,
            systems: Vec::new(),
        })
        .add_systems(
            DebugSchedule,
            (
                build_ui.run_if(not(initialized)),
                handle_input,
                update_ui.run_if(initialized),
            )
                .chain(),
        );
    }
}

/// ステッピング UI の状態を管理するリソース
#[derive(Resource, Debug)]
struct State {
    systems: Vec<(InternedScheduleLabel, NodeId, usize)>, // システムの情報

    ui_top: Val,
    ui_left: Val,
}

/// UI が初期化されているかどうかを判定する条件関数
fn initialized(state: Res<State>) -> bool {
    !state.systems.is_empty()
}

const FONT_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
const FONT_BOLD: &str = "fonts/FiraSans-Bold.ttf";

#[derive(Component)]
struct SteppingUi;

/// ステッピング UI を構築するシステム
fn build_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    schedules: Res<Schedules>,
    mut stepping: ResMut<Stepping>,
    mut state: ResMut<State>,
) {
    let mut text_spans = Vec::new();
    let mut always_run = Vec::new();

    let Ok(schedule_order) = stepping.schedules() else {
        return;
    };

    // スケジュール内のシステムをリスト化
    for label in schedule_order {
        let schedule = schedules.get(*label).unwrap();
        text_spans.push((
            TextSpan(format!("{label:?}\n")),
            TextFont {
                font: asset_server.load(FONT_BOLD),
                ..default()
            },
            TextColor(FONT_COLOR),
        ));

        let Ok(systems) = schedule.systems() else {
            return;
        };

        for (node_id, system) in systems {
            if system.name().starts_with("bevy") {
                always_run.push((*label, node_id));
                continue;
            }

            state.systems.push((*label, node_id, text_spans.len() + 1));

            text_spans.push((
                TextSpan::new("   "),
                TextFont::default(),
                TextColor(FONT_COLOR),
            ));

            text_spans.push((
                TextSpan(format!("{}\n", system.name())),
                TextFont::default(),
                TextColor(FONT_COLOR),
            ));
        }
    }

    for (label, node) in always_run.drain(..) {
        stepping.always_run_node(label, node);
    }

    commands
        .spawn((
            Text::default(),
            SteppingUi,
            Node {
                position_type: PositionType::Absolute,
                top: state.ui_top,
                left: state.ui_left,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.33)),
            Visibility::Hidden,
        ))
        .with_children(|p| {
            for span in text_spans {
                p.spawn(span);
            }
        });
}

/// ステッピングのヒントをコンソールに表示する
fn build_stepping_hint(mut commands: Commands) {
    let hint_text = if cfg!(feature = "bevy_debug_stepping") {
        "Press ` to toggle stepping mode (S: step system, Space: step frame)"
    } else {
        "Bevy was compiled without stepping support. Run with `--features=bevy_debug_stepping` to enable stepping."
    };
    info!("{}", hint_text);
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
}

/// ユーザー入力を処理し、ステッピングを制御する
fn handle_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut stepping: ResMut<Stepping>) {
    if keyboard_input.just_pressed(KeyCode::Slash) {
        info!("{:#?}", stepping);
    }

    if keyboard_input.just_pressed(KeyCode::Backquote) {
        if stepping.is_enabled() {
            stepping.disable();
            debug!("disabled stepping");
        } else {
            stepping.enable();
            debug!("enabled stepping");
        }
    }

    if !stepping.is_enabled() {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        debug!("continue");
        stepping.continue_frame();
    } else if keyboard_input.just_pressed(KeyCode::KeyS) {
        debug!("stepping frame");
        stepping.step_frame();
    }
}

fn update_ui(
    mut commands: Commands, // エンティティの操作 (UI の可視性を変更するため)
    state: Res<State>, // 現在の UI の状態 (システムリストや UI の位置情報など)
    stepping: Res<Stepping>, // ステッピングの状態 (有効かどうか、現在のカーソル位置など)
    ui: Single<(Entity, &Visibility), With<SteppingUi>>, // ステッピング UI のエンティティと可視状態
    mut writer: TextUiWriter, // UI のテキストを更新するためのライター
) {
    // ステッピング UI を有効・無効の状態にする
    let (ui, vis) = *ui;
    match (vis, stepping.is_enabled()) {
        // ステッピングが有効になったら UI を表示
        (Visibility::Hidden, true) => {
            commands.entity(ui).insert(Visibility::Inherited);
        }
        // すでに可視の場合や変更が不要な場合は何もしない
        (Visibility::Hidden, false) | (_, true) => (),
        // ステッピングが無効になったら UI を非表示
        (_, false) => {
            commands.entity(ui).insert(Visibility::Hidden);
        }
    }

    // ステッピングが無効ならこれ以上処理しない
    if !stepping.is_enabled() {
        return;
    }

    // ステッピングのカーソル位置を取得
    let (cursor_schedule, cursor_system) = match stepping.cursor() {
        // カーソルがない場合 (ステッピングが有効でも選択されたシステムがない場合) は処理を終了
        None => return,
        Some(c) => c, // カーソルがある場合は取得
    };

    // 各システムの UI を更新
    for (schedule, system, text_index) in &state.systems {
        // 現在のカーソル位置にあるシステムには "->" を表示し、それ以外はスペースを表示
        let mark = if &cursor_schedule == schedule && *system == cursor_system {
            "-> "
        } else {
            "   "
        };
        // UI の対応するテキストを更新
        *writer.text(ui, *text_index) = mark.to_string();
    }
}