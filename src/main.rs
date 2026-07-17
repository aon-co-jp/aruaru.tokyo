//! aruaru.tokyo — Rust + Poem 版TOPページ。
//! audiocafe.tokyo (PHP) とは異なり、こちらはpoem-cosmo-tauriのエコシステム
//! 方針に合わせてRust+Poemで実装する。DB非依存・1バイナリ完結。

use poem::listener::TcpListener;
use poem::web::{Html, Query};
use poem::{get, handler, Route, Server};
use rand::seq::SliceRandom;
use serde::Deserialize;

const ARUARU_EASYWEB_URL: &str = "https://runo.tokyo/";
const GITHUB_ORG: &str = "aon-co-jp";
const GITHUB_ORG_URL: &str = "https://github.com/aon-co-jp";

const GITHUB_REPOS: &[&str] = &[
    "open-runo",
    "poem-cosmo-tauri",
    "open-web-server",
    "aruaru-db",
    "open-raid-z",
    "open-easyweb",
    "aruaru-easyweb",
    "rs-to-readme",
    "readme-to-rs",
    "aruaru-ai",
    "open-cuda",
];

/// リポジトリ名として妥当な形式か検証する(GitHubの実際の命名規則: 英数字・
/// ハイフン・アンダースコア・ドットのみ)。「🔄 最新のリポジトリ一覧を取得」で
/// 動的に取得したリポジトリは`GITHUB_REPOS`の静的リストに含まれないため、
/// 固定リストとの照合ではなくこの形式検証で受け付ける。
fn is_valid_repo_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 100
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

const REPO_FILES: &[(&str, &str, bool)] = &[
    ("README.md", "README(概要)", true),
    ("CLAUDE.md", "CLAUDE.md(開発方針 & 開発環境ルール)", false),
    ("PORTING.md", "PORTING.md(お引越し可能ファイル)", false),
];

struct RelatedSite {
    label_ja: &'static str,
    label_en: &'static str,
    url_ja: &'static str,
    url_en: &'static str,
}

const RELATED_SITES: &[RelatedSite] = &[
    RelatedSite {
        label_ja: "audiocafe.tokyo/aruaru(IT・建築系求人 日本語版)",
        label_en: "audiocafe.tokyo/aruaru (English, translated by Claude Code)",
        url_ja: "https://audiocafe.tokyo/aruaru/",
        url_en: "https://audiocafe.tokyo/aruaru/index-en.php",
    },
    RelatedSite {
        label_ja: "audiocafe.tokyo/aruaru-lady(女性向け求人 日本語版)",
        label_en: "audiocafe.tokyo/aruaru-lady (English, translated by Claude Code)",
        url_ja: "https://audiocafe.tokyo/aruaru-lady/",
        url_en: "https://audiocafe.tokyo/aruaru-lady/index-en.php",
    },
    RelatedSite {
        label_ja: "aon.tokyo(AI・IT・WEB・オーディオ)",
        label_en: "aon.tokyo (AI/IT/WEB & audio equipment)",
        url_ja: "https://aon.tokyo/",
        url_en: "https://aon.tokyo/",
    },
    RelatedSite {
        label_ja: "aon.co.jp(AI・IT・WEB・オーディオ、aon.tokyoと同一内容)",
        label_en: "aon.co.jp (AI/IT/WEB & audio equipment, same content as aon.tokyo)",
        url_ja: "https://aon.co.jp/",
        url_en: "https://aon.co.jp/",
    },
    RelatedSite {
        label_ja: "karu.tokyo(軽井沢・あきる野・東京の観光とリモートワーク)",
        label_en: "karu.tokyo (Karuizawa/Akiruno/Tokyo tourism & remote work)",
        url_ja: "https://karu.tokyo/",
        url_en: "https://karu.tokyo/",
    },
    RelatedSite {
        label_ja: "aruaru.tokyo/rakuten-mobile(楽天モバイル情報)",
        label_en: "aruaru.tokyo/rakuten-mobile (Rakuten Mobile info)",
        url_ja: "https://aruaru.tokyo/rakuten-mobile/",
        url_en: "https://aruaru.tokyo/rakuten-mobile/",
    },
];

