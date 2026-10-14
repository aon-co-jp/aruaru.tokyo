# aruaru-tokyo-server

[aruaru.tokyo](https://aruaru.tokyo/) のTOPページ。Rust + [Poem](https://github.com/poem-web/poem)製、DB非依存・1バイナリ完結。

`audiocafe.tokyo`(PHP)とは別ドメイン・別スタックの姉妹サイトで、poem-cosmo-tauriエコシステムの規約(hyper/Poemを直接使い、重量級フレームワークやDBに依存しない)に合わせて実装している。

## 機能

- 「あるある」(誰もが頷く日常のあるある集)コンテンツ、ジャンル別5カテゴリ + ランダム表示
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb)への即リンク
- GitHub(aon-co-jp)アカウントのトップページへのリンク
- 「🔄 最新のリポジトリ一覧を取得」ボタンで、GitHub APIから最新の全リポジトリ名を取得して選択肢を動的に更新
- 選択中のリポジトリのGitHubページへの直接リンク
- README.md・CLAUDE.md・PORTING.mdを、GitHub風にレンダリングしたHTML表示と、rustdocコメント(`//!`)形式の`.rs`風テキスト表示の両方で閲覧可能(タブ切替、readme-to-rs構想の実装)。表示エリアは横幅いっぱい(94vw、最大1400px)

## ビルド・起動

```bash
cargo build --release
ARUARU_TOKYO_BIND=0.0.0.0:4100 ./target/release/aruaru-tokyo-server
```

`ARUARU_TOKYO_BIND`未指定時は`0.0.0.0:4100`で待ち受ける。

## 本番構成(参考)

VPS上ではsystemdサービスとして`127.0.0.1:4100`にバインドし、nginxが443番でTLS終端した上でリバースプロキシする。`/aruaru/`・`/aruaru-lady/`・`/rakuten-mobile/`パスは`audiocafe.tokyo`側の実体(PHP)へ内部プロキシするミラーとして、同じnginx vhost内に個別のlocationブロックを追加している(詳細は[CLAUDE.md](CLAUDE.md)参照)。

## 関連プロジェクト

- [open-runo](https://github.com/aon-co-jp/open-runo) — Rust→WASM/tokio+hyperのopen-runoエコシステム本体
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — Poem/Tauri実装規約の出典元
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — 汎用WEBサーバーゲートウェイ
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — DB層(このリポジトリはDB非依存)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — 開発ルールの正本
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — OTP認証・サイト管理サーバー(tokio+hyper製)
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo)(PHP)

## オンライン学習サイトの紹介と「AI駆動開発の研究」記事の掲載(2026-10-08)
トップページの「Claude / aruaru 紹介」リンク群の直前に、オンライン学習サイト5件(Progate「AI駆動開発」「HTML・CSS」「Python入門」、paizaラーニングPython、python.jp)と、
研究の一部としてのRust移植メリット記事(タイトル・URL・リンク)を追加(`ONLINE_LEARNING_HTML` / `AI_DRIVEN_RESEARCH_HTML`)。5件のリンクは公開前に実在確認済み。
**お引越し**: 本番は`/root/url/aruaru.tokyo`で`cargo build --release`→`systemctl restart aruaru.tokyo.service`。手順全体は`audiocafe-tokyo`の`PORTING.md`「VPS お引越し手順」。

## JSON処理の使用方針: `parse_secure_sonic`(速い型+安全)と`sonic-rs`は使える所では全て使う(2026-10-14、ユーザー指示)
正本は[`RS-JSON`](https://github.com/aon-co-jp/RS-JSON)(`README.md`・`PORTING.md`)。

**方針**: JSONを読む所は、使える所では**全て** `rust-json`(RS-JSON)の `parse_secure_sonic`(`fast`機能、結果は`sonic_rs::Value`)を使う。
`sonic-rs` を直接使える所(型付き構造体への読み込みなど)も同様。使えない所は `parse_secure`(結果は`serde_json::Value`)を使う。
外部から来る信頼できない入力(HTTP本文・ネットワーク越しのファイル・アップロード)は、**必ずサイズ・深さ・重複キーの上限つき**で読む。

**実測(7 MB・4万件)**: `serde_json` 200 ms / `parse_secure` 217 ms / **`parse_secure_sonic` 83 ms(約2.4倍速い、防御込み)** / `sonic-rs`そのまま 33 ms(約6倍、防御なし)。
`serde_json::Value`に読む限りSIMDでもほぼ速くならない(188 ms)。**速さの源は、結果を連続した領域に詰め込む値の型**。

**使える所 / 使えない所**
- ✅ 読み取り中心で、JSONを大きく・頻繁に読む所(キャッシュJSON、設定、APIの応答、ログ、インデックス)。
- ✅ 外部からの入力の入口(速さより防御が目的。`parse_secure`でも可)。
- ⚠️ `serde_json::Value`を関数間で受け渡している既存コードは、型を`sonic_rs::Value`に直す書き換えが要る(`JsonValueTrait`/`JsonContainerTrait`のメソッド)。書き換えずに`serde_json::Value`へ読むと速くならない。
- ❌ WASM(`rust_json::light`、依存ゼロ)、PHP/JavaScript側のJSON処理(Rustの部品は使えない)。
- 注意: 依存先の`sonic-rs`は内部で`unsafe`を使う(自前コードに`unsafe`は無い)。小さなJSON(数KB)では体感差が出ない。速度目的の書き換えは、実測で効く所から。

**導入**: `Cargo.toml`に `rust-json = { path = "../RS-JSON", features = ["fast"] }`。使用例は`RS-JSON/README.md`。

**このリポジトリでの適用状況**: 未適用(方針のみ)。コードの切り替えは、JSONを大きく・頻繁に読む所から実測して順に行う。
