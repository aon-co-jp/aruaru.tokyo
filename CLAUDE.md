# 開発方針＆開発環境ルール(aruaru-tokyo-server)

作業ドライブは`F:\open-runo`。この節は[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)の`CLAUDE.md`を正本とし、各プロジェクトへコピーして同期する方針に準じる。

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
- `top()` ハンドラ — TOPページ全体のHTML組み立て(`Query<TopQuery>`で`?repo=`パラメータを受ける)

## デプロイ

VPS(ConoHa、AlmaLinux)上で直接`cargo build --release`し、生成バイナリをsystemdサービス(`aruaru-tokyo-server.service`)として`127.0.0.1:4100`にバインド。nginx(`/etc/nginx/conf.d/aruaru.tokyo.conf`)が443番でTLS終端し、`location /`でこのポートへリバースプロキシする。

**`/aruaru/`・`/aruaru-lady/`・`/rakuten-mobile/`のミラーlocation**: これらのパスは元々`audiocafe.tokyo`側にのみ存在するコンテンツ(PHP)で、`aruaru.tokyo`からも直接閲覧できるようにするため、nginx側で`http://127.0.0.1:80/`(Hostヘッダーを`audiocafe.tokyo`に上書き)へ内部プロキシするlocationブロックを個別に追加している。このバイナリ自体はこれらのパスを一切処理しない(`Route`には`/`と`/healthz`しか登録されていない)。

## 運用ルール

- このリポジトリにDB依存機能・重量級フレームワークを持ち込まないこと。
- `ARUARU_EASYWEB_URL`などの外部リンク定数は、必ず公開URLを使うこと(`127.0.0.1`などのループバックアドレスを埋め込むと、閲覧者のブラウザからは到達不能になるバグを生む——2026-07-15に実際に発生し修正した実例あり)。
- VPSの実IPアドレスをコード・ドキュメントに記録しないこと(既存の運用ルールを継承)。

## 現状

- 2026-07-15 ブートストラップ・本番投入完了。VPS上の実バイナリで動作確認済み(TOPページ・repo=パラメータ経由の`.rs`変換ビューアともに200 OK、実際にGitHub raw contentを取得して変換表示することを確認)。
- `/aruaru/`・`/aruaru-lady/`・`/rakuten-mobile/`のミラーlocationも実際に200が返ることを確認済み。

## HANDOFF(直近の作業ログ、上が最新)

- **2026-07-15**: PHPからRust+Poemへの書き換え・VPS本番投入完了。`ARUARU_EASYWEB_URL`がループバックアドレス(`127.0.0.1:8080`)を指していたバグを発見・修正(公開URL`https://runo.tokyo/`に変更)。README/CLAUDE.md/PORTING.md新規作成。
