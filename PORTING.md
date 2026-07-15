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
