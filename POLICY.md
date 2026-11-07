# POLICY(API 方針)

正本: RUNO の API-POLICY.md。REST 全廃・Connect(ブラウザ/curl)・gRPC(サービス間)。新規 REST は作らない。例外は規格が形を決めるもののみ。

## REST 全廃・Connect 移行の状況 (2026-11-06)

方針の正本は `aon-co-jp/RUNO` の `API-POLICY.md`(日英独露)。このリポジトリの状況:

- **移した(2本)**: `GET /api/repos` → `POST /tokyo_aruaru.v1.Site/ListRepos`、`POST /api/tokoro-unlock` → `POST /tokyo_aruaru.v1.Site/TokoroUnlock`。`content-type: application/json` のみ受け付け(415)、本文は4KB上限(超過・不正JSONは400)、エラーは `{"code","message"}`。呼び出し元(`static/app.js`、トップのインラインJS)を同時に直し、旧 `/api/*` は削除(404)。
- **例外(移さない)**: `/healthz`(ヘルスチェック)、`/video/*.mp4`・`/style.css`・`/*.js`・`/help` 等の静的ファイル配信、`/`・`/p`・`/k`・`/open-aruaru-runo*` のHTML画面(ブラウザが直接開くGET、副作用なし)。`/video/tokoro.mp4?t=` はMP4の `<video src>` が直接取得する静的ファイルで、トークン検査つき(Connectにできない)。
- 「REST風17本」は一覧ツールが静的動画・画面ルートを数えた値で、実際の自前APIは上の2本のみ。
- 未対応: 受信JSONの読み込みは `serde_json`(RS-JSON未導入)。サイズ上限のみで深さ・重複キー上限は未実装(入力は `password` 1項目のみ)。
- 注意: Connect のパッケージ名は `tokyo_aruaru.v1.Site`。前面(open-web-server)の `path_prefix = "/aruaru"`(audiocafe.tokyo へ転送)に先頭一致しない名前にする必要がある(`aruaru.tokyo.v1.*` は横取りされ404になった)。

## 落ちたら自動で起動し直す(自動復帰)方針と状況 / Auto-restart policy and status (2026-11-07)

### 日本語
- **方針**: 常駐するサービス・アプリは、`kill`(SIGTERM)・強制終了(SIGKILL)・異常終了のどれで止まっても、数秒以内に自動で起動し直す。`systemctl stop` のような**意図した停止**だけは止まったままにする。REST 全廃と同時に、全リポジトリで 調査 → 確認 → TEST → DEBUG → 開発 → TEST → DEBUG を行う。
- **VPS(実施済み、2026-11-07)**: `Restart=on-failure` だったと確認した自前のサービス30個と、`postgresql`・`php-fpm`(`Restart=no`)に、`/etc/systemd/system/<unit>.d/10-autorestart.conf`(`Restart=always`、`RestartSec=3`、`StartLimitIntervalSec=0`)を追加。元のユニットファイルは変更していない。`on-failure` は SIGTERM/`kill` を正常終了とみなして再起動しないため。
- **実機TEST**: `karu-tokyo-server`・`runo-tokyo`・`rs-ops` を SIGTERM、`rs-guard` を SIGKILL で止め、6秒後に全て `active`(新しいPID)で復帰。
- **PC(Windows)の IME**: 常駐エンジン `aruaru-ime-daemon.exe` を強制終了して約3秒で `/health` が復帰することを確認(2026-11-07)。
- **未了**: 他の PC/スマホ向けアプリ(open-backup 等)の常駐プロセスの自動復帰の確認、macOS(`KeepAlive`)・Linux 版の実機確認、各リポジトリへの個別追記(担当が REST 移行する際に実施)。
- **ユニットを追加するとき**: 新しいサービスは最初から `Restart=always`、`RestartSec=3`、`StartLimitIntervalSec=0` にする。

### English
- **Policy**: resident services and apps must restart automatically within a few seconds, whether stopped by `kill` (SIGTERM), a forced kill (SIGKILL) or a crash. Only an **intentional stop** such as `systemctl stop` stays stopped. Alongside the REST abolition, every repository gets survey → verify → TEST → DEBUG → develop → TEST → DEBUG.
- **VPS (done, 2026-11-07)**: for the 30 own services found with `Restart=on-failure`, plus `postgresql` and `php-fpm` (`Restart=no`), a drop-in `/etc/systemd/system/<unit>.d/10-autorestart.conf` (`Restart=always`, `RestartSec=3`, `StartLimitIntervalSec=0`) was added. The original unit files are unchanged. Reason: `on-failure` treats SIGTERM/`kill` as a clean exit and does not restart.
- **Live TEST**: `karu-tokyo-server`, `runo-tokyo`, `rs-ops` were stopped with SIGTERM and `rs-guard` with SIGKILL; after 6 s all were `active` with new PIDs.
- **PC (Windows) IME**: force-killing the resident engine `aruaru-ime-daemon.exe` brought `/health` back in about 3 s (2026-11-07).
- **Open**: confirming automatic recovery of other PC/phone apps (open-backup etc.), real-device checks for macOS (`KeepAlive`) and Linux, and per-repository notes (done by each assignee when they migrate REST).
- **When adding a unit**: start with `Restart=always`, `RestartSec=3`, `StartLimitIntervalSec=0`.

### 追加の実機TEST(2026-11-07)/ Additional live tests
- **日本語**: Linux(WSL Ubuntu・systemd): インストーラーと同じユニット(`Restart=always`)で `aruaru-ime-daemon` を動かし、SIGTERM・SIGKILL のどちらでも約10秒以内に別PIDで復帰、`systemctl stop` では止まったままを確認。Android 実機(moto g53y・Android 14): IME のプロセスを強制終了すると OS が再生成(PID 24034→24172)。端末の既定キーボードは試験後に Gboard へ戻した。端末に入っている版は 0.2.21 でリリースは 0.2.23(未更新)。macOS の `KeepAlive` は実機が無く、設定の静的確認のみ。
- **REST 移行の副作用で見つけた不具合を修正**: VPS の `maid-news-archive` が旧 `GET /v1/internal/news/export`(移行で削除済み)を呼んで404で失敗していたため、`open-english` の `scripts/archive-articles-to-github.sh` を `POST /openenglish.v1.Internal/NewsExport` に直して反映(成功を確認)。`daily-news-collect` は `scripts/daily-news-collect.sh` の実行権限が無く失敗していたので VPS で `chmod +x`(git 上のモードは 644 のままで、`aruaru-llm` 側で直す必要あり)。
- **English**: Linux (WSL Ubuntu, systemd): the installer's unit (`Restart=always`) brought `aruaru-ime-daemon` back with a new PID within ~10 s after both SIGTERM and SIGKILL, and `systemctl stop` stayed stopped. Android device (moto g53y, Android 14): force-killing the IME process made the OS recreate it (PID 24034→24172); the default keyboard was restored to Gboard afterwards. The installed build is 0.2.21 vs release 0.2.23 (not updated). macOS `KeepAlive` was only checked statically (no device).
- **Bugs found as side effects of the REST migration, fixed**: `maid-news-archive` on the VPS called the removed `GET /v1/internal/news/export` (404); `open-english`'s `scripts/archive-articles-to-github.sh` now uses `POST /openenglish.v1.Internal/NewsExport` and is deployed (verified). `daily-news-collect` failed because `scripts/daily-news-collect.sh` lost its exec bit; fixed with `chmod +x` on the VPS (git still records mode 644; `aruaru-llm` needs the fix).