fn categories() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "IT エンジニアあるある",
            vec![
                "「動かないんですけど」→ 5分後に自己解決している",
                "本番環境で試したらすぐ直る不具合、ローカルでは絶対再現しない",
                "会議は「あとでSlackで」で締めるのに結局Slackでも決まらない",
                "ドキュメントを書いた瞬間に仕様が変わる",
                "「ちょっと直しますね」が気づいたら3時間経っている",
            ],
        ),
        (
            "在宅ワークあるある",
            vec![
                "カメラオンの会議、下だけパジャマ",
                "「聞こえてますか?」を1日に3回は言う",
                "昼休みのつもりが気づいたら1時間半経っている",
                "宅配便のインターホンにビクッとする",
                "椅子から立った回数より水を飲んだ回数の方が少ない",
            ],
        ),
        (
            "朝活あるある",
            vec![
                "前日の夜は「明日は5時起きする」と誓う",
                "起きた瞬間には「今日はやめとこう」に変わっている",
                "三日坊主どころか初日で心が折れる",
                "それでも次の日また同じ誓いを立てる",
            ],
        ),
        (
            "SNSあるある",
            vec![
                "「見るだけのつもり」が気づいたら1時間",
                "投稿した3秒後に誤字を発見する",
                "「いいね」の数を意味もなく確認しにいく",
                "通知オフにしたはずなのに結局アプリを開いている",
            ],
        ),
        (
            "日本の会社あるある",
            vec![
                "「一応」で始まる念のための確認メールが多い",
                "会議の議題より雑談の方が盛り上がる",
                "有給を取る理由を無駄に考えてしまう",
                "「持ち帰って検討します」が一番よく使うフレーズ",
            ],
        ),
    ]
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// READMEなど各Markdownを`//!`付きrustdocコメント形式の`.rs`風テキストへ変換する
/// (readme-to-rs構想の簡易実装)。
fn markdown_to_rs(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| if line.is_empty() { "//!".to_string() } else { format!("//! {line}") })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// MarkdownをGitHub風の見た目でレンダリングしたHTMLへ変換する
/// (`pulldown-cmark`使用。見出し・リスト・コードブロック・リンク等の
/// 標準的なMarkdown記法に対応)。
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

/// GitHub APIから`aon-co-jp`の全リポジトリ名を最新の状態で取得する。
/// `aon-co-jp`はOrganizationではなく個人アカウントのため`/users/`
/// エンドポイントを使う(`/orgs/`だと404になることを実機確認済み)。
/// 認証無しの公開APIを使うため、レート制限(未認証: 60回/時/IP)に注意。
async fn fetch_org_repos(client: &reqwest::Client) -> Result<Vec<String>, String> {
    let url = format!("https://api.github.com/users/{GITHUB_ORG}/repos?per_page=100&sort=updated");
    let resp = client
        .get(&url)
        .header("User-Agent", "aruaru.tokyo-repo-list/0.1")
        .header("Accept", "application/vnd.github+json")
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("GitHub API returned {}", resp.status()));
    }
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let names = body
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|repo| repo.get("name").and_then(|n| n.as_str()).map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(names)
}

#[handler]
async fn api_repos() -> poem::web::Json<serde_json::Value> {
    let client = reqwest::Client::new();
    match fetch_org_repos(&client).await {
        Ok(names) => poem::web::Json(serde_json::json!({ "repos": names })),
        Err(e) => poem::web::Json(serde_json::json!({ "error": e })),
    }
}

async fn fetch_repo_file(client: &reqwest::Client, repo: &str, filename: &str) -> Option<String> {
    for branch in ["main", "master"] {
        let url = format!("https://raw.githubusercontent.com/{GITHUB_ORG}/{repo}/{branch}/{filename}");
        if let Ok(resp) = client
            .get(&url)
            .header("User-Agent", "aruaru.tokyo-readme-to-rs/0.1")
            .timeout(std::time::Duration::from_secs(6))
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(body) = resp.text().await {
                    return Some(body);
                }
            }
        }
    }
    None
}

