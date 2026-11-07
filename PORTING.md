# PORTING.md — お引越し可能ファイル

他プロジェクトへそのまま(または軽微な変更で)持っていける実装パターン一覧。

## `markdown_to_rs()` (src/main.rs)

Markdownの各行を`//!`付きrustdocコメント形式に変換する関数。README.md/CLAUDE.md/PORTING.mdのいずれにも汎用的に使える。readme-to-rs構想を採用する他リポジトリへそのままコピー可能。

```rust
fn markdown_to_rs(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| if line.is_empty() { "//!".to_string() } else { format!("//! {line}") })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
```

## `fetch_repo_file()` (src/main.rs)

GitHub raw contentを認証不要で取得するヘルパー(main→masterへのフォールバック付き)。組織名・タイムアウト値を差し替えるだけで他のGitHub連携機能に転用可能。

## nginx「conf.d優先」パターン

`/etc/nginx/conf.d/*.conf`は`/etc/nginx/sites-enabled/*.conf`より先にnginx.confでincludeされるため、同じ`server_name`を持つ設定を`conf.d/`に置くことで、UIツール(aruaru-easyweb等)が`sites-enabled/`側に自動生成する設定より優先させられる。他ドメインでも同じ競合が起きた場合に再利用可能な手法(詳細は`CLAUDE.md`参照)。

## GitHub API: Organization向けエンドポイントと個人アカウント向けエンドポイントの違い

`GET /orgs/{name}/repos`はGitHub Organizationにしか使えず、個人アカウント
(User)に対して呼ぶと404が返る(実機確認済み)。個人アカウントの全
リポジトリ一覧を取得するには`GET /users/{name}/repos`を使うこと。
`aon-co-jp`は個人アカウントであり、Organizationではない。

```rust
// ✗ 個人アカウントには使えない(404になる)
let url = format!("https://api.github.com/orgs/{name}/repos?per_page=100");
// ✓ 個人アカウント向け
let url = format!("https://api.github.com/users/{name}/repos?per_page=100");
```

GitHub連携機能を他プロジェクトへ移植する際、対象アカウントが
Organizationか個人アカウントかを`GET /users/{name}`(`"type"`フィールドが
`"Organization"`か`"User"`か)で事前確認するか、両エンドポイントを
試すフォールバックを検討すること。

## `markdown_to_github_style_html()` + `pulldown-cmark`(src/main.rs)

外部READMEをGitHub風に実際にレンダリングして表示する機能。`.rs`変換
表示(`markdown_to_rs`)と併用し、タブで切替可能にするパターン。

```rust
fn markdown_to_github_style_html(markdown: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(markdown, options);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out
}
```

表示先のCSSに`.markdown-body`風のスタイル(見出しの下線・コードブロック・
テーブル境界線等)を用意すると、GitHubの見た目に近づく。

## 狭い`main`コンテナからのbreak-out(vw基準の中央寄せ)

ページ全体は読みやすさのため狭い`max-width`(例: 780px)にしつつ、
特定セクション(README表示など)だけ画面幅いっぱいに広げたい場合:

```css
section.wide {
  width: 94vw;
  max-width: 1400px;
  position: relative;
  left: 50%;
  transform: translateX(-50%);
}
```

親コンテナの`max-width`に関わらず、viewport基準で中央寄せの広い
セクションを作れる。

## 動的取得したリストの検証は「形式検証」で行う(ホワイトリスト照合をしない)

サーバー起動時のハードコードされたリスト(`GITHUB_REPOS`等)と、
実行時にAPIから動的取得したリストを両方受け付けたい場合、入力値の
妥当性検証を「静的リストとの照合」で行うと、動的リストにしか
存在しない新しい値が弾かれてしまう。代わりに「形式が正しいか」
(英数字・ハイフン・アンダースコア・ドットのみ、長さ制限等)で
検証すること。

## 「ミラーlocation」パターン(Hostヘッダー書き換えによる内部プロキシ)

あるドメイン配下の特定パスだけを、別ドメインの実体へ内部的にプロキシしたい場合:

```nginx
location /some-path/ {
    proxy_pass http://127.0.0.1:80/some-path/;
    proxy_set_header Host other-domain.example;
    proxy_set_header X-Real-IP $remote_addr;
}
```

同一VPS上に複数ドメインが同居している構成で、コンテンツを複製せずに別ドメインからも閲覧可能にしたい場合に再利用できる。

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

## API 方針: REST 全廃・gRPC / Connect へ (2026-11-04)

