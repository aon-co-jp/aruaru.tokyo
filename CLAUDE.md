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