#[derive(Deserialize)]
struct TopQuery {
    repo: Option<String>,
}

fn render_related_sites() -> String {
    RELATED_SITES
        .iter()
        .map(|s| {
            format!(
                r#"<a href="{ja_url}" target="_blank" rel="noopener">{ja_label}</a>
    <a href="{en_url}" target="_blank" rel="noopener">🌐 {en_label}</a>"#,
                ja_url = s.url_ja,
                ja_label = html_escape(s.label_ja),
                en_url = s.url_en,
                en_label = html_escape(s.label_en),
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ")
}

fn render_categories() -> String {
    categories()
        .into_iter()
        .map(|(cat, items)| {
            let lis = items
                .iter()
                .map(|item| format!("<li>{}</li>", html_escape(item)))
                .collect::<Vec<_>>()
                .join("\n      ");
            format!(
                r#"<section class="category">
    <h2>{cat}</h2>
    <ul>
      {lis}
    </ul>
  </section>"#,
                cat = html_escape(cat)
            )
        })
        .collect::<Vec<_>>()
        .join("\n  ")
}

fn render_repo_options(selected: &str) -> String {
    GITHUB_REPOS
        .iter()
        .map(|repo| {
            let sel = if *repo == selected { " selected" } else { "" };
            format!(r#"<option value="{repo}"{sel}>{repo}</option>"#)
        })
        .collect::<Vec<_>>()
        .join("\n        ")
}

async fn render_repo_results(selected_repo: &str) -> String {
    if !is_valid_repo_name(selected_repo) {
        return String::new();
    }
    let client = reqwest::Client::new();
    let repo_url = format!("{GITHUB_ORG_URL}/{selected_repo}");
    let mut out = String::new();
    out.push_str(&format!(
        "<p class=\"repo-link\"><a href=\"{repo_url}\" target=\"_blank\" rel=\"noopener\">🔗 {selected_repo} をGitHubで開く</a></p>\n"
    ));
    for (idx, (filename, label, required)) in REPO_FILES.iter().enumerate() {
        let markdown = fetch_repo_file(&client, selected_repo, filename).await;
        out.push_str("<div class=\"repo-file-block\">\n");
        out.push_str(&format!("  <h3>{}</h3>\n", html_escape(label)));
        match markdown {
            Some(md) => {
                let gh_html = markdown_to_github_style_html(&md);
                let rs = html_escape(&markdown_to_rs(&md));
                let tab_id = format!("{selected_repo}-{idx}");
                out.push_str(&format!(
                    r#"  <div class="view-toggle" data-tab="{tab_id}">
    <button type="button" class="view-toggle-btn active" data-view="gh">GitHub風表示</button>
    <button type="button" class="view-toggle-btn" data-view="rs">.rs形式</button>
  </div>
  <div class="markdown-body" id="gh-{tab_id}">{gh_html}</div>
  <pre class="rs-output hidden" id="rs-{tab_id}">{rs}</pre>
"#
                ));
            }
            None => {
                let msg = if *required {
                    format!("❌ {filename} を取得できませんでした({selected_repo})。")
                } else {
                    format!("❌ {filename} はこのリポジトリにはありません。")
                };
                out.push_str(&format!("  <p class=\"rs-error\">{}</p>\n", html_escape(&msg)));
            }
        }
        out.push_str("</div>\n");
    }
    out
}

fn flat_items_json() -> String {
    let mut items = Vec::new();
    for (cat, list) in categories() {
        for text in list {
            items.push(serde_json::json!({"category": cat, "text": text}));
        }
    }
    // shuffle server-side once at render time is unnecessary; client shuffles on click.
    let _ = items.choose(&mut rand::thread_rng());
    serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string())
}

const STYLE: &str = r#"
  :root { color-scheme: light dark; --bg:#fdf6ec; --bg-card:#ffffff; --fg:#2c2418; --muted:#8a7f6b; --accent:#ff7a45; --accent-2:#2f6fed; --border:#eadfc9; }
  @media (prefers-color-scheme: dark) { :root { --bg:#1a1712; --bg-card:#24201a; --fg:#f1ece0; --muted:#b3a893; --accent:#ff9466; --accent-2:#6ea1ff; --border:#3a3327; } }
  * { box-sizing: border-box; }
  body { margin:0; font-family:"Hiragino Sans","Noto Sans JP",system-ui,sans-serif; background:var(--bg); color:var(--fg); line-height:1.7; }
  main { max-width:780px; margin:0 auto; padding:2.5rem 1.25rem 5rem; }
  header { text-align:center; margin-bottom:1.25rem; }
  header h1 { font-size:2rem; margin:0 0 .4rem; letter-spacing:-.02em; }
  header h1 span { color:var(--accent); }
  header p { color:var(--muted); margin:0; }
  .quick-links { text-align:center; margin-bottom:2rem; }
  .quick-links a { display:inline-block; text-decoration:none; font-weight:600; background:var(--accent-2); color:#fff; border-radius:999px; padding:.55rem 1.4rem; margin:.2rem; font-size:.9rem; }
  .shuffle-bar { text-align:center; margin:1.5rem 0 2.5rem; }
  button { cursor:pointer; font:inherit; font-weight:600; background:var(--accent); color:#fff; border:none; border-radius:999px; padding:.7rem 1.6rem; box-shadow:0 2px 8px rgba(0,0,0,.12); }
  #shuffle-result { display:none; background:var(--bg-card); border:1px solid var(--border); border-radius:.75rem; padding:1.25rem 1.5rem; margin:1.5rem auto 0; max-width:640px; text-align:center; }
  #shuffle-result .cat { color:var(--accent-2); font-size:.8rem; font-weight:600; }
  #shuffle-result .txt { font-size:1.15rem; margin-top:.4rem; }
  .category { background:var(--bg-card); border:1px solid var(--border); border-radius:.9rem; padding:1.25rem 1.5rem; margin-bottom:1.25rem; }
  .category h2 { font-size:1.05rem; margin:0 0 .75rem; color:var(--accent-2); }
  .category ul { margin:0; padding-left:1.2rem; }
  .category li { margin-bottom:.5rem; }
  /* GitHub風README表示は横幅いっぱいに使うため、main の780px制約から
     意図的にbreak-outさせる(vw基準の中央寄せトリック)。 */
  section.tool { background:var(--bg-card); border:1px solid var(--border); border-radius:.9rem; padding:1.25rem 1.75rem; margin:2.5rem 0; width:94vw; max-width:1400px; position:relative; left:50%; transform:translateX(-50%); }
  section.tool h2 { font-size:1.1rem; margin:0 0 .5rem; color:var(--accent-2); }
  section.tool p.desc { color:var(--muted); font-size:.85rem; margin:0 0 1rem; }
  .repo-form { display:flex; gap:.6rem; flex-wrap:wrap; }
  .repo-form select { flex:1; min-width:200px; font:inherit; padding:.5rem .7rem; border-radius:.5rem; border:1px solid var(--border); background:var(--bg); color:var(--fg); }
  .repo-form button { padding:.5rem 1.2rem; }
  pre.rs-output { margin-top:1.25rem; background:#1e1e1e; color:#d4d4d4; border-radius:.6rem; padding:1.25rem 1.5rem; overflow-x:auto; font-family:"SFMono-Regular",Consolas,"Liberation Mono",Menlo,monospace; font-size:.82rem; line-height:1.5; max-height:70vh; width:100%; }
  .rs-error { color:#c0392b; font-size:.85rem; margin-top:1rem; }
  .repo-file-block { margin-top:1.5rem; }
  .repo-file-block h3 { font-size:.9rem; margin:0 0 .4rem; color:var(--muted); }
  .repo-link { margin:0 0 1rem; font-size:.9rem; }
  .repo-fetch-row { display:flex; align-items:center; gap:.6rem; margin-bottom:.75rem; flex-wrap:wrap; }
  .repo-fetch-row button { padding:.4rem 1rem; font-size:.85rem; }
  .repo-fetch-status { font-size:.8rem; color:var(--muted); }
  .view-toggle { display:flex; gap:.4rem; margin-bottom:.5rem; }
  .view-toggle-btn { background:transparent; color:var(--muted); border:1px solid var(--border); border-radius:999px; padding:.3rem .9rem; font-size:.78rem; font-weight:600; box-shadow:none; }
  .view-toggle-btn.active { background:var(--accent-2); color:#fff; border-color:var(--accent-2); }
  .hidden { display:none !important; }
  .markdown-body { background:var(--bg-card); border:1px solid var(--border); border-radius:.6rem; padding:1.5rem 2rem; overflow-x:auto; max-height:70vh; overflow-y:auto; width:100%; }
  .markdown-body img { max-width:100%; }
  .markdown-body ul, .markdown-body ol { padding-left:1.6rem; }
  .markdown-body h1, .markdown-body h2, .markdown-body h3 { border-bottom:1px solid var(--border); padding-bottom:.3rem; }
  .markdown-body code { background:rgba(148,163,184,.18); padding:.1rem .35rem; border-radius:.3rem; font-family:"SFMono-Regular",Consolas,"Liberation Mono",Menlo,monospace; font-size:.85em; }
  .markdown-body pre { background:#1e1e1e; color:#d4d4d4; padding:.8rem 1rem; border-radius:.5rem; overflow-x:auto; }
  .markdown-body pre code { background:none; padding:0; }
  .markdown-body table { border-collapse:collapse; width:100%; font-size:.88rem; }
  .markdown-body th, .markdown-body td { border:1px solid var(--border); padding:.4rem .6rem; }
  .markdown-body blockquote { border-left:3px solid var(--accent); margin:0; padding:.2rem 1rem; color:var(--muted); }
  .markdown-body a { color:var(--accent-2); }
  .org-link { text-align:center; margin:2rem 0; }
  .org-link a { color:var(--accent-2); font-weight:600; text-decoration:none; }
  footer { text-align:center; color:var(--muted); font-size:.8rem; margin-top:3rem; }
"#;

#[handler]
async fn top(Query(q): Query<TopQuery>) -> Html<String> {
    let selected_repo = q.repo.unwrap_or_default();
    let selected_repo = if is_valid_repo_name(&selected_repo) { selected_repo } else { String::new() };

    let repo_results = render_repo_results(&selected_repo).await;
    let related_sites = render_related_sites();
    let categories_html = render_categories();
    let repo_options = render_repo_options(&selected_repo);
    let flat_json = flat_items_json();

    let body = format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>aruaru.tokyo | みんなの「あるある」集めました</title>
<meta name="description" content="IT・在宅ワーク・SNS・日本の会社など、誰もが頷く「あるある」をジャンル別にまとめたサイト。GitHubリポジトリのREADMEを.rs風に変換して表示する機能も搭載。Rust+Poem製。">
<style>{STYLE}</style>
</head>
<body>
<main>
  <header>
    <h1>aruaru<span>.tokyo</span></h1>
    <p>「それ、あるある!」を集めました。 / A collection of everyday "aruaru" moments.</p>
  </header>

  <div class="quick-links">
    <a href="{ARUARU_EASYWEB_URL}" target="_blank" rel="noopener">🔧 aruaru-easyweb を開く</a>
    <br />
    {related_sites}
  </div>

  <div class="shuffle-bar">
    <button id="shuffle-btn" type="button">🎲 ランダムに1つ表示</button>
    <div id="shuffle-result">
      <div class="cat"></div>
      <div class="txt"></div>
    </div>
  </div>

  {categories_html}

  <section class="tool">
    <h2>📄 README/CLAUDE.md/PORTING.md → .rs 変換ビューア</h2>
    <p class="desc">
      GitHub({GITHUB_ORG})のリポジトリを選ぶと、README(概要)・CLAUDE.md
      (開発方針 & 開発環境ルール)・PORTING.md(お引越し可能ファイル)を
      それぞれrustdocコメント(<code>//!</code>)形式の <code>.rs</code> 風
      テキストに変換して表示します。(readme-to-rs構想の簡易実装、Rust+Poem実装)
    </p>
    <div class="repo-fetch-row">
      <button type="button" id="fetch-repos-btn">🔄 最新のリポジトリ一覧を取得</button>
      <span class="repo-fetch-status" id="fetch-repos-status"></span>
    </div>
    <form class="repo-form" method="get">
      <select name="repo" id="repo-select">
        <option value="">リポジトリを選択…</option>
        {repo_options}
      </select>
      <button type="submit">表示</button>
    </form>
    {repo_results}
  </section>

  <div class="org-link">
    <a href="{GITHUB_ORG_URL}" target="_blank" rel="noopener">🏢 GitHub organization: {GITHUB_ORG} のトップを見る</a>
  </div>

  <footer>&copy; 2026 aruaru.tokyo (Rust + Poem)</footer>
</main>
<script>
  const items = {flat_json};
  const btn = document.getElementById('shuffle-btn');
  const result = document.getElementById('shuffle-result');
  btn.addEventListener('click', () => {{
    const pick = items[Math.floor(Math.random() * items.length)];
    result.querySelector('.cat').textContent = pick.category;
    result.querySelector('.txt').textContent = pick.text;
    result.style.display = 'block';
  }});

  // ボタンを押した瞬間にGitHub組織の最新リポジトリ一覧をAPI経由で取得し、
  // <select>を動的に差し替える。
  const fetchBtn = document.getElementById('fetch-repos-btn');
  const fetchStatus = document.getElementById('fetch-repos-status');
  const repoSelect = document.getElementById('repo-select');
  fetchBtn.addEventListener('click', async () => {{
    fetchStatus.textContent = '取得中… / Fetching…';
    try {{
      const res = await fetch('/api/repos');
      const data = await res.json();
      if (data.error) {{ fetchStatus.textContent = '❌ ' + data.error; return; }}
      const repos = data.repos || [];
      const current = repoSelect.value;
      repoSelect.innerHTML = '<option value="">リポジトリを選択…</option>' +
        repos.map(r => `<option value="${{r}}">${{r}}</option>`).join('');
      if (repos.includes(current)) repoSelect.value = current;
      fetchStatus.textContent = `✅ ${{repos.length}}件取得しました。`;
    }} catch (e) {{
      fetchStatus.textContent = '❌ 取得に失敗しました。';
    }}
  }});

  // GitHub風表示 / .rs形式 の切替タブ配線。
  document.querySelectorAll('.view-toggle').forEach(toggle => {{
    const tabId = toggle.getAttribute('data-tab');
    const ghEl = document.getElementById('gh-' + tabId);
    const rsEl = document.getElementById('rs-' + tabId);
    toggle.querySelectorAll('.view-toggle-btn').forEach(b => {{
      b.addEventListener('click', () => {{
        toggle.querySelectorAll('.view-toggle-btn').forEach(x => x.classList.remove('active'));
        b.classList.add('active');
        const view = b.getAttribute('data-view');
        if (view === 'gh') {{ ghEl.classList.remove('hidden'); rsEl.classList.add('hidden'); }}
        else {{ ghEl.classList.add('hidden'); rsEl.classList.remove('hidden'); }}
      }});
    }});
  }});
</script>
</body>
</html>
"#
    );
    Html(body)
}

#[handler]
fn healthz() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();
    let bind = std::env::var("ARUARU_TOKYO_BIND").unwrap_or_else(|_| "0.0.0.0:4100".to_string());
    let app = Route::new()
        .at("/", get(top))
        .at("/healthz", get(healthz))
        .at("/api/repos", get(api_repos));
    tracing::info!(%bind, "starting aruaru-tokyo-server");
    Server::new(TcpListener::bind(&bind)).run(app).await
}
