# Audio Control

[出典先](https://bevyengine.org/examples/audio/audio-control/)

## サンプル

サンプル画像なし

仕様：

- 音楽再生
- スピード調整
- 音量調整

## Bevy 特有の機能

- `Query<&AudioSink, With<MyMusic>>`
  - `Query<T, F>` を使うと、**条件に合うエンティティを取得** できる。
  - `&AudioSink` → **オーディオ再生のコントロール** を取得。
  - `With<MyMusic>` → `MyMusic` **コンポーネントを持つエンティティを検索**。
- `get_single()`
  - `Query` の結果から **1 つだけエンティティを取得**。
  - `Ok(sink)` `で成功した場合、sink` を操作できる。
- `sink.set_speed(...)`

  - **再生速度を変化** させる (`set_speed(float)` メソッド)。
  - `ops::sin(time.elapsed_secs() / 5.0) + 1.0` で、5 秒周期で音楽のスピードを変える。

- `sink.toggle()`
  - **音楽を一時停止/再開** する
