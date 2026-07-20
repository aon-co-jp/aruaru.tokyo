//! `/open-aruaru-runo-iLumi`(エイリアス`/open-aruaru-runo`)ページ。
//!
//! `F:\open-runo`エコシステム全体のメタ索引リポジトリ
//! [`aon`](https://github.com/aon-co-jp/aon)(旧称: `open-aruaru-runo-iLumi`、
//! 2026-07-20にユーザーがリポジトリ名を変更)と同じ内容(プロジェクト一覧・
//! README/PORTING.md/CLAUDE.mdへのリンク)を
//! Webページとして表示する。GitHubのリポジトリファイル一覧のような
//! 見た目(📄アイコン+ファイル名+リンクのミニリスト)で各プロジェクトの
//! ファイルを提示し、クリック時にGitHub APIから最新情報(⭐Star数・
//! 最終更新日時・既定ブランチ)を取得するボタンを添える。
//!
//! 13ヶ国語対応(`?lang=`)・言語別UIについては`crate::i18n`参照。
//!
//! **正直な開示**: 各プロジェクトの役割説明(`ROLE_JA`)は日本語のみで、
//! 13言語へは翻訳していない(スコープの限界、`crate::i18n`冒頭コメント
//! 参照)。

use crate::html_escape;
use crate::i18n::{index_strings, Lang};

/// メタ索引リポジトリ`aon`(旧称: `open-aruaru-runo-iLumi`)のREADME.mdに掲載した
/// プロジェクト一覧と全く同じ内容(名称・GitHubリポジトリ・各ファイルの
/// 有無・役割の要約)。`F:\open-runo`配下で実際にgitリポジトリとして
/// 存在し、`.git/config`のリモートURLで`aon-co-jp`上の実在を確認した
/// プロジェクトのみを掲載している(推測での追加はしていない)。
pub struct Project {
    pub name: &'static str,
    /// `aon-co-jp/<repo>`形式(GitHub APIの`owner/repo`)。
    pub repo: &'static str,
    pub has_readme: bool,
    pub has_porting: bool,
    pub has_claude: bool,
    pub role_ja: &'static str,
}

pub const PROJECTS: &[Project] = &[
    Project {
        name: "RCosmo",
        repo: "aon-co-jp/RCosmo",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "Rust製GraphQL Federationプラットフォーム(Poem/Tauri/Cosmoは非依存・互換自前実装)。WunderGraph Cosmoの有料版機能をOSS・Pure Rustで実現し、独自の自己学習AIを搭載(外部LLM契約不要)。姉妹リポジトリpoem-cosmo-tauriと並行開発。",
    },
    Project {
        name: "RFrontEnd",
        repo: "aon-co-jp/RFrontEnd",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "HTML5/CSS3/TypeScript/React相当を、既存実装のコードを一切流用せず一から開発する複数プロジェクト(RHTML/RCSS/RTypeScript等)を束ねる親リポジトリ。",
    },
    Project {
        name: "RPoem",
        repo: "aon-co-jp/RPoem",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "RCosmoと同種のRust製GraphQL Federationプラットフォーム。open-runoを正本として分岐したpoem-runoをさらにリネーム・統合した後継リポジトリ。",
    },
    Project {
        name: "aruaru-db",
        repo: "aon-co-jp/aruaru-db",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "CockroachDBの分散強整合×Snowflakeのストレージ/コンピュート分離×Git-on-SQLバージョン管理を、すべてPure Rustで実装するハイブリッド分散データベース。",
    },
    Project {
        name: "aruaru-llm",
        repo: "aon-co-jp/aruaru-llm",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "aruaruエコシステム(aruaru-tokyo・aruaru-db・e-gov.info・karu.tokyo等)共通の「AIチャットコマース」応答サービス。リポジトリ名は「LLM」を冠するが、実際のLLM推論への差し替えは今後という正直な開示がREADMEに明記されている。",
    },
    Project {
        name: "aruaru-tokyo",
        repo: "aon-co-jp/aruaru-tokyo",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "aruaru.tokyoのTOPページ(このリポジトリ自身)。Rust + Poem製、DB非依存・1バイナリ完結。audiocafe.tokyo(PHP)とは別ドメイン・別スタックの姉妹サイト。",
    },
    Project {
        name: "audiocafe-tokyo-rust",
        repo: "aon-co-jp/audiocafe-tokyo-rust",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "audiocafe.tokyoの既存PHPモノリスをRust + Poemへ段階的に移行するプロジェクト(第一段)。既存PHP実装はaudiocafe-tokyoリポジトリのまま並行運用。",
    },
    Project {
        name: "audiocafe.tokyo",
        repo: "aon-co-jp/audiocafe-tokyo",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "PHP製のマルチコンテンツサイト。IT/建築系求人情報(aruaru)・女性向け求人/夜間エンターテインメント情報(aruaru-lady)・楽天モバイル関連情報・会社案内などを扱う。",
    },
    Project {
        name: "e-gov.info",
        repo: "aon-co-jp/e-gov",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "行政のデジタル化と、個人〜貿易商社まで対応するオンライン貿易・不動産プラットフォームを、LINEアプリ・WEBサイト・コンビニ端末という複数の入口から統合するプロジェクト。まだサンプル・デモンストレーション段階(READMEに明記)。",
    },
    Project {
        name: "karu.tokyo",
        repo: "aon-co-jp/karu-tokyo",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "karu.tokyoのTOPページ。Rust + Poem製、DB非依存の1バイナリ完結サーバー。軽井沢・あきる野市・東京の観光とリモートワーク、IT・AI・AUDIO・貿易産業を紹介。",
    },
    Project {
        name: "open-cuda",
        repo: "aon-co-jp/open-cuda",
        has_readme: false,
        has_porting: false,
        has_claude: false,
        role_ja: "OmniGPU設計文書(OmniGPU-Design.md)等を含むGPUランタイム関連プロジェクト(README.mdの代わりにREADME-Japan.md等が存在、標準構成のCLAUDE.md/PORTING.mdは現時点で未整備)。",
    },
    Project {
        name: "open-easy-web",
        repo: "aon-co-jp/open-easy-web",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "「第二のKUSANAGI」——アプリのアップロード後にIPアドレスで起動し、ドメイン登録・HTTPS化を簡単に自動適用できる運用ツール(Rust → WebAssembly、フレームワーク不使用)。",
    },
    Project {
        name: "open-raid-z",
        repo: "aon-co-jp/open-raid-z",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "Rust製の実マウント可能なRAID-Z/Z2/Z3ストレージプール実装(ZFS「風」のCoW/チェックサム/スナップショット)。エコシステム開発ルールの正本リポジトリ。",
    },
    Project {
        name: "open-web-server",
        repo: "aon-co-jp/open-web-server",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "Rust + tokio/hyper自前実装のWebサーバー——課金アイテム・金融データを「消失させない」ために設計。open-runo・aruaru-dbと4層防御通信で連携するミッションクリティカル向け。",
    },
    Project {
        name: "rs-to-readme",
        repo: "aon-co-jp/rs-to-readme",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "Rustクレートの Cargo.toml メタデータからREADME.mdを自動生成するCLIツール(crates.io公開)。",
    },
    Project {
        name: "aon (旧称: open-aruaru-runo-iLumi)",
        repo: "aon-co-jp/aon",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "このエコシステム全体の「プロジェクトシリーズ索引」を担うメタリポジトリ。個別のコード実装は持たない。2026-07-20に open-aruaru-runo-iLumi から aon へリポジトリ名を変更(内容は移行済み)。",
    },
];

