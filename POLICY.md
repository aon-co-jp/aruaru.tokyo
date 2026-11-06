# POLICY(API 方針)

正本: RUNO の API-POLICY.md。REST 全廃・Connect(ブラウザ/curl)・gRPC(サービス間)。新規 REST は作らない。例外は規格が形を決めるもののみ。

## REST 全廃・Connect 移行の状況 (2026-11-06)

方針の正本は `aon-co-jp/RUNO` の `API-POLICY.md`(日英独露)。このリポジトリの状況:

- **移した(2本)**: `GET /api/repos` → `POST /aruaru.tokyo.v1.Site/ListRepos`、`POST /api/tokoro-unlock` → `POST /aruaru.tokyo.v1.Site/TokoroUnlock`。`content-type: application/json` のみ受け付け(415)、本文は4KB上限(超過・不正JSONは400)、エラーは `{"code","message"}`。呼び出し元(`static/app.js`、トップのインラインJS)を同時に直し、旧 `/api/*` は削除(404)。
- **例外(移さない)**: `/healthz`(ヘルスチェック)、`/video/*.mp4`・`/style.css`・`/*.js`・`/help` 等の静的ファイル配信、`/`・`/p`・`/k`・`/open-aruaru-runo*` のHTML画面(ブラウザが直接開くGET、副作用なし)。`/video/tokoro.mp4?t=` はMP4の `<video src>` が直接取得する静的ファイルで、トークン検査つき(Connectにできない)。
- 「REST風17本」は一覧ツールが静的動画・画面ルートを数えた値で、実際の自前APIは上の2本のみ。
- 未対応: 受信JSONの読み込みは `serde_json`(RS-JSON未導入)。サイズ上限のみで深さ・重複キー上限は未実装(入力は `password` 1項目のみ)。
