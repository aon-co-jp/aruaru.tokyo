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
- `fetch_org_repos()` / `api_repos` ハンドラ(`GET /api/repos`) — `aon-co-jp`の全リポジトリ名をGitHub APIから毎回最新取得する。**注意: `aon-co-jp`はOrganizationではなく個人アカウントのため`/users/{name}/repos`エンドポイントを使う**(`/orgs/`だと404になることを実機確認済み——今後同様のAPI呼び出しを追加する際はこの点に注意)。未認証呼び出しのためGitHub APIのレート制限(60回/時/IP)を受ける。
- `is_valid_repo_name()` — リポジトリ名の形式検証。動的取得したリポジトリは`GITHUB_REPOS`の静的リストに含まれないため、固定リストとの照合ではなくこの形式検証で受け付ける(この検証を怠ると、動的一覧から選んだ新しいリポジトリの表示が固定リストのホワイトリストで弾かれてしまう)。
- `top()` ハンドラ — TOPページ全体のHTML組み立て(`Query<TopQuery>`で`?repo=`パラメータを受ける)

## デプロイ

VPS(ConoHa、AlmaLinux)上で直接`cargo build --release`し、生成バイナリをsystemdサービス(`aruaru-tokyo-server.service`)として`127.0.0.1:4100`にバインド。nginx(`/etc/nginx/conf.d/aruaru.tokyo.conf`)が443番でTLS終端し、`location /`でこのポートへリバースプロキシする。

**`/aruaru/`・`/aruaru-lady/`・`/rakuten-mobile/`のミラーlocation**: これらのパスは元々`audiocafe.tokyo`側にのみ存在するコンテンツ(PHP)で、`aruaru.tokyo`からも直接閲覧できるようにするため、nginx側で`http://127.0.0.1:80/`(Hostヘッダーを`audiocafe.tokyo`に上書き)へ内部プロキシするlocationブロックを個別に追加している。このバイナリ自体はこれらのパスを一切処理しない(`Route`には`/`と`/healthz`しか登録されていない)。

## 関連プロジェクト

