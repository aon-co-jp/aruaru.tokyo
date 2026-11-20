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