### 日本語
- **REST は全廃する。新しい API は REST で作らない。** ブラウザや `curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`、本文は JSON)、サービス間は **gRPC / gRPC-web**(RPoem の管理API `open_runo.admin.v1.Admin` と同じ流儀)。ストリーミングは、必要になってから。
- **既存の REST は、触るときに順に移す。** 移したら旧 REST の経路は消す。移行が終わるまで残るが、**新規には追加しない**。
- **JSON は RS-JSON に統一**する(読み込みは `from_str_secure` / `parse_secure` でサイズ・深さ・重複キーの上限つき、書き出しは `rust_json::fast`)。ブラウザの WASM に `prost` は入れない(サイズが増えるため)。
- **例外(自分たちの API ではなく、相手の規格が決めているもの)**: ACME、OAuth/OIDC のコールバック、第三者(GitHub など)からの Webhook、ヘルスチェック、静的ファイル配信、OTLP など。これらは規格どおりの HTTP を受ける。
- 参考実装: `RPoem`(`open_runo.admin.v1.Admin`)、`open-easy-web`(`server/src/connect.rs`、`proto/open_easy_web/admin/v1/admin.proto`)。
- ※ このリポジトリのコードが既に方針どおりかどうかは、各リポジトリの現状による。この節は方針の記録であり、コードの変更を意味しない。

### English
- **REST is being abolished. New APIs are not built as REST.** APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`, JSON body); service-to-service calls use **gRPC / gRPC-web** (the same style as RPoem's admin API `open_runo.admin.v1.Admin`). Streaming only when it is actually needed.
- **Existing REST endpoints are migrated one by one, when they are touched.** The old REST route is removed once migrated. Until migration is finished they remain, but **no new ones are added**.
- **JSON is unified on RS-JSON**: reading with `from_str_secure` / `parse_secure` (size, depth and duplicate-key limits), writing with `rust_json::fast`. `prost` is not put into browser WASM (it would grow the binary).
- **Exceptions (shapes dictated by someone else's standard, not our own API)**: ACME, OAuth/OIDC callbacks, webhooks from third parties (e.g. GitHub), health checks, static file serving, OTLP, and so on. These accept plain HTTP as the standard requires.
- Reference implementations: `RPoem` (`open_runo.admin.v1.Admin`) and `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Note: whether the code in a given repository already follows this policy depends on that repository. This section records the policy; it does not mean the code has been changed.

### Deutsch (KI-Übersetzung, redaktionell überarbeitet; noch nicht von Muttersprachlern geprüft)
- **REST wird vollständig abgeschafft; neue APIs entstehen nicht mehr als REST.** APIs, die aus dem Browser oder per `curl` aufgerufen werden, verwenden **unäre Connect-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body). Zwischen Diensten kommen **gRPC bzw. gRPC-web** zum Einsatz, im selben Stil wie die Admin-API von RPoem (`open_runo.admin.v1.Admin`). Streaming wird erst ergänzt, wenn es tatsächlich gebraucht wird.
- **Bestehende REST-Endpunkte werden schrittweise umgestellt, und zwar immer dann, wenn man ohnehin an ihnen arbeitet.** Sobald ein Endpunkt umgestellt ist, wird die alte REST-Route entfernt. Bis zum Abschluss der Migration bleiben die übrigen bestehen, es werden aber **keine neuen mehr hinzugefügt**.
- **JSON wird einheitlich über RS-JSON verarbeitet**: Gelesen wird mit `from_str_secure` bzw. `parse_secure` (mit Obergrenzen für Größe, Verschachtelungstiefe und doppelte Schlüssel), geschrieben mit `rust_json::fast`. `prost` wird nicht in das Browser-WASM aufgenommen, da es die Datei vergrößern würde.
- **Ausnahmen (Schnittstellen, deren Form ein fremder Standard vorgibt und die nicht zu unserer eigenen API gehören)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Drittanbietern (z. B. GitHub), Health-Checks, die Auslieferung statischer Dateien, OTLP und Ähnliches. Diese nehmen, wie vom jeweiligen Standard verlangt, gewöhnliches HTTP entgegen.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines einzelnen Repositorys diese Richtlinie bereits erfüllt, hängt vom jeweiligen Repository ab. Dieser Abschnitt dokumentiert lediglich die Richtlinie; er besagt nicht, dass der Code schon angepasst wurde.

