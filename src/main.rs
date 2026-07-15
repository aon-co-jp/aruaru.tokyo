//! aruaru.tokyo — Rust + Poem 版TOPページ。
//! audiocafe.tokyo (PHP) とは異なり、こちらはpoem-cosmo-tauriのエコシステム
//! 方針に合わせてRust+Poemで実装する。DB非依存・1バイナリ完結。

use poem::listener::TcpListener;
use poem::web::{Html, Query};
use poem::{get, handler, Route, Server};
use rand::seq::SliceRandom;
use serde::Deserialize;

const ARUARU_EASYWEB_URL: &str = "http://127.0.0.1:8080/";
const GITHUB_ORG: &str = "aon-co-jp";

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
    if selected_repo.is_empty() || !GITHUB_REPOS.contains(&selected_repo) {
        return String::new();
    }
    let client = reqwest::Client::new();
    let mut out = String::new();
    for (filename, label, required) in REPO_FILES {
        let markdown = fetch_repo_file(&client, selected_repo, filename).await;
        out.push_str("<div class=\"repo-file-block\">\n");
        out.push_str(&format!("  <h3>{}</h3>\n", html_escape(label)));
        match markdown {
            Some(md) => {
                let rs = html_escape(&markdown_to_rs(&md));
                out.push_str(&format!("  <pre class=\"rs-output\">{rs}</pre>\n"));
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
  section.tool { background:var(--bg-card); border:1px solid var(--border); border-radius:.9rem; padding:1.25rem 1.5rem; margin:2.5rem 0; }
  section.tool h2 { font-size:1.1rem; margin:0 0 .5rem; color:var(--accent-2); }
  section.tool p.desc { color:var(--muted); font-size:.85rem; margin:0 0 1rem; }
  .repo-form { display:flex; gap:.6rem; flex-wrap:wrap; }
  .repo-form select { flex:1; min-width:200px; font:inherit; padding:.5rem .7rem; border-radius:.5rem; border:1px solid var(--border); background:var(--bg); color:var(--fg); }
  .repo-form button { padding:.5rem 1.2rem; }
  pre.rs-output { margin-top:1.25rem; background:#1e1e1e; color:#d4d4d4; border-radius:.6rem; padding:1rem 1.2rem; overflow-x:auto; font-family:"SFMono-Regular",Consolas,"Liberation Mono",Menlo,monospace; font-size:.82rem; line-height:1.5; max-height:480px; }
  .rs-error { color:#c0392b; font-size:.85rem; margin-top:1rem; }
  .repo-file-block { margin-top:1.5rem; }
  .repo-file-block h3 { font-size:.9rem; margin:0 0 .4rem; color:var(--muted); }
  footer { text-align:center; color:var(--muted); font-size:.8rem; margin-top:3rem; }
"#;

#[handler]
async fn top(Query(q): Query<TopQuery>) -> Html<String> {
    let selected_repo = q.repo.unwrap_or_default();
    let selected_repo = if GITHUB_REPOS.contains(&selected_repo.as_str()) { selected_repo } else { String::new() };

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
    <form class="repo-form" method="get">
      <select name="repo">
        <option value="">リポジトリを選択…</option>
        {repo_options}
      </select>
      <button type="submit">.rs に変換して表示</button>
    </form>
    {repo_results}
  </section>

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
    let app = Route::new().at("/", get(top)).at("/healthz", get(healthz));
    tracing::info!(%bind, "starting aruaru-tokyo-server");
    Server::new(TcpListener::bind(&bind)).run(app).await
}