fn file_line(repo_url: &str, filename: &str, label: &str, present: bool, none_label: &str) -> String {
    if present {
        format!(
            r#"<li>📄 <a href="{repo_url}/blob/main/{filename}" target="_blank" rel="noopener">{label}</a></li>"#,
            repo_url = repo_url,
            filename = filename,
            label = html_escape(label),
        )
    } else {
        format!(r#"<li>📄 <span class="file-missing">{label} {none}</span></li>"#, label = html_escape(label), none = html_escape(none_label))
    }
}

fn render_project_card(p: &Project, s: &crate::i18n::IndexStrings) -> String {
    let repo_url = format!("https://github.com/{}", p.repo);
    let files = format!(
        "{}\n      {}\n      {}",
        file_line(&repo_url, "README.md", s.label_readme, p.has_readme, s.label_none),
        file_line(&repo_url, "PORTING.md", s.label_porting, p.has_porting, s.label_none),
        file_line(&repo_url, "CLAUDE.md", s.label_claude, p.has_claude, s.label_none),
    );
    format!(
        r#"<article class="repo-card" data-repo="{repo}">
  <h3><a href="{repo_url}" target="_blank" rel="noopener">📦 {name}</a></h3>
  <p class="role">{role}</p>
  <ul class="file-list">
      {files}
  </ul>
  <div class="live-fetch-row">
    <button type="button" class="live-fetch-btn" data-repo="{repo}">{btn_label}</button>
    <span class="live-fetch-result" data-repo-result="{repo}"></span>
  </div>
</article>"#,
        repo = html_escape(p.repo),
        repo_url = repo_url,
        name = html_escape(p.name),
        role = html_escape(p.role_ja),
        files = files,
        btn_label = html_escape(s.btn_fetch_live),
    )
}

fn render_lang_switcher(current: Lang, path: &str) -> String {
    Lang::ALL
        .iter()
        .map(|&l| {
            let cur = if l == current { " class=\"current\"" } else { "" };
            format!(r#"<a href="{path}?lang={code}"{cur}>{name}</a>"#, path = path, code = l.code(), name = html_escape(l.native_name()))
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

pub const META_INDEX_STYLE: &str = r#"
  .meta-index main { max-width: 980px; }
  .lang-switch { text-align:center; font-size:.78rem; margin-bottom:1.5rem; line-height:2; }
  .lang-switch a { text-decoration:none; color:var(--muted); margin:0 .15rem; }
  .lang-switch a.current { color:var(--accent-2); font-weight:700; }
  .meta-intro { color:var(--muted); font-size:.9rem; margin-bottom:2rem; }
  .repo-card { background:var(--bg-card); border:1px solid var(--border); border-radius:.75rem; padding:1.1rem 1.4rem; margin-bottom:1.1rem; }
  .repo-card h3 { margin:0 0 .4rem; font-size:1.02rem; }
  .repo-card h3 a { text-decoration:none; }
  .repo-card p.role { color:var(--muted); font-size:.85rem; margin:0 0 .75rem; line-height:1.6; }
  ul.file-list { list-style:none; margin:0 0 .75rem; padding:0; font-family:"SFMono-Regular",Consolas,"Liberation Mono",Menlo,monospace; font-size:.82rem; }
  ul.file-list li { padding:.25rem 0; border-bottom:1px dashed var(--border); }
  ul.file-list li:last-child { border-bottom:none; }
  ul.file-list a { text-decoration:none; color:var(--accent-2); }
  .file-missing { color:var(--muted); font-style:italic; }
  .live-fetch-row { display:flex; align-items:center; gap:.6rem; flex-wrap:wrap; }
  .live-fetch-btn { font-size:.78rem; padding:.35rem .9rem; }
  .live-fetch-result { font-size:.78rem; color:var(--muted); }
  .live-fetch-result .ok { color:var(--fg); }
"#;

/// `/open-aruaru-runo-iLumi`ページ本体のHTMLを組み立てる。
pub fn render_page(lang: Lang, canonical_path: &str) -> String {
    let s = index_strings(lang);
    let dir = if lang.is_rtl() { "rtl" } else { "ltr" };
    let cards = PROJECTS.iter().map(|p| render_project_card(p, &s)).collect::<Vec<_>>().join("\n  ");
    let lang_switch = render_lang_switcher(lang, canonical_path);

    format!(
        r#"<!DOCTYPE html>
<html lang="{html_lang}" dir="{dir}">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<meta name="description" content="{intro}">
<style>{STYLE}{META_INDEX_STYLE}</style>
</head>
<body class="meta-index">
<main>
  <header>
    <h1>{h1}</h1>
  </header>

  <div class="lang-switch">{lang_switch}</div>

  <div class="org-link">
    <a href="https://github.com/aon-co-jp" target="_blank" rel="noopener">{org_link_label}</a>
  </div>

  <p class="meta-intro">{intro}</p>

  {cards}

  <div class="org-link">
    <a href="/">{back_to_top}</a>
  </div>

  <footer>&copy; 2026 aruaru.tokyo (Rust + Poem) — meta index for the F:\open-runo ecosystem</footer>
</main>
<script>
  document.querySelectorAll('.live-fetch-btn').forEach(btn => {{
    const repo = btn.getAttribute('data-repo');
    const resultEl = document.querySelector('.live-fetch-result[data-repo-result="' + repo + '"]');
    btn.addEventListener('click', async () => {{
      resultEl.textContent = {loading_json};
      try {{
        const res = await fetch('https://api.github.com/repos/' + repo, {{
          headers: {{ 'Accept': 'application/vnd.github+json' }}
        }});
        if (!res.ok) throw new Error('status ' + res.status);
        const data = await res.json();
        const stars = data.stargazers_count;
        const updated = data.pushed_at || data.updated_at;
        const branch = data.default_branch;
        resultEl.innerHTML = '<span class="ok">' + {stars_label_json} + ': ' + stars +
          ' / ' + {updated_label_json} + ': ' + new Date(updated).toLocaleDateString() +
          ' / ' + {branch_label_json} + ': ' + branch + '</span>';
      }} catch (e) {{
        resultEl.textContent = {fail_json};
      }}
    }});
  }});
</script>
</body>
</html>
"#,
        html_lang = lang.html_lang(),
        dir = dir,
        title = html_escape(s.title),
        intro = html_escape(s.intro),
        STYLE = crate::STYLE,
        META_INDEX_STYLE = META_INDEX_STYLE,
        h1 = html_escape(s.h1),
        lang_switch = lang_switch,
        org_link_label = html_escape(s.org_link_label),
        cards = cards,
        back_to_top = html_escape(s.back_to_top),
        loading_json = serde_json::to_string(s.fetch_loading).unwrap_or_default(),
        stars_label_json = serde_json::to_string(s.field_stars).unwrap_or_default(),
        updated_label_json = serde_json::to_string(s.field_updated).unwrap_or_default(),
        branch_label_json = serde_json::to_string(s.field_default_branch).unwrap_or_default(),
        fail_json = serde_json::to_string(s.fetch_fail).unwrap_or_default(),
    )
}
