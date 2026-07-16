# 開発方針＆開発環境ルール(aruaru-tokyo-server)

作業ドライブは`F:\open-runo`。この節は[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)の`CLAUDE.md`を正本とし、各プロジェクトへコピーして同期する方針に準じる。

## 開発方針・開発環境ルール(全リポジトリ共通ヘッダー、2026-07-15追記)

### 1. 比較的新しい言語・フレームワークの参照資料一覧

Rust自体は歴史があるが、本エコシステムが採用する[Poem](https://github.com/poem-web/poem)のような比較的新しい・情報量がまだ少なめのWebフレームワークは、Python+FastAPIのような広く普及した組み合わせと比べ、AIモデルの学習データ・公開されている実装例/Q&A/ブログ記事の絶対量が少ない傾向がある。そのため、AI駆動開発(Claude等)がこれらを扱う際、実装の勘違い・API名の記憶違い・古いバージョンのAPIでの実装(本プロジェクトで実際に複数回発生した既知の失敗パターン)による手戻り・いたちごっこが起きやすい。

対策として、AIが作業を始める際は、以下からそのタスクに必要な部分だけを先に参照してから実装に着手すること(全部読む必要はない。関連しそうな1〜2件を拾い読みする程度で十分)。これにより歩留まりが上がり、AI駆動開発の手戻りが減ることが期待される。

| 技術 | 公式ドキュメント | GitHub | 補足・ブログ等 |
|---|---|---|---|
| Rust言語本体 | https://doc.rust-lang.org/book/ | https://github.com/rust-lang/rust | https://blog.rust-lang.org/ |
| Poem(Webフレームワーク) | https://docs.rs/poem/latest/poem/ | https://github.com/poem-web/poem | https://crates.io/crates/poem |
| Tokio(非同期ランタイム) | https://tokio.rs/tokio/tutorial | https://github.com/tokio-rs/tokio | https://tokio.rs/blog |
| async-graphql | https://async-graphql.github.io/async-graphql/en/index.html | https://github.com/async-graphql/async-graphql | https://crates.io/crates/async-graphql |
| Tauri | https://tauri.app/ | https://github.com/tauri-apps/tauri | https://tauri.app/blog/ |
| wasm-bindgen / web-sys | https://rustwasm.github.io/wasm-bindgen/ | https://github.com/rustwasm/wasm-bindgen | https://rustwasm.github.io/docs/book/ |
| SurrealDB | https://surrealdb.com/docs | https://github.com/surrealdb/surrealdb | https://surrealdb.com/blog |
| sqlx | https://docs.rs/sqlx/latest/sqlx/ | https://github.com/launchbadge/sqlx | |
| WinFsp | https://winfsp.dev/ | https://github.com/winfsp/winfsp | |
| DirectX 12 / DirectML | https://learn.microsoft.com/en-us/windows/win32/direct3d12/directx-12-programming-guide | https://github.com/microsoft/DirectML | https://devblogs.microsoft.com/directx/ |
| WebAssembly(wasm32全般) | https://webassembly.org/ | https://github.com/WebAssembly | https://rustwasm.github.io/docs/book/ |

⚠️ **重要な注意(正直な開示)**: このURL一覧は、Web検索ツールを持たないセッションで学習データに基づき記載したものであり、実在性・現在の有効性・記載内容の正確性を検証していない。特にAI(Claude含む)がこのリストを鵜呑みにして実装や回答の根拠にすることは避け、開発者自身が実際にアクセスして確認するか、Web検索が使えるセッションで一次情報を再確認してから利用すること。リンク切れ・リダイレクト・バージョン変更(特にAPIの破壊的変更)の可能性を常に考慮する。新しい技術を追加する場合はこの表に追記していくこと。

### 2. AI駆動開発ツールに関する所感(2026-07-15、ユーザー所感として記録)

2026-07-15時点、ChatGPT等の汎用AIチャットは小規模なWebアプリ程度までは開発できるものの、システムがある程度複雑・大規模になると出戻りが大きくなり、一度に扱えるプログラムサイズにもすぐ限界が来る傾向がある。

Claude Code / Claude Desktopは、ローカルドライブを直接指定してファイルの読み書きができ、GitHubリポジトリの読み出し(本プロジェクトのような複数リポジトリにまたがるエコシステム)にも対応できるため、本プロジェクトのような規模のAI駆動開発には適していると考えられる。新しくAI駆動開発環境をセットアップする際の選択肢として推奨する。

## このリポジトリの役割

`aruaru.tokyo`のTOPページ。2026-07-15、それまでPHPで実装していたものをRust+[Poem](https://github.com/poem-web/poem)へ書き換えた(ユーザー指示: 「aruaru.tokyoはRust+Poemベースでお願いします」)。`audiocafe.tokyo`は引き続きPHPのまま——ドメインごとにスタックが異なる意図的な設計。

## 技術スタック

- Rust + Poem(hyperベースの軽量Webフレームワーク)。DBに依存しない、1バイナリ完結。
- 重量級フレームワーク・ORMは使わない(poem-cosmo-tauriエコシステムの規約に準拠)。
- フロントエンドはサーバーサイドで文字列組み立てしたプレーンHTML(テンプレートエンジン不使用)。JSはページ内`<script>`のみ(shuffleボタンの挙動)。

## 主要モジュール(`src/main.rs`単一ファイル)

- `categories()` — 「あるある」コンテンツの静的データ
- `render_related_sites()` / `RELATED_SITES` — audiocafe.tokyo側関連ページへのリンク
- `fetch_repo_file()` / `markdown_to_rs()` — GitHub raw contentを取得し`.rs`風に変換するreadme-to-rs機能
- `markdown_to_github_style_html()` — `pulldown-cmark`を使い、README等をGitHub風に実際にレンダリングしたHTMLへ変換(`.rs`変換表示と切替タブで両方見られる)
- `fetch_org_repos()` / `api_repos` ハンドラ(`GET /api/repos`) — `aon-co-jp`の全リポジトリ名をGitHub APIから毎回最新取得する。**注意: `aon-co-jp`はOrganizationではなく個人アカウントのため`/users/{name}/repos`エンドポイントを使う**(`/orgs/`だと404になることを実機確認済み)。未認証呼び出しのためGitHub APIのレート制限(60回/時/IP)を受ける。
- `is_valid_repo_name()` — リポジトリ名の形式検証。動的取得したリポジトリは`GITHUB_REPOS`の静的リストに含まれないため、固定リストとの照合ではなくこの形式検証で受け付ける。
- `top()` ハンドラ — TOPページ全体のHTML組み立て(`Query<TopQuery>`で`?repo=`パラメータを受ける)

## デプロイ

VPS(ConoHa、AlmaLinux)上で直接`cargo build --release`し、生成バイナリをsystemdサービス(`aruaru-tokyo-server.service`)として`127.0.0.1:4100`にバインド。nginx(`/etc/nginx/conf.d/aruaru.tokyo.conf`)が443番でTLS終端し、`location /`でこのポートへリバースプロキシする。

**`/aruaru/`・`/aruaru-lady/`・`/rakuten-mobile/`のミラーlocation**: これらのパスは元々`audiocafe.tokyo`側にのみ存在するコンテンツ(PHP)で、`aruaru.tokyo`からも直接閲覧できるようにするため、nginx側で`http://127.0.0.1:80/`(Hostヘッダーを`audiocafe.tokyo`に上書き)へ内部プロキシするlocationブロックを個別に追加している。このバイナリ自体はこれらのパスを一切処理しない(`Route`には`/`と`/healthz`しか登録されていない)。

## 関連プロジェクト

- [open-runo](https://github.com/aon-co-jp/open-runo) — Rust→WASM/tokio+hyperのopen-runoエコシステム本体
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — Poem/Tauri実装規約の出典元
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — 汎用WEBサーバーゲートウェイ
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — DB層(このリポジトリはDB非依存)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — 開発ルールの正本
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — OTP認証・サイト管理サーバー
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo)(PHP) — ミラーlocationの実体

## 運用ルール

- このリポジトリにDB依存機能・重量級フレームワークを持ち込まないこと。
- 外部リンク定数は必ず公開URLを使うこと(ループバックアドレスは閲覧者から到達不能)。
- VPSの実IPアドレスをコード・ドキュメントに記録しないこと。
- CLAUDE.mdを更新した場合、この10ヶ国語版(`CLAUDE-<言語>.md`)も同じ内容で更新し、一緒にpushすること。

## 現状

- 2026-07-15 ブートストラップ・本番投入完了。
- 2026-07-16 GitHub連携機能拡張(組織リンク・動的リポジトリ一覧取得・GitHub風README表示・全幅表示)完了。

## HANDOFF(直近の作業ログ、上が最新)

- **2026-07-16**: GitHub連携機能を拡張。GitHub organizationトップページへのリンク、最新リポジトリ一覧の動的取得(`GET /api/repos`)、選択リポジトリへの直接リンク、`pulldown-cmark`によるGitHub風README表示(既存の`.rs`変換と切替可能)、表示エリアの全幅化を実装。実装中に発見: `aon-co-jp`はOrganizationではなく個人アカウントのため`/users/{name}/repos`が正しいエンドポイント。リポジトリ検証も静的リスト照合から形式検証へ変更。
- **2026-07-15**: PHPからRust+Poemへの書き換え・VPS本番投入完了。

## アプリケーションサーバー層の役割(open-runo / poem-cosmo-tauri、2026-07-16追記)

「配信エンジン(vhost)」に`open-web-server`を選択肢として追加したが、open-web-serverがApache＋Nginxのハイブリッド仕様のWebサーバーとしてまだ機能していない間は、Tomcatのような互換レイヤーとして機能するのは`open-runo`または`poem-cosmo-tauri`である。

これらは`open-raid-z`とVersionlessAPIによって、バージョンレス運用とバージョン管理・Git管理を両立しながら、ACID互換性とZFS互換性に対応した`aruaru-db`と、PostgreSQLとのDUAL DATABASE構成による「4層4重」の最新鋭の通信システムを構築し、仕様変更が容易なデータベース設計により、3DオンラインゲームAI課金アイテム、オンライン金融、オンライン証券、オンラインクレジットカード決済など、ネット上で紛失してはならないミッションクリティカルな用途向けに、24時間365日ノンストップのサーバー対応WEBサイト開発を全面的にバックアップするフレームワーク・ミドルウェアとして機能することを目指す。