- [open-runo](https://github.com/aon-co-jp/open-runo) — Rust→WASM/tokio+hyperのopen-runoエコシステム本体
- [RPoem](https://github.com/aon-co-jp/RPoem) — Poem/Tauri実装規約の出典元、このリポジトリの技術スタック規約はここに準拠
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — 汎用WEBサーバーゲートウェイ(Apache的な位置付け)
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — DB層(このリポジトリはDB非依存だが、同エコシステムのDB実装)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — 開発ルールの正本、ストレージ関連
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — OTP認証・サイト管理サーバー(tokio+hyper製)、`ARUARU_EASYWEB_URL`のリンク先(`https://runo.tokyo/`)
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo)(PHP) — `/aruaru/`・`/aruaru-lady/`・`/rakuten-mobile/`ミラーlocationの実体

## 運用ルール

- このリポジトリにDB依存機能・重量級フレームワークを持ち込まないこと。
- `ARUARU_EASYWEB_URL`などの外部リンク定数は、必ず公開URLを使うこと(`127.0.0.1`などのループバックアドレスを埋め込むと、閲覧者のブラウザからは到達不能になるバグを生む——2026-07-15に実際に発生し修正した実例あり)。
- VPSの実IPアドレスをコード・ドキュメントに記録しないこと(既存の運用ルールを継承)。

## 現状

- 2026-07-15 ブートストラップ・本番投入完了。VPS上の実バイナリで動作確認済み(TOPページ・repo=パラメータ経由の`.rs`変換ビューアともに200 OK、実際にGitHub raw contentを取得して変換表示することを確認)。
- `/aruaru/`・`/aruaru-lady/`・`/rakuten-mobile/`のミラーlocationも実際に200が返ることを確認済み。

## 運用ルール追記(2026-07-18、正本はopen-raid-zのCLAUDE.md参照) — 確認不要の自動継続・リミット解除後の自動再開

- **コンテキストウインドウ・5時間利用制限・その他のセッション中断が
  発生し、その後リミットが解除されて新しいセッションが開始された場合、
  「続けてよろしいですか」等の確認を挟まず、毎回自動的に前回セッションの
  続きの作業を再開すること**(ユーザー指示、2026-07-18)。具体的には:
  1. セッション開始時、各リポジトリの`git status`/`git log`と、この
     `CLAUDE.md`(および他プロジェクトのCLAUDE.md)のHANDOFF節・
     「次にすべきこと」記載を確認し、未完了・未pushの作業が無いかを
     まず裏取りする(タスク管理メタデータを鵜呑みにしない既存方針と
     同じ姿勢で、実際のgit状態を確認する)。
  2. 未完了作業が見つかった場合、ユーザーへの確認を求めず、そのまま
     自動的に検証(build/test)→修正→コミット→pushまで完了させる。
  3. 完了している場合は、各CLAUDE.mdの「次にすべきこと」「未着手・
     未完成」に記載された次の項目へ確認なしに着手する(既存の
     「未着手だからといって確認を求めて手を止めない」方針の延長)。
  4. 「続けてよろしければそのまま自動開発を継続します」のような、
     続行そのものを尋ねる確認は今後一切行わない(ユーザー指示、
     2026-07-18)。作業内容の要約・進捗報告はしてよいが、それは
     承認を求めるものではなく完了報告として書く。
  5. こまめにコミット・pushしておくことで、次回セッションが「どこから
     再開すべきか」を迷わず`git log`/CLAUDE.mdから機械的に判断できる
     ようにしておく(区切りがついた時点で都度コミット・pushする既存
     方針との組み合わせ)。


## 運用ルール追記(2026-07-19、正本はopen-raid-zのCLAUDE.md参照) — 白画面バグ等を見逃さない検証徹底

- **WEB/UIを持つ機能を実装した後は、ビルド成功・`cargo test`・curlでの
  ステータスコード確認だけで「完了」と報告せず、実際に画面が正しく
  表示される(白画面・レンダリング崩れ・コンソールエラーが無い)ところ
  まで確認すること**(ユーザー指示、2026-07-19)。
  1. ブラウザ操作が可能な環境では、実際にページを開いて表示内容
     (見出し・本文・想定した要素の存在)とコンソールエラーの有無を
     確認する。
  2. ブラウザ操作ができない環境では、少なくとも`curl`等でHTMLボディの
     中身を取得し、期待される文字列が実際に含まれているかを確認する
     ——ステータスコード200だけを見て「動作確認済み」としない。
  3. 白画面・エラー・期待した内容の欠落等の不具合が見つかった場合は、
     確認を求めず自動的に原因調査・修正・再確認まで行う。
  4. 本番ドメインが未取得・DNS未設定なだけの状態は上記の「白画面
     バグ」とは別物であり、混同しない(`localhost`確認で代替可)。


## HANDOFF(直近の作業ログ、上が最新)

- **2026-08-08**: TOPページに3件目のブログリンク(「足振りで腰痛改善＋
  ダイエット」、`ameblo.jp/www-aon/entry-12975130765`)を追加
  (`BLOG_POST3_URL`/`BLOG_POST3_TITLE_JA`定数)。ユーザー指示により
  掲載位置は「民間のガン治療法に関する報道」セクション(`{cancer_news_
  section}`)の直下(既存の2件のブログリンクは`<header>`直前に配置されて
  いるのに対し、今回はユーザー指示の位置を優先し別の箇所へ配置——既存
  スタイル`<p class="blog-link">`は流用)。検証: `cargo build --release`
  成功、実バイナリをローカル起動し`curl`でTOPページHTML内に新リンクが
  実在すること(href/リンクテキストとも正しい値、HTTP 200)を確認済み。

- **2026-08-04**: (1) 「民間のガン治療法に関する報道」セクションに、
  金沢大学の分子標的療法紹介動画(https://youtu.be/u4xTbs4JZ30)を追加。
  (2) TOPページに2件目のブログリンク(「上下水道配管や屋根瓦などの
  ハイテク新素材。パナホームとヤマダホームのコーキングレス外壁」、
  `ameblo.jp/www-aon/entry-12974607800`)を既存の1件目と同じ位置・スタイルで
  追加(`BLOG_POST2_URL`/`BLOG_POST2_TITLE_JA`定数)。検証: `cargo build
  --release`成功、VPS本番反映・実インターネット経由での表示確認済み。

- **2026-07-24**: TOPページに「民間のガン治療法に関する報道」セクション
  (`render_cancer_news_section`)を追加。ユーザー提供の実際の報道見出し・
  リンクをそのまま紹介するのみに留め、独自の医療的な効能・安全性の主張や
  推奨は一切追加していない。掲載内容:
  (1) 大阪公立大の肝臓がん新治療法の報道(見出し日英併記) —
  Facebookリンク(https://www.facebook.com/masahiro.ishizuka.54?locale=ja_JP)と
  YouTubeリンク(https://www.youtube.com/watch?v=hRFXYCGX8Fo)を、
  「片方が見られなくなった場合の予備」の位置づけで単純に2つ併記
  (特別なフェイルオーバー機構は無し、ユーザー指示通り)。
  (2) 九州大学の"からだ自身ががん治療"報道(見出し日英併記、YouTubeリンク
  https://www.youtube.com/watch?v=84EkcJmgmnQ)。
  (3) 「民間のガン治療法についての情報は aon.tokyo/cancer をご覧ください」
  リンク(日英併記)。
  **検証**: `cargo build`/`cargo test`成功。実バイナリ起動+`curl`で
  TOPページHTML内に上記の日英見出し・全リンク先URL(Facebook/YouTube×2/
  aon.tokyo/cancer)が実在することを確認済み。

- **2026-07-20(追記)**: ユーザー指示により、`src/i18n.rs`の
  `LABEL_CLAUDE_JA`定数(`/open-aruaru-runo-iLumi`ページでCLAUDE.mdへの
  リンクに使う表示ラベル)を「CLAUDE.md(設計思想＆開発方針＆開発環境
  ルール)」から「CLAUDE.md(WEBアプリ設計思想、開発方針、開発環境
  ルール)」へ改名(ファイル名`CLAUDE.md`自体は変更なし、表示ラベルのみ)。
  `cargo build --release`後、実機起動(`127.0.0.1:4188`)し`curl`で
  `/open-aruaru-runo-iLumi`のHTML内に新ラベル文字列が16箇所含まれる
  こと・HTTP 200であることを確認済み。

- **2026-07-20**: エコシステム全体のメタ索引リポジトリ(`aon-co-jp/aon`、
  旧称`open-aruaru-runo-iLumi`——作業開始時点ではこの旧称でユーザーが
  作成済みだったが、作業途中でユーザー側が`aon`へリポジトリ名変更・
  内容移行済みであることが判明し、本文中のGitHubリンク表記を追従修正)
  と同内容の索引ページを本サーバーに実装。
  - 新規モジュール`src/i18n.rs`(13言語対応: 日本語・英語(米/英)・
    中国語簡体字/繁体字・韓国語・イタリア語・フランス語・ドイツ語・
    アラビア語・ペルシャ語・ロシア語・ウクライナ語、`?lang=`クエリ
    パラメータ方式、姉妹リポジトリ`e-gov.info`の`src/i18n.rs`と同じ
    設計パターンを踏襲。既定言語は本サイトの既存TOPページに合わせ
    日本語)と`src/meta_index.rs`(プロジェクト一覧データ・GitHub風
    ファイル一覧描画・ページ組み立て)を追加。
  - ルート`/open-aruaru-runo-iLumi`(メイン)と`/open-aruaru-runo`
    (エイリアス、同一ハンドラ)を新設。両方とも200を返すことを確認済み。
  - 掲載16プロジェクト(RCosmo/RFrontEnd/RPoem/aruaru-db/aruaru-llm/
    aruaru-tokyo/audiocafe-tokyo-rust/audiocafe.tokyo/e-gov.info/
    karu.tokyo/open-cuda/open-easy-web/open-raid-z/open-web-server/
    rs-to-readme/aon)は、`F:\open-runo`配下で実際にgitリポジトリとして
    存在し`.git/config`のリモートURLで`aon-co-jp`上の実在を確認できた
    ものだけを掲載(推測での追加はしていない)。`open-cuda`はREADME.md/
    CLAUDE.md/PORTING.mdが存在しないため、ファイル一覧にその旨を表示。
  - 各プロジェクトカードに「🔄 GitHubから最新情報を取得」ボタンを設置。
    クリックするとクライアントサイドJSが`https://api.github.com/repos/
    <owner>/<repo>`へ直接fetchし、⭐Star数・最終更新日時・既定ブランチを
    その場で表示する(認証トークン不要、レート制限到達時・ネットワーク
    到達不能時は静的情報のみのフォールバック表示に留まる作り)。
  - TOPページ(`/`)の quick-links に「📚 プロジェクトシリーズ」リンクを
    追加(TOPページ自体は13言語版を持たないため、既存の日本語/英語
    併記スタイルのまま1つのリンクを追加するにとどめた)。
  - **検証**: `cargo check`/`cargo test`が警告無しで通過。
    `cargo run --release`で実機起動(`127.0.0.1:4177`)し、`curl`で
    (1) TOPページHTML内に`href="/open-aruaru-runo-iLumi"`が含まれる
    こと、(2) `/open-aruaru-runo-iLumi`・エイリアス`/open-aruaru-runo`
    ともにHTTP 200で、HTML内に文字列
    「設計思想＆開発方針＆開発環境ルール」・`github.com/aon-co-jp`への
    リンク(62箇所)・`aon-co-jp/aon`が含まれること、(3) `?lang=en`で
    見出しが英語に切り替わること、(4) `?lang=ar`で`dir="rtl"`が
    付与されることを確認済み。GitHub API(`api.github.com`)自体への
    到達性も`curl`で確認済み(`aon-co-jp/aruaru-tokyo`・`aon-co-jp/aon`
    ともにHTTP 200・実データ取得成功)——ただしクライアントサイドJSの
    fetchボタンの実クリック動作は、ブラウザ操作環境が無かったため
    未検証(正直な開示)。GitHub API自体の到達性・レスポンス形式は
    サーバー側から確認済みであり、`fetch`実装も同一エンドポイント・
    同一レスポンス形式(`stargazers_count`/`pushed_at`/`default_branch`)
    を前提にした標準的な実装のため、動作する見込みが高いと判断している。
  - **制約・今後の課題**: 各プロジェクトの役割説明(`role_ja`)は日本語
    のみで、13言語へは翻訳していない(UIの見出し・ラベル・ボタン等は
    13言語対応済みだが、16件分の説明文を13言語へ翻訳すると分量・
    翻訳精度検証コストが大きいため、`e-gov.info`の既存スコープ限定
    方針に倣った設計判断——詳細は`src/i18n.rs`冒頭コメント参照)。

- **2026-07-16**: GitHub連携機能を拡張。(1) GitHub organization(`aon-co-jp`)トップページへのリンクをページ下部に追加。(2)「🔄 最新のリポジトリ一覧を取得」ボタンで、クリックした瞬間にGitHub APIから最新の全リポジトリ名を取得し`<select>`を動的に差し替える機能(`GET /api/repos`)を新設。(3) 選択中のリポジトリのGitHub URLへの直接リンクを追加。(4) README等のファイル表示に、既存の`.rs`変換に加えて`pulldown-cmark`によるGitHub風レンダリング(見出し・リスト・コードブロック・テーブル等)を追加し、タブで切替可能に。(5) README表示エリアを`main`の780px制約からbreak-outさせ、横幅いっぱい(94vw、最大1400px)で表示するよう改善。**実装中に発見した実バグ**: `aon-co-jp`はGitHub Organizationではなく個人アカウントのため、`/orgs/{name}/repos`は404を返す——`/users/{name}/repos`が正しいエンドポイント(実際にcurlで両方検証済み)。またリポジトリ表示の妥当性検証を静的リスト照合(`GITHUB_REPOS.contains`)から形式検証(`is_valid_repo_name`)に変更——動的取得した新しいリポジトリが静的ホワイトリストで弾かれるバグを防ぐため。VPS実機でビルド・再起動・`curl`による全機能(`/api/repos`・repo表示・markdown-body/view-toggle/repo-link/org-link要素の存在)を確認済み。

- **2026-07-15**: PHPからRust+Poemへの書き換え・VPS本番投入完了。`ARUARU_EASYWEB_URL`がループバックアドレス(`127.0.0.1:8080`)を指していたバグを発見・修正(公開URL`https://runo.tokyo/`に変更)。README/CLAUDE.md/PORTING.md新規作成。

## アプリケーションサーバー層の役割(open-runo / poem-cosmo-tauri、2026-07-16追記)

「配信エンジン(vhost)」に`open-web-server`を選択肢として追加したが、
open-web-serverがApache＋Nginxのハイブリッド仕様のWebサーバーとして
まだ機能していない間は、Tomcatのような互換レイヤーとして機能するのは
`open-runo`または`poem-cosmo-tauri`である。

これらは`open-raid-z`とVersionlessAPIによって、バージョンレス運用と
バージョン管理・Git管理を両立しながら、ACID互換性とZFS互換性に対応した
`aruaru-db`と、PostgreSQLとのDUAL DATABASE構成による「4層4重」の
最新鋭の通信システムを構築し、仕様変更が容易なデータベース設計により、
3DオンラインゲームAI課金アイテム、オンライン金融、オンライン証券、
オンラインクレジットカード決済など、ネット上で紛失してはならない
ミッションクリティカルな用途向けに、24時間365日ノンストップの
サーバー対応WEBサイト開発を全面的にバックアップするフレームワーク・
ミドルウェアとして機能することを目指す。

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

- **2026-10-26 索引ページをメタ索引(`aon-co-jp/RUNO`の`README.md`)に同期**: `src/meta_index.rs`の`PROJECTS`を、索引READMEの表(40行)から再生成した。
  追加21件(aruaru-IME・aruaru-search・aruaru-vpn・open-github-backup・backup-to-discord-by-github・open-bar・open-av・open-mqa-dsd・open-audio-sr・open-music-llm・open-LiveKit・open-tv-chat・
  open-vsc-claude・open-directx・maid-cafe-se・360-speaker・version-control・webeditor-sftp・realdata.pro・rs-dpiAwareness・aruaru-db-archive)、
  削除2件(削除済みの`open-english-pc`、旧称の`aon`→`RUNO`行に置換)、サイト独自の`aruaru.tokyo`は残した。役割文はREADMEの表から記号(`**`・バッククォート・リンク)を除いたもの。
  再生成スクリプトは使い捨て(リポジトリには入れていない)。**自動同期の仕組みは無いまま**なので、索引READMEを変えたら再度同期が要る。
  検証: `cargo test`は通過(テストは0件)。実機でページを開いての表示確認は未実施(ブラウザのプレビュー起動が自動で止められたため)。

## 依存の更新(2026-11-03)
reqwest 0.13(`rustls-tls`→`rustls`)、rand 0.10(`thread_rng`→`rng()`、`choose`は`IndexedRandom`)、pulldown-cmark 0.13。ビルド・テスト39件通過。VPSへの反映(build)は未実施。

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

### 追加の実機TEST(2026-11-07)/ Additional live tests
- **日本語**: Linux(WSL Ubuntu・systemd): インストーラーと同じユニット(`Restart=always`)で `aruaru-ime-daemon` を動かし、SIGTERM・SIGKILL のどちらでも約10秒以内に別PIDで復帰、`systemctl stop` では止まったままを確認。Android 実機(moto g53y・Android 14): IME のプロセスを強制終了すると OS が再生成(PID 24034→24172)。端末の既定キーボードは試験後に Gboard へ戻した。端末に入っている版は 0.2.21 でリリースは 0.2.23(未更新)。macOS の `KeepAlive` は実機が無く、設定の静的確認のみ。
- **REST 移行の副作用で見つけた不具合を修正**: VPS の `maid-news-archive` が旧 `GET /v1/internal/news/export`(移行で削除済み)を呼んで404で失敗していたため、`open-english` の `scripts/archive-articles-to-github.sh` を `POST /openenglish.v1.Internal/NewsExport` に直して反映(成功を確認)。`daily-news-collect` は `scripts/daily-news-collect.sh` の実行権限が無く失敗していたので VPS で `chmod +x`(git 上のモードは 644 のままで、`aruaru-llm` 側で直す必要あり)。
- **English**: Linux (WSL Ubuntu, systemd): the installer's unit (`Restart=always`) brought `aruaru-ime-daemon` back with a new PID within ~10 s after both SIGTERM and SIGKILL, and `systemctl stop` stayed stopped. Android device (moto g53y, Android 14): force-killing the IME process made the OS recreate it (PID 24034→24172); the default keyboard was restored to Gboard afterwards. The installed build is 0.2.21 vs release 0.2.23 (not updated). macOS `KeepAlive` was only checked statically (no device).
- **Bugs found as side effects of the REST migration, fixed**: `maid-news-archive` on the VPS called the removed `GET /v1/internal/news/export` (404); `open-english`'s `scripts/archive-articles-to-github.sh` now uses `POST /openenglish.v1.Internal/NewsExport` and is deployed (verified). `daily-news-collect` failed because `scripts/daily-news-collect.sh` lost its exec bit; fixed with `chmod +x` on the VPS (git still records mode 644; `aruaru-llm` needs the fix).