### Русский (перевод выполнен с помощью ИИ и отредактирован; носителями языка пока не проверен)
- **От REST мы полностью отказываемся; новые API больше не создаются в стиле REST.** API, к которым обращаются из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело запроса — JSON). Для взаимодействия между сервисами применяются **gRPC и gRPC-web** — в том же стиле, что и административный API RPoem (`open_runo.admin.v1.Admin`). Потоковая передача данных добавляется лишь тогда, когда в ней действительно возникает необходимость.
- **Существующие REST-эндпоинты переводятся поэтапно — по мере того, как с ними приходится работать.** После перевода старый REST-маршрут удаляется. Пока миграция не завершена, оставшиеся эндпоинты сохраняются, но **новые не добавляются**.
- **Работа с JSON унифицирована на основе RS-JSON**: чтение — через `from_str_secure` или `parse_secure` (с ограничениями на размер, глубину вложенности и повторяющиеся ключи), запись — через `rust_json::fast`. `prost` в браузерный WASM не включается, поскольку это увеличило бы размер файла.
- **Исключения (интерфейсы, форму которых определяет чужой стандарт, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и тому подобное. Такие интерфейсы принимают обычный HTTP, как того требует соответствующий стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Данный раздел лишь фиксирует политику и не означает, что код уже изменён.

## API 方針の補足: 用語の整理と使い分け (2026-11-04)

### 日本語
**1つに統一するものではなく、別の層の話です。** 「REST の代わり」になるのは Connect・gRPC・GraphQL だけで、WASM は画面を作る技術です。

| 用語 | 何か | REST の代わりか |
|---|---|---|
| REST | URL と HTTP メソッドで操作を表す API の作り方 | 全廃する側 |
| **Connect** | `POST /サービス名/メソッド名` に JSON を送る形式。`curl` でもブラウザの `fetch` でも呼べる | はい(**採用**) |
| **gRPC** | 同じ「サービス/メソッド」の考え方を、バイナリ(protobuf)と HTTP/2 で行う形式。速く、ストリーミングも得意 | はい(**サービス同士に採用**) |
| gRPC-web | gRPC のブラウザ向けの変形版 | はい(必要なら) |
| GraphQL | 欲しい項目を問い合わせる形式 | 一部だけ(**問い合わせ(読み取り)専用**) |
| WASM | Rust をブラウザの中で動かす仕組み | いいえ(画面を作る技術) |

- **ブラウザの画面や `curl` から呼ぶ API は Connect**(POST、JSON を RS-JSON で読み書き)。ブラウザの WASM は素の gRPC を話せず、Connect なら `fetch` のままで、`prost` を足さずに済む(画面のサイズも増えない)。
- **サービス同士の通信(サーバー間、重い処理、ストリーミング)は gRPC。** 多くは Connect で足りるので、ストリーミングが必要になった所だけ gRPC に上げる。
- **GraphQL は、まとめて問い合わせたいときの読み取り専用。** 書き込み(操作)には使わない。
- **WASM は画面を作る技術として、そのまま使う。** 画面から呼ぶ API が Connect になっただけ。
- 外部の規格が形を決めるもの(ACME、OAuth のコールバック、第三者の Webhook、ヘルスチェック、静的ファイル、OTLP)は、規格どおりの HTTP のまま。
- **迷ったら Connect。**

### English
**These are not alternatives to unify; they sit at different layers.** Only Connect, gRPC and GraphQL can replace REST; WASM is a way to build the UI.

| Term | What it is | Replaces REST? |
|---|---|---|
| REST | API style that expresses operations through URLs and HTTP methods | The thing being abolished |
| **Connect** | `POST /Service/Method` with a JSON body; callable from `curl` and from a browser `fetch` | Yes (**adopted**) |
| **gRPC** | The same service/method idea over binary (protobuf) and HTTP/2; fast, good at streaming | Yes (**for service-to-service**) |
| gRPC-web | A browser-friendly variant of gRPC | Yes (if needed) |
| GraphQL | A query language: ask for exactly the fields you want | Partly (**read-only queries**) |
| WASM | Runs Rust inside the browser | No (a UI technology) |

- **APIs called from browser screens or `curl` use Connect** (POST, JSON read and written with RS-JSON). Browser WASM cannot speak plain gRPC; with Connect it stays a `fetch`, and `prost` is not needed (the binary does not grow).
- **Service-to-service calls (server to server, heavy work, streaming) use gRPC.** Connect is enough in most places; move a call to gRPC only where streaming is actually needed.
- **GraphQL is read-only**, for queries that gather several pieces of data. It is not used for writes (operations).
- **WASM stays as the way to build the UI.** The API it calls simply became Connect.
- Interfaces whose shape an outside standard dictates (ACME, OAuth callbacks, third-party webhooks, health checks, static files, OTLP) stay plain HTTP.
- **When in doubt, use Connect.**

### Deutsch (KI-Übersetzung, sprachlich überarbeitet; noch nicht von Muttersprachlern geprüft)
**Es geht nicht darum, eine einzige Technik auszuwählen – die Begriffe liegen auf unterschiedlichen Ebenen.** Als Ersatz für REST kommen nur Connect, gRPC und GraphQL in Frage; WASM dagegen ist eine Technik, mit der die Benutzeroberfläche gebaut wird.

| Begriff | Was ist das? | Ersatz für REST? |
|---|---|---|
| REST | Ein API-Stil, bei dem Operationen über URLs und HTTP-Methoden ausgedrückt werden | wird abgeschafft |
| **Connect** | `POST /Dienst/Methode` mit JSON im Body; aufrufbar mit `curl` ebenso wie mit `fetch` im Browser | ja (**gewählt**) |
| **gRPC** | Dasselbe Prinzip aus Dienst und Methode, aber über binäres protobuf und HTTP/2; schnell und gut für Streaming geeignet | ja (**zwischen Diensten**) |
| gRPC-web | Eine für den Browser angepasste Variante von gRPC | ja (bei Bedarf) |
| GraphQL | Abfragesprache, mit der man genau die benötigten Felder anfordert | teilweise (**nur für lesende Abfragen**) |
| WASM | Führt Rust-Code direkt im Browser aus | nein (Technik für die Oberfläche) |

- **APIs, die von Browseroberflächen oder per `curl` aufgerufen werden, verwenden Connect** (POST; JSON wird mit RS-JSON gelesen und geschrieben). WASM im Browser kann kein reines gRPC sprechen. Mit Connect bleibt es bei einem gewöhnlichen `fetch`, und `prost` ist nicht nötig – die WASM-Datei wird dadurch nicht größer.
- **Für die Kommunikation zwischen Diensten (Server zu Server, rechenintensive Aufgaben, Streaming) wird gRPC eingesetzt.** In den meisten Fällen reicht Connect aus; auf gRPC wird nur dort umgestellt, wo Streaming tatsächlich gebraucht wird.
- **GraphQL dient ausschließlich lesenden Abfragen**, wenn mehrere Informationen gebündelt abgefragt werden sollen. Für Schreibzugriffe (Operationen) wird es nicht verwendet.
- **WASM bleibt die Technik für die Oberfläche.** Geändert hat sich lediglich, dass die aufgerufene API nun auf Connect beruht.
- Schnittstellen, deren Form durch einen externen Standard festgelegt ist (ACME, OAuth-Callbacks, Webhooks von Drittanbietern, Health-Checks, statische Dateien, OTLP), bleiben gewöhnliches HTTP.
- **Im Zweifel: Connect.**

### Русский (перевод выполнен с помощью ИИ и литературно отредактирован; носителями языка пока не проверен)
**Это не варианты, из которых нужно выбрать один, а технологии разных уровней.** Заменить REST могут только Connect, gRPC и GraphQL; WASM же — это способ создания пользовательского интерфейса.

| Термин | Что это | Заменяет REST? |
|---|---|---|
| REST | Стиль построения API, при котором операции выражаются через URL и HTTP-методы | отменяется |
| **Connect** | `POST /Сервис/Метод` с телом в формате JSON; вызывается как из `curl`, так и из браузерного `fetch` | да (**выбран**) |
| **gRPC** | Тот же принцип «сервис — метод», но поверх двоичного формата protobuf и HTTP/2; работает быстро и хорошо подходит для потоковой передачи | да (**для связи между сервисами**) |
| gRPC-web | Адаптированный для браузера вариант gRPC | да (при необходимости) |
| GraphQL | Язык запросов: клиент запрашивает ровно те поля, которые ему нужны | частично (**только запросы на чтение**) |
| WASM | Позволяет выполнять код на Rust прямо в браузере | нет (технология для интерфейса) |

- **API, которые вызываются из браузерных интерфейсов или через `curl`, строятся на Connect** (POST; JSON читается и записывается через RS-JSON). Браузерный WASM не умеет работать с «чистым» gRPC. С Connect достаточно обычного `fetch`, а `prost` не требуется — размер WASM-файла не увеличивается.
- **Для обмена между сервисами (сервер — сервер, ресурсоёмкие задачи, потоковая передача) применяется gRPC.** В большинстве случаев достаточно Connect; на gRPC переходят только там, где потоковая передача действительно необходима.
- **GraphQL используется только для чтения**, когда нужно получить сразу несколько связанных данных. Для записи (операций) он не применяется.
- **WASM по-прежнему остаётся технологией для интерфейса.** Изменилось лишь то, что вызываемый им API теперь построен на Connect.
- Интерфейсы, форма которых задана внешним стандартом (ACME, обратные вызовы OAuth, вебхуки сторонних сервисов, проверки работоспособности, статические файлы, OTLP), остаются обычным HTTP.
- **Если есть сомнения — выбирайте Connect.**

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
